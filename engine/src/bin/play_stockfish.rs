use engine::play::{SelfPlayConfig, play_stockfish};
use std::time::Duration;

fn main() {
    let config = SelfPlayConfig {
        time_per_move: Duration::from_millis(10),
        start_fen: None,
    };

    let summary = play_stockfish(&config);
    println!("Result: {}, plies: {}", summary.result, summary.plies);
    println!("Board: {}", summary.board);
    println!("PGN: \n{}", summary.pgn);
}
