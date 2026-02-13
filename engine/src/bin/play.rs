use std::io::{self, BufRead, Write};

use engine::Engine;
use engine::chess::{GameState, Move, Player};
use engine::play::pgn;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut ponder_ms = 1000u64;
    let mut user_white = true;
    let mut use_smp = false;
    let mut num_threads = 2usize;

    let mut i = 1;
    while i < args.len() {
        if (args[i] == "-p" || args[i] == "--ponder") && args.get(i + 1).is_some() {
            ponder_ms = args[i + 1].parse().unwrap_or(1000);
            i += 1;
        } else if (args[i] == "-s" || args[i] == "--side") && args.get(i + 1).is_some() {
            let s = args[i + 1].to_lowercase();
            user_white = matches!(s.as_str(), "white" | "w");
            if !matches!(s.as_str(), "white" | "w" | "black" | "b") {
                eprintln!("Invalid side. Use white/w or black/b.");
                std::process::exit(1);
            }
            i += 1;
        } else if args[i] == "-m" || args[i] == "--smp" {
            use_smp = true;
        } else if (args[i] == "-t" || args[i] == "--threads") && args.get(i + 1).is_some() {
            num_threads = args[i + 1].parse().unwrap_or(2).max(1);
            i += 1;
        }
        i += 1;
    }

    let mut engine = Engine::new();
    engine.set_ponder_time(ponder_ms);

    let side = if user_white { "White" } else { "Black" };
    println!("You play as {}.", side);
    println!("Ponder time: {} ms", ponder_ms);
    if use_smp {
        println!("Engine: lazy SMP ({} threads)", num_threads);
    } else {
        println!("Engine: single-thread");
    }
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        println!("{}", engine);
        let user_turn = (engine.player() == Player::White) == user_white;

        if engine.game_state() != GameState::Playing {
            match engine.game_state() {
                GameState::WhiteWin => println!("Checkmate. White wins."),
                GameState::BlackWin => println!("Checkmate. Black wins."),
                GameState::Draw => println!("Draw."),
                _ => {}
            }
            let moves = engine.board().move_history();
            if !moves.is_empty() {
                println!("\nPGN:\n{}", pgn(moves));
            }
            break;
        }

        if user_turn {
            print!("Your move: ");
            stdout.flush().unwrap();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.eq_ignore_ascii_case("resign") {
                let winner = if user_white { "Black" } else { "White" };
                println!("You resign. {} wins.", winner);
                let moves = engine.board().move_history();
                if !moves.is_empty() {
                    println!("\nPGN:\n{}", pgn(moves));
                }
                std::process::exit(0);
            }
            if line.eq_ignore_ascii_case("undo") {
                let n = engine.board().move_history().len();
                if n == 0 {
                    println!("Nothing to undo.");
                    continue;
                }
                let to_undo = if n >= 2 { 2 } else { 1 };
                for _ in 0..to_undo {
                    engine.undo();
                }
                engine.update_game_state();
                continue;
            }
            let mv = match line.parse::<Move>() {
                Ok(m) => m,
                Err(_) => {
                    println!("Invalid move format. Use UCI (e.g. e2e4, e7e8q).");
                    continue;
                }
            };
            if !engine.make_move(mv) {
                println!("Illegal move.");
                continue;
            }
        } else {
            stdout.flush().unwrap();
            let best = if use_smp {
                engine.best_move_smp(num_threads)
            } else {
                engine.best_move()
            };
            engine.make_move(best);
            println!("{}\n", best);
        }
    }
}
