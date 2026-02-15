use engine::play::{PlayConfig, parse_play_args, play};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut config = PlayConfig::default();

    if let Err(e) = parse_play_args(&args, &mut config) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    play(&config);
}
