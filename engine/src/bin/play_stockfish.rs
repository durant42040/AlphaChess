use std::time::Duration;

use engine::play::{SelfPlayConfig, play_stockfish};

fn main() {
    let config = SelfPlayConfig {
        time_per_move: Duration::from_millis(1000),
        start_fen: None,
    };
    
    let summary = play_stockfish(&config);
    println!("Result: {}, plies: {}", summary.result, summary.plies);
    println!("Board: {}", summary.board);
    println!(
        "Moves: {}",
        summary
            .moves
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
}
