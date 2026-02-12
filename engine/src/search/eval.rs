use crate::chess::Bitboard;
use crate::constants::{
    ATTACK_WEIGHT, BLACK_KING_ZONE, BLACK_PASSED_MASK, DOUBLE_PAWN_PENALTY, ENDGAME_BISHOP_SCORE,
    ENDGAME_KING_SCORE, ENDGAME_KNIGHT_SCORE, ENDGAME_PASSED_PAWN_BONUS, ENDGAME_PAWN_SCORE,
    ENDGAME_QUEEN_SCORE, ENDGAME_ROOK_SCORE, FILE_MASKS, ISOLATED_MASK, ISOLATED_PAWN_PENALTY,
    KING_SHIELD_BONUS, OPEN_FILE_BONUS, OPENING_BISHOP_SCORE, OPENING_KING_SCORE,
    OPENING_KNIGHT_SCORE, OPENING_PASSED_PAWN_BONUS, OPENING_PAWN_SCORE, OPENING_QUEEN_SCORE,
    OPENING_ROOK_SCORE, RANK_7_BONUS, SEMI_OPEN_FILE_BONUS, WHITE_KING_ZONE, WHITE_PASSED_MASK,
};
use crate::{
    Engine,
    chess::{Player, Square},
};

#[inline(always)]
fn piece_score_sum(white: Bitboard, black: Bitboard, score: &[i32; 64]) -> i32 {
    let mut sum = 0;
    for i in white.iter() {
        sum += score[(i ^ 56) as usize];
    }
    for i in black.iter() {
        sum -= score[i as usize];
    }
    sum
}

pub trait Evaluation {
    fn material_score(&self) -> i32;
    fn mobility_score(&self) -> i32;
    fn positional_score(&self) -> i32;
    fn double_pawn_penalty(&self) -> i32;
    fn isolated_pawn_penalty(&self) -> i32;
    fn passed_pawn_bonus(&self) -> i32;
    fn rook_open_file_bonus(&self) -> i32;
    fn king_safety(&self) -> i32;
    fn eval(&self) -> i32;
}

impl Evaluation for Engine {
    fn material_score(&self) -> i32 {
        self.board.material_score()
    }

    fn mobility_score(&self) -> i32 {
        let mut score = 0;
        let pieces = self.pieces();
        let white_mobile_pieces =
            pieces.white_pieces() & !pieces.kings() & !pieces.pawns() & !pieces.queens();
        for from in white_mobile_pieces.iter() {
            let moves = self.generate_moves(Square::from(from));
            score += moves.count() as i32;
        }
        let black_mobile_pieces =
            pieces.black_pieces() & !pieces.kings() & !pieces.pawns() & !pieces.queens();
        for from in black_mobile_pieces.iter() {
            let moves = self.generate_moves(Square::from(from));
            score -= moves.count() as i32;
        }
        score
    }

    fn positional_score(&self) -> i32 {
        let pieces = self.pieces();
        let white_pieces = pieces.white_pieces();
        let black_pieces = pieces.black_pieces();
        let pawns = pieces.pawns();
        let knights = pieces.knights();
        let bishops = pieces.bishops();
        let rooks = pieces.rooks();
        let queens = pieces.queens();
        let kings = pieces.kings();

        if self.is_endgame() {
            piece_score_sum(
                white_pieces & pawns,
                black_pieces & pawns,
                &ENDGAME_PAWN_SCORE,
            ) + piece_score_sum(
                white_pieces & knights,
                black_pieces & knights,
                &ENDGAME_KNIGHT_SCORE,
            ) + piece_score_sum(
                white_pieces & bishops,
                black_pieces & bishops,
                &ENDGAME_BISHOP_SCORE,
            ) + piece_score_sum(
                white_pieces & rooks,
                black_pieces & rooks,
                &ENDGAME_ROOK_SCORE,
            ) + piece_score_sum(
                white_pieces & queens,
                black_pieces & queens,
                &ENDGAME_QUEEN_SCORE,
            ) + piece_score_sum(
                white_pieces & kings,
                black_pieces & kings,
                &ENDGAME_KING_SCORE,
            )
        } else {
            piece_score_sum(
                white_pieces & pawns,
                black_pieces & pawns,
                &OPENING_PAWN_SCORE,
            ) + piece_score_sum(
                white_pieces & knights,
                black_pieces & knights,
                &OPENING_KNIGHT_SCORE,
            ) + piece_score_sum(
                white_pieces & bishops,
                black_pieces & bishops,
                &OPENING_BISHOP_SCORE,
            ) + piece_score_sum(
                white_pieces & rooks,
                black_pieces & rooks,
                &OPENING_ROOK_SCORE,
            ) + piece_score_sum(
                white_pieces & queens,
                black_pieces & queens,
                &OPENING_QUEEN_SCORE,
            ) + piece_score_sum(
                white_pieces & kings,
                black_pieces & kings,
                &OPENING_KING_SCORE,
            )
        }
    }

