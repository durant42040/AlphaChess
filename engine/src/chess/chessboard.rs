use std::fmt;

use crate::chess::bitboard::Bitboard;
use crate::chess::castling::CastlingRights;
use crate::chess::constants::*;
use crate::chess::game::Player;
use crate::chess::r#move::Move;
use crate::chess::pieces::{Color, Piece, Pieces};
use crate::chess::square::Square;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub captured_piece: Option<(Piece, Color)>,
    pub prev_en_passant: Bitboard,
    pub prev_castling_rights: CastlingRights,
    pub prev_fifty_move_rule: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChessBoard {
    fifty_move_rule: u8,
    castling_rights: CastlingRights,
    player: Player,
    pieces: Pieces,
    state_history: Vec<State>,
}

impl ChessBoard {
    pub fn new() -> Self {
        let pieces = Pieces::new();
        let state_history = Vec::with_capacity(100);
        let player = Player::White;
        let castling_rights = CastlingRights::new();

        Self {
            fifty_move_rule: 0,
            castling_rights,
            player,
            pieces,
            state_history,
        }
    }

    pub fn load_from_fen(fen: &str) -> Self {
        let mut chessboard = Self::default();
        let mut parts = fen.split_whitespace();

        let board = parts.next().unwrap_or("");
        let mut rank = 7;
        let mut file = 0;

        for c in board.chars() {
            let i: u8 = rank * 8 + file;

            if c == '/' {
                rank -= 1;
                file = 0;
                continue;
            }

            if c.is_ascii_digit() {
                file += c.to_digit(10).unwrap() as u8;
                continue;
            }

            if let Some((piece, color)) = Piece::from_char(c) {
                chessboard.pieces.set(piece, color, i);
            }

            file += 1;
        }

        let player_str = parts.next().unwrap_or("w");
        if player_str == "b" {
            chessboard.player = Player::Black;
        }

        let castling_str = parts.next().unwrap_or("-");
        chessboard.castling_rights = CastlingRights::default();
        for c in castling_str.chars() {
            match c {
                'K' => chessboard.castling_rights.set(WHITE_CASTLE_KINGSIDE),
                'Q' => chessboard.castling_rights.set(WHITE_CASTLE_QUEENSIDE),
                'k' => chessboard.castling_rights.set(BLACK_CASTLE_KINGSIDE),
                'q' => chessboard.castling_rights.set(BLACK_CASTLE_QUEENSIDE),
                _ => {}
            }
        }

        let en_passant_str = parts.next().unwrap_or("-");
        if en_passant_str != "-" {
            let square = en_passant_str.parse::<Square>().unwrap();
            chessboard.pieces.en_passant.set_square(square);
        }

        chessboard.fifty_move_rule = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

        chessboard
    }

    pub fn act(&mut self, r#move: Move) {
        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;

        // king must not be captured
        debug_assert!(!self.pieces.kings.get_square(to));

        self.state_history.push(State {
            captured_piece: self.pieces.get_piece(to.square),
            prev_en_passant: self.pieces.en_passant,
            prev_castling_rights: self.castling_rights,
            prev_fifty_move_rule: self.fifty_move_rule,
        });

        self.fifty_move_rule += 1;
        if self.pieces.pawns.get_square(from) || self.pieces.all_pieces.get_square(to) {
            self.fifty_move_rule = 0;
        }

        self.pieces.promote(promotion, from);
        self.pieces.update_en_passant(from, to);
        self.castle(from, to);

        self.pieces.update(from, to);
        self.switch_player();
    }

    pub fn undo(&mut self, r#move: Move) {
        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;
        if promotion.is_some() {
            self.pieces.undo_promote(to);
        }

        self.pieces.update(to, from);
        if let Some((piece, color)) = self.state_history.last().unwrap().captured_piece {
            self.pieces.set(piece, color, to.square);
        }
        self.undo_castle(from, to);
        let state = self.state_history.last().unwrap();

        if state.prev_en_passant.get_square(to) && self.pieces.pawns.get_square(from) {
            let captured_square = Square::new(from.rank, to.file);
            self.pieces
                .set(Piece::Pawn, self.player.to_color(), captured_square.square);
        }

        self.pieces.en_passant = state.prev_en_passant;
        self.castling_rights = state.prev_castling_rights;
        self.fifty_move_rule = state.prev_fifty_move_rule;
        self.switch_player();
        self.state_history.pop();
    }

    pub fn get_pieces(&self) -> Pieces {
        self.pieces
    }

    pub fn get_our_pieces(&self) -> Bitboard {
        if self.player == Player::White {
            self.pieces.white_pieces
        } else {
            self.pieces.black_pieces
        }
    }

    pub fn get_their_pieces(&self) -> Bitboard {
        if self.player == Player::White {
            self.pieces.black_pieces
        } else {
            self.pieces.white_pieces
        }
    }

