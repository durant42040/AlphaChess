use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use engine::engine::Engine;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use stockfish::Stockfish;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

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
    stockfish.play_move(&move_string)?;

    engine.act(move_string.clone());

    Ok((
        StatusCode::OK,
        Json(
            json!({ "move": move_string, "board": engine.to_string(), "isCheck": engine.is_check() }),
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

    println!("making move: {}", move_string);

    if !engine.act(move_string.clone()) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Illegal move" })),
        ));
    }
    stockfish.play_move(&move_string)?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "board": engine.to_string(),
            "isCheck": engine.is_check()
        })),
    ))
}

async fn reset(State(state): State<AppState>) -> StatusCode {
    let mut engine = state.engine.lock().await;
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

    let app = Router::new()
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game))
        .with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
