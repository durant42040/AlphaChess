//! HTTP client for the chess server API.

use gloo::net::http::Request;
use serde::Deserialize;

/// Base URL for the API. In production, use same origin or set via build env.
fn base_url() -> String {
    #[cfg(not(debug_assertions))]
    return String::new();
    #[cfg(debug_assertions)]
    return std::option_env!("API_BASE_URL").unwrap_or("http://localhost:4000").to_string();
}

fn api_url(path: &str) -> String {
    let base = base_url();
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), path)
    }
}

#[derive(Deserialize)]
pub struct ActResponse {
    pub board: String,
    #[serde(rename = "isCheck")]
    pub is_check: bool,
}

#[derive(Deserialize)]
pub struct GenerateResponse {
    #[serde(rename = "move")]
    pub move_str: String,
    pub board: String,
    #[serde(rename = "isCheck")]
    pub is_check: bool,
}

#[derive(Deserialize)]
pub struct UndoResponse {
    pub board: String,
    #[serde(rename = "isCheck")]
    pub is_check: bool,
}

#[derive(Deserialize)]
pub struct GameResponse {
    #[serde(rename = "gameState")]
    pub game_state: String,
}

/// POST/GET reset - server returns 200 with no body.
pub async fn reset() -> Result<(), String> {
    let url = api_url("/reset");
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        return Err(format!("reset failed: status {}", resp.status()));
    }
    Ok(())
}

/// GET /act?move=e2e4
pub async fn act(move_str: &str) -> Result<ActResponse, String> {
    let path = format!("/act?move={}", move_str);
    let url = api_url(&path);
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        let text = resp.text().await.unwrap_or_default();
        return Err(text);
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// GET /generate - server computes best move and applies it.
pub async fn generate() -> Result<GenerateResponse, String> {
    let url = api_url("/generate");
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        let text = resp.text().await.unwrap_or_default();
        return Err(text);
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// GET /undo - server undoes two half-moves.
pub async fn undo() -> Result<UndoResponse, String> {
    let url = api_url("/undo");
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        let text = resp.text().await.unwrap_or_default();
        return Err(text);
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// GET /game - returns current game state string.
pub async fn game_state() -> Result<String, String> {
    let url = api_url("/game");
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        return Err(format!("game failed: status {}", resp.status()));
    }
    let json: GameResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json.game_state)
}