    pub fn get_player(&self) -> Player {
        self.player
    }

    pub fn switch_player(&mut self) {
        self.player = self.player.switch();
    }

    pub fn is_draw(&self) -> bool {
        !self.pieces.has_mating_material()
            || self.fifty_move_rule == 100
            || self.get_repetition_count() >= 2
    }

    pub fn get_fifty_move_rule(&self) -> u8 {
        self.fifty_move_rule
    }

    pub fn get_repetition_count(&self) -> u8 {
        0
    }
}

pub trait Castling {
    fn get_castling_rights(&self) -> CastlingRights;
    fn castle(&mut self, from: Square, to: Square);
    fn undo_castle(&mut self, from: Square, to: Square);
}

impl Castling for ChessBoard {
    fn get_castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    fn castle(&mut self, from: Square, to: Square) {
        // Remove castling rights when king or rook moves or is captured.
        self.castling_rights.revoke_castling_rights(from, to);

        // Move rook if this is a castling move (king moves two squares).
        if self.pieces.kings.get_square(from) && (from.square as i8 - to.square as i8).abs() == 2 {
            if from == WHITE_KING_START {
                if to == WHITE_QUEENSIDE_CASTLE_TO {
                    self.pieces.update(
                        WHITE_QUEENSIDE_ROOK_START.into(),
                        WHITE_QUEENSIDE_ROOK_CASTLE_TO.into(),
                    );
                } else if to == WHITE_KINGSIDE_CASTLE_TO {
                    self.pieces.update(
                        WHITE_KINGSIDE_ROOK_START.into(),
                        WHITE_KINGSIDE_ROOK_CASTLE_TO.into(),
                    );
                }
            } else if from == BLACK_KING_START {
                if to == BLACK_QUEENSIDE_CASTLE_TO {
                    self.pieces.update(
                        BLACK_QUEENSIDE_ROOK_START.into(),
                        BLACK_QUEENSIDE_ROOK_CASTLE_TO.into(),
                    );
                } else if to == BLACK_KINGSIDE_CASTLE_TO {
                    self.pieces.update(
                        BLACK_KINGSIDE_ROOK_START.into(),
                        BLACK_KINGSIDE_ROOK_CASTLE_TO.into(),
                    );
                }
            }
        }
    }

    fn undo_castle(&mut self, from: Square, to: Square) {
        if self.pieces.kings.get_square(from) && (from.square as i8 - to.square as i8).abs() == 2 {
            if from == WHITE_KING_START {
                if to == WHITE_QUEENSIDE_CASTLE_TO {
                    self.pieces.update(
                        WHITE_QUEENSIDE_ROOK_CASTLE_TO.into(),
                        WHITE_QUEENSIDE_ROOK_START.into(),
                    );
                } else if to == WHITE_KINGSIDE_CASTLE_TO {
                    self.pieces.update(
                        WHITE_KINGSIDE_ROOK_CASTLE_TO.into(),
                        WHITE_KINGSIDE_ROOK_START.into(),
                    );
                }
            } else if from == BLACK_KING_START {
                if to == BLACK_QUEENSIDE_CASTLE_TO {
                    self.pieces.update(
                        BLACK_QUEENSIDE_ROOK_CASTLE_TO.into(),
                        BLACK_QUEENSIDE_ROOK_START.into(),
                    );
                } else if to == BLACK_KINGSIDE_CASTLE_TO {
                    self.pieces.update(
                        BLACK_KINGSIDE_ROOK_CASTLE_TO.into(),
                        BLACK_KINGSIDE_ROOK_START.into(),
                    );
                }
            }
        }
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board = String::new();

        for i in 0..64 {
            let c = self.pieces.get_char(i);

            board.push(c);

            if i % 8 == 7 {
                board.push('\n');
            } else {
                board.push(' ');
            }
        }

        for rank in (0..8).rev() {
            write!(f, "{}  ", rank + 1)?;
            let start = rank * 16;
            let end = start + 16;
            write!(f, "{}", &board[start..end])?;
        }

        write!(f, "   a b c d e f g h\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::ChessBoard;

    #[test]
    fn test_chessboard_to_string() {
        let board: ChessBoard = ChessBoard::new();
        let board_string = format!("{}", board);

        let expected = r#"8  r n b q k b n r
7  p p p p p p p p
6  . . . . . . . .
5  . . . . . . . .
4  . . . . . . . .
3  . . . . . . . .
2  P P P P P P P P
1  R N B Q K B N R
   a b c d e f g h

"#;

        assert_eq!(board_string, expected);
    }

    #[test]
    fn test_load_from_fen() {
        let board_from_fen =
            ChessBoard::load_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let new_board = ChessBoard::new();

        assert_eq!(board_from_fen, new_board);
    }
}
