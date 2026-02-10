use std::fmt;

use crate::chess::castling::CastlingRights;
use crate::chess::pieces::{Color, Piece, Pieces};
use crate::chess::zobrist::Zobrist;
use crate::chess::{Bitboard, Move, Player, Square};
use crate::constants::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub captured_piece: Option<(Piece, Color)>,
    pub prev_en_passant: Bitboard,
    pub prev_castling_rights: CastlingRights,
    pub prev_fifty_move_rule: u8,
}

impl State {
    pub fn new(
        captured_piece: Option<(Piece, Color)>,
        prev_en_passant: Bitboard,
        prev_castling_rights: CastlingRights,
        prev_fifty_move_rule: u8,
    ) -> Self {
        Self {
            captured_piece,
            prev_en_passant,
            prev_castling_rights,
            prev_fifty_move_rule,
        }
    }
}

#[derive(Default)]
pub struct ChessBoard {
    fifty_move_rule: u8,
    castling_rights: CastlingRights,
    player: Player,
    pieces: Pieces,
    hasher: Zobrist,
    move_history: Vec<Move>,
    state_history: Vec<State>,
    position_history: Vec<u64>,
    material_score: i32,
    total_material: i32,
}

impl ChessBoard {
    pub fn new() -> Self {
        let pieces = Pieces::new();
        let state_history = Vec::with_capacity(8192);
        let position_history = Vec::with_capacity(8192);
        let move_history = Vec::with_capacity(8192);
        let player = Player::White;
        let castling_rights = CastlingRights::new();
        let hasher = Zobrist::new();

        let mut chessboard = Self {
            move_history,
            fifty_move_rule: 0,
            castling_rights,
            player,
            pieces,
            hasher,
            state_history,
            position_history,
            material_score: 0,
            total_material: 8000,
        };

        chessboard
            .position_history
            .push(chessboard.hasher.full_hash(&chessboard));

        chessboard
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
            chessboard.pieces.set_en_passant_square(square);
        }

        chessboard.fifty_move_rule = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
        chessboard.material_score = chessboard.compute_material_score();

        chessboard
            .position_history
            .push(chessboard.hasher.full_hash(&chessboard));

        debug_assert!(chessboard.pieces.kings().count() == 2);

