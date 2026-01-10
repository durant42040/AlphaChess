use crate::chessboard::ChessBoard;
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
        "playing".to_string()
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

    pub fn get_board(&self) -> String {
        "RNBQKBNRPPPPPPPP................................pppppppprnbqkbnr".to_string()
    }
}