    fn double_pawn_penalty(&self) -> i32 {
        let mut score = 0;

        for file in &FILE_MASKS {
            let white_pawns =
                self.pieces().white_pieces() & self.pieces().pawns() & Bitboard::from(*file);
            let white_pawns_count = white_pawns.count();
            if white_pawns_count > 1 {
                score += DOUBLE_PAWN_PENALTY * (white_pawns_count - 1) as i32;
            }

            let black_pawns =
                self.pieces().black_pieces() & self.pieces().pawns() & Bitboard::from(*file);
            let black_pawns_count = black_pawns.count();
            if black_pawns_count > 1 {
                score -= DOUBLE_PAWN_PENALTY * (black_pawns_count - 1) as i32;
            }
        }
        score
    }

    fn passed_pawn_bonus(&self) -> i32 {
        let pieces = self.pieces();
        let white_pawns = pieces.white_pieces() & pieces.pawns();
        let black_pawns = pieces.black_pieces() & pieces.pawns();
        let mut score = 0;
        for square in white_pawns.iter() {
            if !black_pawns.intersects(Bitboard::from(WHITE_PASSED_MASK[square as usize])) {
                score += if self.is_endgame() {
                    ENDGAME_PASSED_PAWN_BONUS[(square / 8) as usize]
                } else {
                    OPENING_PASSED_PAWN_BONUS[(square / 8) as usize]
                };
            }
        }
        for square in black_pawns.iter() {
            if !white_pawns.intersects(Bitboard::from(BLACK_PASSED_MASK[square as usize])) {
                score -= if self.is_endgame() {
                    ENDGAME_PASSED_PAWN_BONUS[7 - (square / 8) as usize]
                } else {
                    OPENING_PASSED_PAWN_BONUS[7 - (square / 8) as usize]
                };
            }
        }
        score
    }

    fn isolated_pawn_penalty(&self) -> i32 {
        let pieces = self.pieces();
        let white_pawns = pieces.white_pieces() & pieces.pawns();
        let black_pawns = pieces.black_pieces() & pieces.pawns();
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
        score
    }

    fn rook_open_file_bonus(&self) -> i32 {
        let mut score = 0;
        let pieces = self.pieces();
        let white_rooks = pieces.white_pieces() & pieces.rooks();
        let white_pawns = pieces.white_pieces() & pieces.pawns();
        let black_rooks = pieces.black_pieces() & pieces.rooks();
        let black_pawns = pieces.black_pieces() & pieces.pawns();
        for idx in white_rooks.iter() {
            let square = Square::from(idx);
            let file_mask = Bitboard::from(FILE_MASKS[square.file() as usize]);
            if !white_pawns.intersects(file_mask) {
                score += SEMI_OPEN_FILE_BONUS;
                if !black_pawns.intersects(file_mask) {
                    score += OPEN_FILE_BONUS;
                }
            }
            // bonus for rooks on the 7th rank
            if square.rank() == 6 {
                score += RANK_7_BONUS;
            }
        }
        for idx in black_rooks.iter() {
            let square = Square::from(idx);
            let file_mask = Bitboard::from(FILE_MASKS[square.file() as usize]);
            if !black_pawns.intersects(file_mask) {
                score -= SEMI_OPEN_FILE_BONUS;
                if !white_pawns.intersects(file_mask) {
                    score -= OPEN_FILE_BONUS;
                }
            }
            if square.rank() == 1 {
                score -= RANK_7_BONUS;
            }
        }
        score
    }

