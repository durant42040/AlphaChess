pub mod perft;

pub use perft::Perft;

use crate::Engine;

pub struct Search {
    engine: Engine,
}

impl Search {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }

    pub fn best_move(&mut self) -> Option<(crate::chess::r#move::Move, i32)> {
        if self.engine.get_game_state() != "playing" {
            return None;
        }

        let moves = self.engine.generate_all_legal_moves();
        let mut best_score = i32::MIN;
        let mut best_move = None;

        for r#move in moves {
            self.engine.board_mut().act(r#move);
            let score = self.engine.eval();
            self.engine.board_mut().undo();

            if score > best_score {
                best_score = score;
                best_move = Some(r#move);
            }
        }

        Some((best_move.unwrap(), best_score))
    }
}
