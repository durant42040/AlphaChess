use std::fmt;

use crate::bitboard::Bitboard;
use crate::chessboard::ChessBoard;
use crate::r#move::Move;
use crate::move_generator::MoveGenerator;
use crate::square::Square;

pub struct Engine {
    board: ChessBoard,
    move_generator: MoveGenerator,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: ChessBoard::new(),
            move_generator: MoveGenerator::new(),
        }
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
    }

    pub fn act(&mut self, move_string: String) -> bool {
        let r#move = move_string.parse::<Move>().unwrap();
        if !self.is_legal_move(r#move) {
            return false;
        }
        self.board.act(r#move);
        true
    }

    pub fn generate_moves(&self, from: Square) -> Bitboard {
        let pieces = self.board.get_pieces();
        let moves = Bitboard::default();
        moves
    }

    pub fn get_game_state(&self) -> String {
        self.board.get_game_state()
    }

    pub fn is_check(&self) -> bool {
        false
    }

    pub fn get_legal_moves(&self) -> Vec<String> {
        vec!["e2e4".to_string(), "e2e3".to_string()]
    }

    pub fn is_legal_move(&self, r#move: Move) -> bool {
        true
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}
