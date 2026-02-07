use std::time::Duration;

use shakmaty::san::San;
use shakmaty::{Chess, Position, uci::UciMove};
use stockfish::Stockfish;

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
            time_per_move: Duration::from_millis(10),
            start_fen: None,
        }
    }
}

pub fn pgn(moves: &[Move]) -> String {
    let mut pgn = String::new();
    let mut pos = Chess::default();

    for (i, r#move) in moves.iter().enumerate() {
        if i.is_multiple_of(2) {
            pgn.push_str(&format!("\n{}.", i / 2 + 1));
        } else {
            pgn.push(' ');
        }
        let uci = r#move.to_string().parse::<UciMove>().expect("bad uci");
        let m = uci.to_move(&pos).expect("illegal move for position");
        let san = San::from_move(&pos, m);
        pgn.push_str(&san.to_string());
        pos.play_unchecked(m);
        pgn.push(' ');
    }

    pgn
}

/// Summary of a completed self-play game.
pub struct GameSummary {
    pub result: GameState,
    pub plies: u32,
    pub board: String,
    pub pgn: String,
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

    let pgn = pgn(&moves);

    GameSummary {
        result: engine.game_state(),
        plies,
        board: engine.to_string(),
        pgn,
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

pub fn play_stockfish(config: &SelfPlayConfig) -> GameSummary {
    let mut engine = Engine::new();
    engine.set_search_time_limit(config.time_per_move);

    let mut stockfish = Stockfish::new("stockfish").unwrap();
    stockfish.setup_for_new_game().unwrap();
    stockfish.set_depth(8);

    let mut moves = Vec::new();
    let mut plies: u32 = 0;

    while engine.game_state() == GameState::Playing {
        if plies.is_multiple_of(2) {
            let best_move = engine.best_move();
            engine.make_move(best_move);
            stockfish.play_move(&best_move.to_string()).unwrap();
            moves.push(best_move);
        } else {
            let stockfish_output = stockfish.go().unwrap();
            let move_string = stockfish_output.best_move();
            let best_move = Move::from(move_string);
            engine.make_move(best_move);
            stockfish.play_move(move_string).unwrap();
            moves.push(best_move);
        }
        let stockfish_eval = stockfish.go().unwrap().eval().value();
        println!(
            "\x1b[1;31m[Stockfish]\x1b[0m \x1b[1mstockfish eval\x1b[0m \x1b[1;34m{}\x1b[0m",
            stockfish_eval
        );
        println!("{}", engine);
        plies += 1;
    }

    let pgn = pgn(&moves);

    GameSummary {
        result: engine.game_state(),
        plies,
        board: engine.to_string(),
        pgn,
    }
}
