use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use engine::Engine;
use engine::chess::Move;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct MoveQuery {
    r#move: String,
}

async fn best_move(State(state): State<Arc<Mutex<Engine>>>) -> (StatusCode, Json<Value>) {
    let mut engine = state.lock().await;
    use engine::chess::GameState;
    if engine.game_state() != GameState::Playing {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Game is not in progress" })),
        );
    }
    let best = engine.best_move_smp(4);
    (StatusCode::OK, Json(json!({ "move": best.to_string() })))
}

async fn generate_move(State(state): State<Arc<Mutex<Engine>>>) -> (StatusCode, Json<Value>) {
    let mut engine = state.lock().await;
    let best_move = engine.best_move();
    engine.make_move(best_move);
    println!("{}", engine);

    (
        StatusCode::OK,
        Json(
            json!({ "move": best_move.to_string(), "board": engine.to_board_string(), "isCheck": engine.is_check() }),
        ),
    )
}

async fn make_move(
    State(state): State<Arc<Mutex<Engine>>>,
    Query(params): Query<MoveQuery>,
) -> (StatusCode, Json<Value>) {
    let mut engine = state.lock().await;

    let move_string = params.r#move;
    if move_string.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Move is required" })),
        );
    }

    if !engine.make_move(Move::from_string(&move_string)) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Illegal move" })),
        );
    }
    println!("{}", engine);

    (
        StatusCode::OK,
        Json(json!({
            "board": engine.to_board_string(),
            "isCheck": engine.is_check()
        })),
    )
}

async fn undo_move(State(state): State<Arc<Mutex<Engine>>>) -> (StatusCode, Json<Value>) {
    let mut engine = state.lock().await;

    // undo twice to get back to the previous turn
    if !engine.unmake_move() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No moves to undo" })),
        );
    }
    if !engine.unmake_move() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No moves to undo" })),
        );
    }
    println!("{}", engine);

    (
        StatusCode::OK,
        Json(json!({
            "board": engine.to_board_string(),
            "isCheck": engine.is_check()
        })),
    )
}

async fn reset(State(state): State<Arc<Mutex<Engine>>>) -> StatusCode {
    let mut engine = state.lock().await;
    engine.reset();
    println!("{}", engine);
    StatusCode::OK
}

async fn game(State(state): State<Arc<Mutex<Engine>>>) -> Json<Value> {
    let engine = state.lock().await;
    Json(json!({ "gameState": engine.game_state().to_string() }))
}

async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".into());

    let engine = Arc::new(Mutex::new(Engine::new()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ping", get(ping))
        .route("/best-move", get(best_move))
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game))
        .route("/undo", get(undo_move))
        .layer(cors)
        .with_state(engine);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
