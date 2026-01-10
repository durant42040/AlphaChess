use std::fmt;

use crate::bitboard::Bitboard;
use crate::chessboard::ChessBoard;
use crate::game::{GameState, Player};
use crate::r#move::Move;
use crate::move_generator::MoveGenerator;
use crate::square::Square;

pub struct Engine {
    board: ChessBoard,
    move_generator: MoveGenerator,
    game_state: GameState,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: ChessBoard::new(),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
        }
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
        self.game_state = GameState::Playing;
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
        let our_pieces = if pieces.white_pieces.get_square(from) {
            pieces.white_pieces
        } else {
            pieces.black_pieces
        };

        let mut moves = Bitboard::default();

        if pieces.pawns.get_square(from) {
            if pieces.white_pieces.get_square(from) {
                moves = self.move_generator.generate_white_pawn_moves(
                    from,
                    pieces.all_pieces,
                    pieces.black_pieces | pieces.en_passant,
                );
            } else {
                moves = self.move_generator.generate_black_pawn_moves(
                    from,
                    pieces.all_pieces,
                    pieces.white_pieces | pieces.en_passant,
                );
            }
        } else if pieces.knights.get_square(from) {
            moves = self.move_generator.generate_knight_moves(from);
        } else if pieces.bishops.get_square(from) {
            moves = self
                .move_generator
                .generate_bishop_moves(from, pieces.all_pieces);
        } else if pieces.rooks.get_square(from) {
            moves = self
                .move_generator
                .generate_rook_moves(from, pieces.all_pieces);
        } else if pieces.queens.get_square(from) {
            moves = self
                .move_generator
                .generate_queen_moves(from, pieces.all_pieces);
        } else if pieces.kings.get_square(from) {
            moves = self.move_generator.generate_king_moves(from);
        }

        moves &= !our_pieces;

        moves
    }

    pub fn generate_legal_moves(&self, from: Square) -> Bitboard {
        let moves = self.generate_moves(from);
        todo!();
        moves
    }

    pub fn get_game_state(&self) -> String {
        self.game_state.to_string()
    }

    pub fn is_check(&self) -> bool {
        self.board.is_check()
    }

    pub fn is_legal_move(&self, r#move: Move) -> bool {
        let from = r#move.from;
        let to = r#move.to;
        todo!();
        self.generate_legal_moves(from).get_square(to)
            && self.board.get_our_pieces().get_square(from)
    }

    pub fn update_game_state(&mut self) {
        let mut moves = Bitboard::default();
        for from in self.board.get_our_pieces().iter() {
            moves |= self.generate_moves(Square::from(from));
        }
        if moves.empty() {
            if self.is_check() {
                if self.board.get_player() == Player::White {
                    self.game_state = GameState::BlackWin;
                } else {
                    self.game_state = GameState::WhiteWin;
                }
            } else {
                self.game_state = GameState::Draw;
            }
        }
        todo!();
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}
