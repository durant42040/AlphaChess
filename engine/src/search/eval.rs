use crate::{Engine, GameState};

pub trait Evaluation {
    fn eval(&self) -> i32;
}

impl Evaluation for Engine {
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

        self.board.score()
    }
}
