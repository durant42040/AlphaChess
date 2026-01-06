use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use engine::engine::Engine;
use serde::Deserialize;
use serde_json::{Value, json};

mod stockfish;

#[derive(Deserialize)]
struct MoveQuery {
    r#move: String,
}

async fn generate_move(State(mut engine): State<Engine>) -> (StatusCode, Json<Value>) {
    let move_string = match stockfish::generate_move() {
        Ok(move_string) => move_string,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate move" })),
            );
        }
    };

    engine.act(move_string.clone());

    (
        StatusCode::OK,
        Json(
            json!({ "move": move_string, "board": engine.get_board(), "isCheck": engine.is_check() }),
        ),
    )
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
    let engine = Engine::new();
    let app = Router::new()
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game))
        .with_state(engine);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("Server is running on port 4000");
    axum::serve(listener, app).await.unwrap();
}