        chessboard
    }

    fn compute_material_score(&self) -> i32 {
        let pieces = self.pieces;

        let white = pieces.white_pieces();
        let black = pieces.black_pieces();

        let white_score = (pieces.pawns() & white).count() as i32 * Piece::Pawn.value()
            + (pieces.knights() & white).count() as i32 * Piece::Knight.value()
            + (pieces.bishops() & white).count() as i32 * Piece::Bishop.value()
            + (pieces.rooks() & white).count() as i32 * Piece::Rook.value()
            + (pieces.queens() & white).count() as i32 * Piece::Queen.value();

        let black_score = (pieces.pawns() & black).count() as i32 * Piece::Pawn.value()
            + (pieces.knights() & black).count() as i32 * Piece::Knight.value()
            + (pieces.bishops() & black).count() as i32 * Piece::Bishop.value()
            + (pieces.rooks() & black).count() as i32 * Piece::Rook.value()
            + (pieces.queens() & black).count() as i32 * Piece::Queen.value();

        white_score - black_score
    }

    pub fn act(&mut self, r#move: Move) {
        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;
        let moving_piece = self.pieces.piece(from);
        let captured_piece = self.pieces.piece(to);

        assert!(moving_piece.is_some());
        // king must not be captured
        assert!(!self.pieces.kings().get_square(to));

        if let Some((piece, color)) = captured_piece {
            if color == Color::White {
                self.material_score -= piece.value();
            } else {
                self.material_score += piece.value();
            }
            self.total_material -= piece.value();
        }

        let prev_state = State::new(
            captured_piece,
            self.pieces.en_passant(),
            self.castling_rights,
            self.fifty_move_rule,
        );
        self.state_history.push(prev_state);

        self.fifty_move_rule += 1;
        if self.pieces.pawns().get_square(from) || self.pieces.all_pieces().get_square(to) {
            self.fifty_move_rule = 0;
        }

        if let Some(promotion) = promotion {
            if self.player == Player::White {
                self.material_score += promotion.value() - Piece::Pawn.value();
            } else {
                self.material_score -= promotion.value() - Piece::Pawn.value();
            }
            self.total_material += promotion.value() - Piece::Pawn.value();
            self.pieces.promote(promotion, from);
        }
        self.update_en_passant(from, to);
        self.castle(from, to);
        self.pieces.update(from, to);
        self.player = !self.player;
        self.move_history.push(r#move);
        let prev_hash = self.position_history.last().unwrap();
        let new_hash = self.hasher.hash(
            r#move,
            moving_piece.unwrap(),
            self.castling_rights,
            self.pieces.en_passant(),
            prev_state,
            *prev_hash,
        );
        self.position_history.push(new_hash);
        debug_assert_eq!(self.material_score, self.compute_material_score());
    }

    pub fn make_null_move(&mut self) -> Bitboard {
        let prev_en_passant = self.pieces.en_passant();
        self.pieces.set_en_passant(Bitboard::zero());
        self.player = !self.player;

        self.fifty_move_rule += 1;

        let prev_hash = self.position_history.last().unwrap();
        let new_hash = self
            .hasher
            .hash_null_move(self.pieces.en_passant(), *prev_hash);
        self.position_history.push(new_hash);

        prev_en_passant
    }

    pub fn undo_null_move(&mut self, en_passant: Bitboard) {
        self.pieces.set_en_passant(en_passant);
        self.player = !self.player;

        self.fifty_move_rule -= 1;

        self.position_history.pop();
    }

    pub fn undo(&mut self) {
        debug_assert!(!self.move_history.is_empty());
        let r#move = self.move_history.pop().unwrap();
        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;
        if let Some(promotion) = promotion {
            if self.player == Player::White {
                self.material_score += promotion.value() - Piece::Pawn.value();
            } else {
                self.material_score -= promotion.value() - Piece::Pawn.value();
            }
            self.total_material -= promotion.value() - Piece::Pawn.value();
            self.pieces.undo_promote(to);
        }

        self.pieces.update(to, from);
        self.undo_castle(from, to);
        let state = self.state_history.last().unwrap();

        if let Some((piece, color)) = state.captured_piece {
            self.pieces.set(piece, color, to.square);
            if color == Color::White {
                self.material_score += piece.value();
            } else {
                self.material_score -= piece.value();
            }
            self.total_material += piece.value();
        }

        if state.prev_en_passant.get_square(to) && self.pieces.pawns().get_square(from) {
            let to = Square::new(from.rank, to.file);
            let color = self.player.color();
            self.pieces.set(Piece::Pawn, color, to.square);
            if color == Color::White {
                self.material_score += Piece::Pawn.value();
            } else {
                self.material_score -= Piece::Pawn.value();
            }
            self.total_material += Piece::Pawn.value();
        }

        self.pieces.set_en_passant(state.prev_en_passant);
        self.castling_rights = state.prev_castling_rights;
        self.fifty_move_rule = state.prev_fifty_move_rule;
        self.player = !self.player;
        self.state_history.pop();
        self.position_history.pop();
        debug_assert_eq!(self.material_score, self.compute_material_score());
    }

    pub fn update_en_passant(&mut self, from: Square, to: Square) {
        if self.pieces.pawns().get_square(from) && self.pieces.en_passant().get_square(to) {
            let captured_square = Square::new(from.rank, to.file);
            let capturing_color = if self.pieces.white_pieces().get_square(from) {
                Color::White
            } else {
                Color::Black
            };
            self.pieces
                .clear(Piece::Pawn, !capturing_color, captured_square.square);
            if capturing_color == Color::White {
                self.material_score += Piece::Pawn.value();
            } else {
                self.material_score -= Piece::Pawn.value();
            }
            self.total_material -= Piece::Pawn.value();
        }

        self.pieces.set_en_passant(Bitboard::zero());
        if self.pieces.pawns().get_square(from) && (from.rank as i8 - to.rank as i8).abs() == 2 {
            self.pieces
                .set_en_passant_square(Square::from((from.square + to.square) / 2));
        }
    }

    pub fn pieces(&self) -> Pieces {
        self.pieces
    }

    pub fn our_pieces(&self) -> Bitboard {
        if self.player == Player::White {
            self.pieces.white_pieces()
        } else {
            self.pieces.black_pieces()
        }
    }

    pub fn their_pieces(&self) -> Bitboard {
        if self.player == Player::White {
            self.pieces.black_pieces()
        } else {
            self.pieces.white_pieces()
        }
    }

    #[inline(always)]
    pub fn pieces_of_color(&self, color: Color) -> Bitboard {
        if color == Color::White {
            self.pieces.white_pieces()
        } else {
            self.pieces.black_pieces()
        }
    }

    #[inline(always)]
    pub fn player(&self) -> Player {
        self.player
    }

    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        !self.pieces.has_mating_material()
            || self.fifty_move_rule >= 100
            || self.repetition_count() >= 2
    }

    #[inline(always)]
    pub fn fifty_move_rule(&self) -> u8 {
        self.fifty_move_rule
    }

    pub fn repetition_count(&self) -> u8 {
        if self.position_history.len() <= 2 || self.fifty_move_rule == 0 {
            return 0;
        }

        let hash = self.position_hash();
        let mut count = 0;

        let len = self.position_history.len();
        let fifty_move = self.fifty_move_rule as usize;

        let start = len.saturating_sub(1 + fifty_move);

        let mut i = len.saturating_sub(3);
        while i >= start {
            if self.position_history[i] == hash {
                count += 1;
            }
            if i < 2 {
                break;
            }
            i -= 2;
        }

        count
    }

    #[inline(always)]
    pub fn move_history(&self) -> &[Move] {
        &self.move_history
    }

    #[inline(always)]
    pub fn position_hash(&self) -> u64 {
        debug_assert!(!self.position_history.is_empty());
        self.position_history.last().copied().unwrap()
    }

    #[inline(always)]
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    #[inline(always)]
    pub fn material_score(&self) -> i32 {
        self.material_score
    }
}

