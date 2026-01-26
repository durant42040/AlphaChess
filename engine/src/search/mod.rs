pub mod perft;

use std::cmp::max;

pub use perft::Perft;

use crate::engine::{Engine, Evaluation};

pub trait Search {
    /// Find the best sequence of moves up to the given depth
    fn max_search(&mut self, depth: u8) -> i32;
    fn minimax_search(&mut self, depth: u8) -> i32;
}

impl Search for Engine {
    fn max_search(&mut self, depth: u8) -> i32 {
        if depth == 0 {
            let score = self.eval();
            return score;
        }

        let mut score = i32::MIN;

        let moves = self.generate_all_legal_moves();
        for r#move in moves {
            self.board_mut().act(r#move);
            self.update_game_state();
            score = max(score, self.max_search(depth - 1));
            self.undo();
        }

        score
    }

    fn minimax_search(&mut self, depth: u8) -> i32 {
        if depth == 0 {
            let score = self.eval();
            return score;
        }

        let mut score = i32::MIN;

        let moves = self.generate_all_legal_moves();
        for r#move in moves {
            self.board_mut().act(r#move);
            self.update_game_state();
            score = max(score, -self.minimax_search(depth - 1));
            self.undo();
        }

        score
    }
}

#[test]
fn test_search() {
    let mut engine = Engine::new();
    let best_score = engine.max_search(5);
    println!("{}", best_score);
}