    fn king_safety(&self) -> i32 {
        let mut score = 0;
        let pieces = self.pieces();
        let white_pieces = pieces.white_pieces();
        let black_pieces = pieces.black_pieces();
        let white_king = white_pieces & pieces.kings();
        let black_king = black_pieces & pieces.kings();
        let white_pawns = white_pieces & pieces.pawns();
        let black_pawns = black_pieces & pieces.pawns();

        // The king shield is the number of friendly pieces near the king
        score += (self
            .move_generator
            .generate_king_moves(Square::from(white_king))
            & white_pieces)
            .count() as i32
            * KING_SHIELD_BONUS;
        score -= (self
            .move_generator
            .generate_king_moves(Square::from(black_king))
            & black_pieces)
            .count() as i32
            * KING_SHIELD_BONUS;

        // semi-open and open file penalties
        let square = Square::from(white_king);
        let file_mask = Bitboard::from(FILE_MASKS[square.file() as usize]);
        if !white_pawns.intersects(file_mask) {
            score -= SEMI_OPEN_FILE_BONUS;
            if !black_pawns.intersects(file_mask) {
                score -= OPEN_FILE_BONUS;
            }
        }
        let square = Square::from(black_king);
        let file_mask = Bitboard::from(FILE_MASKS[square.file() as usize]);
        if !black_pawns.intersects(file_mask) {
            score += SEMI_OPEN_FILE_BONUS;
            if !white_pawns.intersects(file_mask) {
                score += OPEN_FILE_BONUS;
            }
        }

        // king zone attacks
        let white_king_zone = Bitboard::from(WHITE_KING_ZONE[white_king.get_lsb() as usize]);
        let mut attack_count = 0;
        let mut attack_value = 0;

        for square in (black_pieces & !black_king & !black_pawns).iter() {
            let square = Square::from(square);
            let moves = self.generate_moves(square);
            if white_king_zone.intersects(moves) {
                attack_count += 1;
                let attacked_squares = (white_king_zone & moves).count() as i32;
                attack_value += attacked_squares * self.pieces().attack_value(square);
            }
        }
        score -= attack_value * ATTACK_WEIGHT[attack_count] / 100;

        let black_king_zone = Bitboard::from(BLACK_KING_ZONE[black_king.get_lsb() as usize]);
        attack_count = 0;
        attack_value = 0;

        for square in (white_pieces & !white_king & !white_pawns).iter() {
            let square = Square::from(square);
            let moves = self.generate_moves(square);
            if black_king_zone.intersects(moves) {
                attack_count += 1;
                let attacked_squares = (black_king_zone & moves).count() as i32;
                attack_value += attacked_squares * self.pieces().attack_value(square);
            }
        }
        score += attack_value * ATTACK_WEIGHT[attack_count] / 100;

        score
    }

    /// Evaluate the position
    fn eval(&self) -> i32 {
        let score = self.material_score()
            + self.mobility_score()
            + self.positional_score()
            + self.double_pawn_penalty()
            + self.passed_pawn_bonus()
            + self.isolated_pawn_penalty()
            + self.rook_open_file_bonus()
            + self.king_safety();

        if self.board.player() == Player::White {
            score
        } else {
            -score
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Engine, search::Evaluation};

    #[test]
    fn test_eval() {
        let engine = Engine::new();
        let eval = engine.eval();
        assert_eq!(eval, 0);
    }
}
