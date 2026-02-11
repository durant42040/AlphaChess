use engine::Engine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut fen = None;

    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        if (a == "-f" || a == "--fen") && args.get(i + 1).is_some() {
            fen = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let fen = fen.unwrap_or_else(|| {
        eprintln!("Usage: cargo run --bin analyze -- -f <fen>");
        std::process::exit(1);
    });

    let mut engine = Engine::from_fen(&fen);
    engine.set_ponder_time(1000);
    engine.best_move();
}
