use engine::play::{SelfPlayConfig, parse_args, play_stockfish};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut config = SelfPlayConfig::default();
    parse_args(&args, &mut config);

    let mut results = Vec::with_capacity(config.num_games as usize);
    for game in 1..=config.num_games {
        println!("=== Game {} / {} ===", game, config.num_games);
        let summary = play_stockfish(&config);
        println!("Result: {}, plies: {}", summary.result, summary.plies);
        println!("PGN: \n{}", summary.pgn);
        results.push(summary);
    }
    engine::play::print_overall_result(&results);
}
