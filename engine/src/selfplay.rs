use std::time::Duration;

use crate::Engine;
use crate::chess::{GameState, Move};

/// Configuration for a self-play game.
pub struct SelfPlayConfig {
    pub time_per_move: Duration,
    pub start_fen: Option<String>,
}

impl Default for SelfPlayConfig {
    fn default() -> Self {
        Self {
            time_per_move: Duration::from_millis(100),
            start_fen: None,
        }
    }
}

/// Summary of a completed self-play game.
pub struct GameSummary {
    pub result: GameState,
    pub plies: u32,
    pub moves: Vec<Move>,
    pub board: String,
}

/// Run a single self-play game where the engine plays both sides.
pub fn play_one(config: &SelfPlayConfig) -> GameSummary {
    let mut engine = if let Some(fen) = &config.start_fen {
        Engine::from_fen(fen)
    } else {
        Engine::new()
    };

    engine.set_search_time_limit(config.time_per_move);

    let mut moves = Vec::new();
    let mut plies: u32 = 0;

    while engine.game_state() == GameState::Playing {
        let best_move = engine.best_move();
        engine.make_move(best_move);
        println!("{}", engine);
        moves.push(best_move);
        plies += 1;
    }

    GameSummary {
        result: engine.game_state(),
        plies,
        moves,
        board: engine.to_string(),
    }
}

/// Run multiple self-play games and return their summaries.
pub fn play_many(num_games: u32, base_config: &SelfPlayConfig) -> Vec<GameSummary> {
    let mut results = Vec::with_capacity(num_games as usize);

    for _ in 0..num_games {
        results.push(play_one(base_config));
    }

    results
}
