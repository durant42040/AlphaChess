use engine::play::{
    best_move_vs_lazy_smp, parse_lazy_smp, LazySmpConfig,
    print_overall_result,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut config = LazySmpConfig::default();
    parse_lazy_smp(&args, &mut config);

    let mut results = Vec::with_capacity(config.num_games as usize);
    for game in 1..=config.num_games {
        println!("=== Game {} / {} (White: best_move, Black: lazy_smp {} threads) ===", 
            game, config.num_games, config.num_threads);
        let summary = best_move_vs_lazy_smp(&config);
        println!("Result: {}, plies: {}", summary.result, summary.plies);
        println!("PGN:\n{}", summary.pgn);
        results.push(summary);
    }
    print_overall_result(&results);
}