pub trait Castling {
    fn castling_rights(&self) -> CastlingRights;
    fn castle(&mut self, from: Square, to: Square);
    fn undo_castle(&mut self, from: Square, to: Square);
}

impl Castling for ChessBoard {
    fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    fn castle(&mut self, from: Square, to: Square) {
        // Remove castling rights when king or rook moves or is captured.
        self.castling_rights.revoke_castling_rights(from, to);

        // Move rook if this is a castling move (king moves two squares).
        if self.pieces.kings().get_square(from) && (from.square as i8 - to.square as i8).abs() == 2
        {
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
        if self.pieces.kings().get_square(from) && (from.square as i8 - to.square as i8).abs() == 2
        {
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
        let mut board = String::from("\n");

        // Print ranks from 8 down to 1, files from a to h (left to right).
        for rank in (0..8).rev() {
            // Rank label at the start of each line.
            board.push_str(&format!("\x1b[90m{} \x1b[0m", rank + 1));

            for file in 0..8 {
                let i = rank * 8 + file;
                let c = self.pieces.get_char(i);
                let colored = match c {
                    'P' | 'N' | 'B' | 'R' | 'Q' | 'K' => format!("\x1b[97m{}\x1b[0m", c),
                    'p' | 'n' | 'b' | 'r' | 'q' | 'k' => format!("\x1b[96m{}\x1b[0m", c),
                    '.' => "\x1b[90m.\x1b[0m".to_string(),
                    _ => c.to_string(),
                };

                board.push_str(&colored);
                if file == 7 {
                    board.push('\n');
                } else {
                    board.push(' ');
                }
            }
        }
        board.push_str("\x1b[90m  a b c d e f g h\x1b[0m\n\n");
        write!(f, "{}", board)
    }
}

#[cfg(test)]
mod tests {
    use super::ChessBoard;
    use crate::chess::Move;

    #[test]
    fn zobrist_basic() {
        let mut board = ChessBoard::new();
        board.act(Move::from("e2e3"));
        board.act(Move::from("e7e6"));
        let incr_hash_1 = board.position_hash();
        let full_hash_1 = board.hasher.full_hash(&board);
        assert_eq!(
            incr_hash_1, full_hash_1,
            "incremental hash should equal full hash"
        );
        board.act(Move::from("g1f3"));
        board.act(Move::from("b8c6"));
        board.act(Move::from("f1d3"));
        board.act(Move::from("f8d6"));
        board.act(Move::from("f3g1"));
        board.act(Move::from("c6b8"));
        board.act(Move::from("d3f1"));
        board.act(Move::from("d6f8"));
        let incr_hash_2 = board.position_hash();
        let full_hash_2 = board.hasher.full_hash(&board);
        assert_eq!(
            incr_hash_1, incr_hash_2,
            "incremental hash should equal full hash"
        );
        assert_eq!(full_hash_1, full_hash_2);
    }

    #[test]
    fn zobrist_undo() {
        let mut board = ChessBoard::new();
        let hash_initial = board.position_hash();
        let r#move = Move::from("e2e4");
        board.act(r#move);
        let hash_after = board.position_hash();
        board.undo();
        let hash_restored = board.position_hash();
        assert_eq!(
            hash_initial, hash_restored,
            "undo should restore position hash"
        );
        assert_ne!(hash_initial, hash_after, "move should change hash");
    }

    #[test]
    fn test_chessboard_to_string() {
        let board = ChessBoard::new();
        let board_string = format!("{}", board);

        println!("{}", board_string);
    }

    #[test]
    fn test_load_from_fen() {
        let board_from_fen =
            ChessBoard::load_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let new_board = ChessBoard::new();

        assert_eq!(board_from_fen.pieces(), new_board.pieces());
    }

    #[test]
    fn test_draw_by_repetition() {
        let mut board = ChessBoard::new();
        let moves = [
            "g1f3", "b8c6", "f3g1", "c6b8", "g1f3", "b8c6", "f3g1", "c6b8",
        ];
        for r#move in &moves {
            board.act(Move::from(r#move));
        }
        assert!(
            board.repetition_count() >= 2,
            "initial position occurred three times"
        );
        assert!(board.is_draw(), "three-fold repetition is a draw");
    }
}
