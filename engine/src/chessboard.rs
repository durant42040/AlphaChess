use std::fmt;

use crate::bitboard::Bitboard;
use crate::game::Player;
use crate::r#move::Move;
use crate::pieces::{Piece, Pieces};
use crate::square::Square;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateInfo {
    pub captured_piece: Option<(Piece, bool)>,
    pub prev_en_passant: Bitboard,
    pub prev_castling_rights: u8,
    pub prev_fifty_move_rule: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChessBoard {
    fifty_move_rule: u8,
    castling_rights: u8,
    player: Player,
    pieces: Pieces,
    state_info: StateInfo,
}

impl ChessBoard {
    pub fn new() -> Self {
        let pieces = Pieces::new();
        let state_info = StateInfo::default();
        let player = Player::White;

        Self {
            fifty_move_rule: 0,
            castling_rights: 0b1111,
            player,
            pieces,
            state_info,
        }
    }

    pub fn load_from_fen(fen: String) -> Self {
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

            if let Some((piece, is_white)) = Piece::from_char(c) {
                chessboard.pieces.set(piece, is_white, i);
            }

            file += 1;
        }

        let player_str = parts.next().unwrap_or("w");
        if player_str == "b" {
            chessboard.player = Player::Black;
        }

        let castling_str = parts.next().unwrap_or("-");
        chessboard.castling_rights = 0u8;
        for c in castling_str.chars() {
            match c {
                'K' => chessboard.castling_rights |= 1,
                'Q' => chessboard.castling_rights |= 2,
                'k' => chessboard.castling_rights |= 4,
                'q' => chessboard.castling_rights |= 8,
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

        self.state_info.captured_piece = self.pieces.get_piece(to.square);
        self.state_info.prev_en_passant = self.pieces.en_passant;
        self.state_info.prev_castling_rights = self.castling_rights;
        self.state_info.prev_fifty_move_rule = self.fifty_move_rule;

        self.fifty_move_rule += 1;
        if self.pieces.pawns.get_square(from) || self.pieces.all_pieces.get_square(to) {
            self.fifty_move_rule = 0;
        }

        self.pieces.promote(promotion, from);
        self.pieces.update_en_passant(from, to);
        self.castle(from, to);

        self.pieces.update(from, to);
        self.player = self.player.switch();
    }

    pub fn undo(&mut self, r#move: Move) {
        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;
        if promotion.is_some() {
            self.pieces.undo_promote(to);
        }

        self.pieces.update(to, from);
        if let Some((piece, is_white)) = self.state_info.captured_piece {
            self.pieces.set(piece, is_white, to.square);
        }
        self.undo_castle(from, to);

        if self.state_info.prev_en_passant.get_square(to) && self.pieces.pawns.get_square(from) {
            let captured_square = Square::new(from.rank, to.file);
            self.pieces.set(
                Piece::Pawn,
                self.player == Player::White,
                captured_square.square,
            );
        }

        self.pieces.en_passant = self.state_info.prev_en_passant;
        self.fifty_move_rule = self.state_info.prev_fifty_move_rule;
        self.player = self.player.switch();
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

    pub fn get_player(&self) -> Player {
        self.player
    }

    pub fn get_castling_rights(&self) -> u8 {
        self.castling_rights
    }

    pub fn castle(&mut self, from: Square, to: Square) {
        // remove castling rights if king or rook is moved or captured
        if from == 0 || to == 0 {
            self.castling_rights &= !2;
        } else if from == 7 || to == 7 {
            self.castling_rights &= !1;
        } else if from == 4 || to == 4 {
            self.castling_rights &= !3;
        } else if from == 56 || to == 56 {
            self.castling_rights &= !8;
        } else if from == 60 || to == 60 {
            self.castling_rights &= !12;
        } else if from == 63 || to == 63 {
            self.castling_rights &= !4;
        }
        // move rook if castling
        if self.pieces.kings.get_square(from) && (from.square as i8 - to.square as i8).abs() == 2 {
            if from == 4 {
                if to == 2 {
                    self.pieces.rooks.update(0, 3);
                    self.pieces.white_pieces.update(0, 3);
                    self.pieces.all_pieces.update(0, 3);
                } else if to == 6 {
                    self.pieces.rooks.update(7, 5);
                    self.pieces.white_pieces.update(7, 5);
                    self.pieces.all_pieces.update(7, 5);
                }
            } else if from == 60 {
                if to == 58 {
                    self.pieces.rooks.update(56, 59);
                    self.pieces.black_pieces.update(56, 59);
                    self.pieces.all_pieces.update(56, 59);
                } else if to == 62 {
                    self.pieces.rooks.update(63, 61);
                    self.pieces.black_pieces.update(63, 61);
                    self.pieces.all_pieces.update(63, 61);
                }
            }
        }
    }

    fn undo_castle(&mut self, from: Square, to: Square) {
        self.castling_rights = self.state_info.prev_castling_rights;
        if self.pieces.kings.get_square(from) && (from.square as i8 - to.square as i8).abs() == 2 {
            if from == 4 {
                if to == 2 {
                    self.pieces.rooks.update(3, 0);
                    self.pieces.white_pieces.update(3, 0);
                    self.pieces.all_pieces.update(3, 0);
                } else if to == 6 {
                    self.pieces.rooks.update(5, 7);
                    self.pieces.white_pieces.update(5, 7);
                    self.pieces.all_pieces.update(5, 7);
                }
            } else if from == 60 {
                if to == 58 {
                    self.pieces.rooks.update(59, 56);
                    self.pieces.black_pieces.update(59, 56);
                    self.pieces.all_pieces.update(59, 56);
                } else if to == 62 {
                    self.pieces.rooks.update(61, 63);
                    self.pieces.black_pieces.update(61, 63);
                    self.pieces.all_pieces.update(61, 63);
                }
            }
        }
    }

    pub fn is_draw(&self) -> bool {
        !self.pieces.has_mating_material()
            || self.fifty_move_rule == 100
            || self.get_repetition_count() >= 2
    }

    pub fn get_repetition_count(&self) -> u8 {
        0
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
        let board_from_fen = ChessBoard::load_from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        );
        let new_board = ChessBoard::new();

        assert_eq!(board_from_fen, new_board);
    }
}
