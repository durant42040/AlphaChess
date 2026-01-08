use crate::r#move::Move;
use crate::chessboard::ChessBoard;

pub struct Engine {
    board: ChessBoard,
    moves: Vec<Move>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: ChessBoard::new(),
            moves: vec![],
        }
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
        self.moves.clear();
    }

    pub fn act(&mut self, move_string: String) -> bool {
        true
    }
    
    pub fn get_game_state(&self) -> String {
        "playing".to_string()
    }
    
    pub fn is_check(&self) -> bool {
        false
    }
    
    pub fn is_legal_move(&self, move_string: String) -> bool {
        true
    }
    
    pub fn get_legal_moves(&self) -> Vec<String> {
        vec!["e2e4".to_string(), "e2e3".to_string()]
    }
    
    pub fn get_board(&self) -> String {
        "RNBQKBNRPPPPPPPP................................pppppppprnbqkbnr".to_string()
    }

    pub fn get_moves(&self) -> Vec<String> {
        self.moves.iter().map(|r#move| r#move.to_string()).collect()
    }
}

