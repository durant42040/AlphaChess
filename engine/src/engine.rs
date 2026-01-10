use std::fmt;

use crate::chessboard::{ChessBoard, GameState};
use crate::r#move::Move;

pub struct Engine {
    board: ChessBoard,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: ChessBoard::new(),
        }
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
    }

    pub fn act(&mut self, move_string: String) -> bool {
        let r#move = move_string.parse::<Move>().unwrap();

        if !self.is_legal_move(r#move.clone()) {
            return false;
        }

        self.board.act(r#move);
        true
    }

    pub fn get_game_state(&self) -> String {
        self.board.get_game_state()
    }

    pub fn is_check(&self) -> bool {
        false
    }

    pub fn is_legal_move(&self, r#move: Move) -> bool {
        true
    }

    pub fn get_legal_moves(&self) -> Vec<String> {
        vec!["e2e4".to_string(), "e2e3".to_string()]
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}
