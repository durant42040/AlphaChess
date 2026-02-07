use crate::chess::Piece;
use crate::constants::{
    BLACK_BISHOP_SCORE, BLACK_KING_SCORE, BLACK_PAWN_SCORE, BLACK_ROOK_SCORE, KNIGHT_SCORE,
    WHITE_BISHOP_SCORE, WHITE_KING_SCORE, WHITE_PAWN_SCORE, WHITE_ROOK_SCORE,
};
use crate::{
    Engine,
    chess::{Color, Player, Square},
};

pub trait Evaluation {
    fn material_score(&self) -> i32;
    fn mobility_score(&self) -> i32;
    fn positional_score(&self) -> i32;
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

    /// Evaluate the position
    fn eval(&self) -> i32 {
        self.material_score() + self.mobility_score() + self.positional_score()
    }
}
