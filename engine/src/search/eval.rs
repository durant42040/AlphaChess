use crate::{Engine, GameState, chess::Square};

pub trait Evaluation {
    fn mobility_score(&self) -> i32;
    fn eval(&self) -> i32;
}

impl Evaluation for Engine {
    fn mobility_score(&self) -> i32 {
        let mut score = 0;
        let our_mobile_pieces =
            self.board.our_pieces() & !self.pieces().kings() & !self.pieces().pawns();
        for from in our_mobile_pieces.iter() {
            let moves = self.generate_moves(Square::from(from));
            score += moves.count() as i32;
        }
        let their_mobile_pieces =
            self.board.their_pieces() & !self.pieces().kings() & !self.pieces().pawns();
        for from in their_mobile_pieces.iter() {
            let moves = self.generate_moves(Square::from(from));
            score -= moves.count() as i32;
        }
        score
    }

    /// Evaluate the position
    fn eval(&self) -> i32 {
        if self.game_state == GameState::Draw {
            return 0;
        }
        if self.game_state == GameState::WhiteWin {
            return i32::MAX;
        }
        if self.game_state == GameState::BlackWin {
            return i32::MIN;
        }

        self.board.score() + self.mobility_score()
    }
}
