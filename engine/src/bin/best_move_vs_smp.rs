use engine::play::{SMPConfig, best_move_vs_smp, parse_smp, print_overall_result};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut config = SMPConfig::default();
    parse_smp(&args, &mut config);

    let mut results = Vec::with_capacity(config.num_games as usize);
    for game in 1..=config.num_games {
        println!(
            "=== Game {} / {} (White: best_move, Black: smp {} threads) ===",
            game, config.num_games, config.num_threads
        );
        let summary = best_move_vs_smp(&config);
        println!("Result: {}, plies: {}", summary.result, summary.plies);
        println!("PGN:\n{}", summary.pgn);
        results.push(summary);
    }
    print_overall_result(&results);
}
