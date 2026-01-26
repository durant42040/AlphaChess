pub mod perft;

pub use perft::Perft;

use crate::{Engine, chess::Move, engine::Evaluation};

pub trait Search {
    /// Find the best sequence of moves up to the given depth
    fn search(&mut self, depth: u8) -> Option<(Vec<Move>, i32)>;
}

impl Search for Engine {
    /// find a sequence of moves, evaluate score, return best sequence of moves
    fn search(&mut self, depth: u8) -> Option<(Vec<Move>, i32)> {
        if self.get_game_state() != "playing" {
            return None;
        }

        let mut searcher = State::new();
        searcher.dfs(self, depth);

        if searcher.best_moves.is_empty() {
            None
        } else {
            assert_eq!(searcher.best_moves.len(), depth as usize);
            Some((searcher.best_moves, searcher.best_score))
        }
    }
}

/// Helper struct to hold search state
struct State {
    curr_moves: Vec<Move>,
    best_moves: Vec<Move>,
    best_score: i32,
}

impl State {
    fn new() -> Self {
        Self {
            curr_moves: Vec::new(),
            best_moves: Vec::new(),
            best_score: i32::MIN,
        }
    }

    /// depth-first search
    fn dfs(&mut self, engine: &mut Engine, depth: u8) {
        if depth == 0 {
            let score = engine.eval();
            if score > self.best_score {
                self.best_score = score;
                self.best_moves = self.curr_moves.clone();
            }
            return;
        }

        let moves = engine.generate_all_legal_moves();
        for r#move in moves {
            engine.board_mut().act(r#move);
            engine.update_game_state();
            self.curr_moves.push(r#move);
            self.dfs(engine, depth - 1);
            self.curr_moves.pop();
            engine.undo();
        }
    }
}

#[test]
fn test_search() {
    let mut engine = Engine::new();
    if let Some((best_moves, best_score)) = engine.search(5) {
        for r#move in best_moves {
            println!("{}", r#move);
        }
        println!("{}", best_score);
    }
}
