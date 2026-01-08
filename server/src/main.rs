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
use stockfish::Stockfish;
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct MoveQuery {
    r#move: String,
}

struct StockfishError(std::io::Error);

impl IntoResponse for StockfishError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
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
    State(mut engine): State<Engine>,
) -> Result<(StatusCode, Json<Value>), StockfishError> {
    let mut stockfish = Stockfish::new("stockfish")?;
    let _ = stockfish.setup_for_new_game();

    for r#move in engine.get_moves() {
        stockfish.play_move(r#move.to_string().as_str())?;
    }

    let move_string = stockfish.go()?.to_string();

    engine.act(move_string.clone());

    Ok((
        StatusCode::OK,
        Json(
            json!({ "move": move_string, "board": engine.get_board(), "isCheck": engine.is_check() }),
        ),
    ))
}

async fn make_move(
    State(mut engine): State<Engine>,
    Query(params): Query<MoveQuery>,
) -> (StatusCode, Json<Value>) {
    let move_string = params.r#move;
    if move_string.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Move is required" })),
        );
    }
    if !engine.act(move_string.clone()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Illegal move" })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "board": engine.get_board(),
            "isCheck": engine.is_check()
        })),
    )
}

async fn reset(State(mut engine): State<Engine>) -> StatusCode {
    engine.reset();
    StatusCode::OK
}

async fn game(State(engine): State<Engine>) -> Json<Value> {
    Json(json!({ "gameState": engine.get_game_state() }))
}

#[tokio::main]
async fn main() {
    let port = 4000;
    let engine = Engine::new();

    let app = Router::new()
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game))
        .with_state(engine);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
