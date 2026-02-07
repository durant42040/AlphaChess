use engine::play::{SelfPlayConfig, play_one};

fn main() {
    let config = SelfPlayConfig::default();
    let summary = play_one(&config);
    println!("Result: {}, plies: {}", summary.result, summary.plies);
    println!("Board: {}", summary.board);
    println!("PGN: \n{}", summary.pgn);
}
