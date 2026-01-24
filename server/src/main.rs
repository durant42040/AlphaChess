use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use engine::Engine;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use stockfish::Stockfish;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct MoveQuery {
    r#move: String,
}

#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<Engine>>,
    stockfish: Arc<Mutex<Stockfish>>,
}

struct StockfishError(std::io::Error);

impl IntoResponse for StockfishError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("stockfish io error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for StockfishError
where
    E: Into<std::io::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn generate_move(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), StockfishError> {
    let mut engine = state.engine.lock().await;
    let mut stockfish = state.stockfish.lock().await;

    let stockfish_output = stockfish.go()?;
    let move_string = stockfish_output.best_move();

    stockfish.play_move(move_string)?;

    engine.act(&move_string);
    println!("{}", engine);

    Ok((
        StatusCode::OK,
        Json(
            json!({ "move": move_string, "board": engine.to_board_string(), "isCheck": engine.is_check() }),
        ),
    ))
}

async fn make_move(
    State(state): State<AppState>,
    Query(params): Query<MoveQuery>,
) -> Result<(StatusCode, Json<Value>), StockfishError> {
    let mut engine = state.engine.lock().await;
    let mut stockfish = state.stockfish.lock().await;

    let move_string = params.r#move;
    if move_string.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Move is required" })),
        ));
    }

    if !engine.act(&move_string) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Illegal move" })),
        ));
    }
    stockfish.play_move(&move_string)?;
    println!("{}", engine);

    Ok((
        StatusCode::OK,
        Json(json!({
            "board": engine.to_board_string(),
            "isCheck": engine.is_check()
        })),
    ))
}

async fn undo_move(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), StockfishError> {
    let mut engine = state.engine.lock().await;
    let mut stockfish = state.stockfish.lock().await;

    // undo twice to get back to the previous turn
    if !engine.undo() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No moves to undo" })),
        ));
    }
    if !engine.undo() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No moves to undo" })),
        ));
    }
    println!("{}", engine);
    stockfish.set_fen_position(&engine.get_fen())?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "board": engine.to_board_string(),
            "isCheck": engine.is_check()
        })),
    ))
}

async fn reset(State(state): State<AppState>) -> StatusCode {
    let mut engine = state.engine.lock().await;
    let mut stockfish = state.stockfish.lock().await;
    stockfish.setup_for_new_game().unwrap();
    stockfish
        .set_fen_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();
    stockfish.set_depth(20);

    engine.reset();
    StatusCode::OK
}

async fn game(State(state): State<AppState>) -> Json<Value> {
    let engine = state.engine.lock().await;
    Json(json!({ "gameState": engine.get_game_state() }))
}

#[tokio::main]
async fn main() {
    let port = 4000;

    let engine = Arc::new(Mutex::new(Engine::new()));

    let stockfish = Arc::new(Mutex::new(
        Stockfish::new("stockfish").expect("Failed to initialize Stockfish"),
    ));
    stockfish.lock().await.setup_for_new_game().unwrap();
    stockfish.lock().await.set_depth(20);

    let app_state = AppState { engine, stockfish };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game))
        .route("/undo", get(undo_move))
        .layer(cors)
        .with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
