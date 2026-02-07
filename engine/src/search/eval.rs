use crate::chess::{Bitboard, Piece};
use crate::constants::{
    BLACK_BISHOP_SCORE, BLACK_KING_SCORE, BLACK_PASSED_MASK, BLACK_PAWN_SCORE, BLACK_ROOK_SCORE, DOUBLE_PAWN_PENALTY, FILE_MASKS, ISOLATED_MASK, ISOLATED_PAWN_PENALTY, KNIGHT_SCORE, PASSED_PAWN_BONUS, WHITE_BISHOP_SCORE, WHITE_KING_SCORE, WHITE_PASSED_MASK, WHITE_PAWN_SCORE, WHITE_ROOK_SCORE
};
use crate::{
    Engine,
    chess::{Color, Player, Square},
};

pub trait Evaluation {
    fn material_score(&self) -> i32;
    fn mobility_score(&self) -> i32;
    fn positional_score(&self) -> i32;
    fn double_pawn_penalty(&self) -> i32;
    fn isolated_pawn_penalty(&self) -> i32;
    fn passed_pawn_bonus(&self) -> i32;
    fn eval(&self) -> i32;
}

impl Evaluation for Engine {
    fn material_score(&self) -> i32 {
        if self.board.player() == Player::White {
            self.board.material_score()
        } else {
            -self.board.material_score()
        }
    }

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

    fn positional_score(&self) -> i32 {
        let pieces = self.pieces();
        let all_pieces = self.pieces().all_pieces();
        let mut score = 0;

        for i in all_pieces.iter() {
            let (piece, color) = pieces.piece(Square::from(i)).unwrap();

            match color {
                Color::White => match piece {
                    Piece::Pawn => score += WHITE_PAWN_SCORE[i as usize],
                    Piece::Knight => score += KNIGHT_SCORE[i as usize],
                    Piece::Bishop => score += WHITE_BISHOP_SCORE[i as usize],
                    Piece::Rook => score += WHITE_ROOK_SCORE[i as usize],
                    Piece::King => score += WHITE_KING_SCORE[i as usize],
                    Piece::Queen => {}
                },
                Color::Black => match piece {
                    Piece::Pawn => score -= BLACK_PAWN_SCORE[i as usize],
                    Piece::Knight => score -= KNIGHT_SCORE[i as usize],
                    Piece::Bishop => score -= BLACK_BISHOP_SCORE[i as usize],
                    Piece::Rook => score -= BLACK_ROOK_SCORE[i as usize],
                    Piece::King => score -= BLACK_KING_SCORE[i as usize],
                    Piece::Queen => {}
                },
            }
        }

        if self.board.player() == Player::White {
            score
        } else {
            -score
        }
    }

    fn double_pawn_penalty(&self) -> i32 {
        let mut score = 0;
        for i in 0..8 {
            let white_pawns = self.pieces().white_pieces()
                & self.pieces().pawns()
                & Bitboard::from(FILE_MASKS[i]);
            let white_pawns_count = white_pawns.count();
            if white_pawns_count > 1 {
                score += DOUBLE_PAWN_PENALTY * (white_pawns_count - 1) as i32;
            }

            let black_pawns = self.pieces().black_pieces()
                & self.pieces().pawns()
                & Bitboard::from(FILE_MASKS[i]);
            let black_pawns_count = black_pawns.count();
            if black_pawns_count > 1 {
                score -= DOUBLE_PAWN_PENALTY * (black_pawns_count - 1) as i32;
            }
        }
        if self.board.player() == Player::White {
            score
        } else {
            -score
        }
    }

    fn passed_pawn_bonus(&self) -> i32 {
        let white_pawns = self.pieces().white_pieces() & self.pieces().pawns();
        let black_pawns = self.pieces().black_pieces() & self.pieces().pawns();
        let mut score = 0;
        for square in white_pawns.iter() {
            if !black_pawns.intersects(Bitboard::from(WHITE_PASSED_MASK[square as usize])) {
                score += PASSED_PAWN_BONUS[(square / 8) as usize];
            }
        }
        for square in black_pawns.iter() {
            if !white_pawns.intersects(Bitboard::from(BLACK_PASSED_MASK[square as usize])) {
                score -= PASSED_PAWN_BONUS[7 - (square / 8) as usize];
            }
        }
        if self.board.player() == Player::White {
            score
        } else {
            -score
        }
    }

    fn isolated_pawn_penalty(&self) -> i32 {
        let white_pawns = self.pieces().white_pieces() & self.pieces().pawns();
        let black_pawns = self.pieces().black_pieces() & self.pieces().pawns();
        let mut score = 0;
        for square in white_pawns.iter() {
            if !white_pawns.intersects(Bitboard::from(ISOLATED_MASK[square as usize])) {
                score += ISOLATED_PAWN_PENALTY;
            }
        }
        for square in black_pawns.iter() {
            if !black_pawns.intersects(Bitboard::from(ISOLATED_MASK[square as usize])) {
                score -= ISOLATED_PAWN_PENALTY;
            }
        }
        if self.board.player() == Player::White {
            score
        } else {
            -score
        }
    }

    /// Evaluate the position
    fn eval(&self) -> i32 {
        self.material_score()
            + self.mobility_score()
            + self.positional_score()
            + self.double_pawn_penalty()
            + self.passed_pawn_bonus()
            + self.isolated_pawn_penalty()
    }
}
