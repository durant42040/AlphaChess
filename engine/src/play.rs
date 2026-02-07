use std::time::Duration;

use shakmaty::san::San;
use shakmaty::{Chess, Position, uci::UciMove};
use stockfish::Stockfish;

use crate::Engine;
use crate::chess::{GameState, Move};

/// Parse standard play options from CLI args: `--ponder`/`-p` (ms), `--games`/`-n` (count, default 1).
pub fn parse_args(args: &[String], config: &mut SelfPlayConfig) {
    let mut ponder_ms = None;
    let mut num_games = None;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if (a == "--ponder" || a == "-p") && args.get(i + 1).is_some() {
            ponder_ms = args[i + 1].parse().ok();
            i += 1;
        } else if (a == "--games" || a == "-n") && args.get(i + 1).is_some() {
            num_games = args[i + 1].parse().ok();
            i += 1;
        }
        i += 1;
    }
    if let Some(num) = num_games {
        config.num_games = num;
    }
    if let Some(ms) = ponder_ms {
        config.ponder_time = Duration::from_millis(ms);
    }
}

/// Configuration for a self-play game.
pub struct SelfPlayConfig {
    pub ponder_time: Duration,
    pub start_fen: Option<String>,
    pub num_games: u32,
}

impl Default for SelfPlayConfig {
    fn default() -> Self {
        Self {
            ponder_time: Duration::from_millis(10),
            start_fen: None,
            num_games: 1,
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

/// Print overall result summary (white wins, black wins, draws).
pub fn print_overall_result(summaries: &[GameSummary]) {
    let white = summaries
        .iter()
        .filter(|s| s.result == GameState::WhiteWin)
        .count();
    let black = summaries
        .iter()
        .filter(|s| s.result == GameState::BlackWin)
        .count();
    let draws = summaries
        .iter()
        .filter(|s| s.result == GameState::Draw)
        .count();
    println!(
        "\nOverall: {} games — White {}, Black {}, Draw {}",
        summaries.len(),
        white,
        black,
        draws
    );
}

/// Run a single self-play game where the engine plays both sides.
pub fn self_play(config: &SelfPlayConfig) -> GameSummary {
    let mut engine = if let Some(fen) = &config.start_fen {
        Engine::from_fen(fen)
    } else {
        Engine::new()
    };

    engine.set_search_time_limit(config.ponder_time);

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
        results.push(self_play(base_config));
    }

    results
}

pub fn play_stockfish(config: &SelfPlayConfig) -> GameSummary {
    let mut engine = Engine::new();
    engine.set_search_time_limit(config.ponder_time);

    let mut stockfish = Stockfish::new("stockfish").unwrap();
    stockfish.setup_for_new_game().unwrap();
    stockfish.set_depth(7);

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
            "\x1b[1;31m[Stockfish]\x1b[0m\n\x1b[1mstockfish eval\x1b[0m \x1b[1;34m{}\x1b[0m",
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
