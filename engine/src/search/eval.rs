use crate::chess::{Bitboard, Piece};
use crate::constants::{
    ATTACK_WEIGHT, BLACK_BISHOP_SCORE, BLACK_KING_SCORE, BLACK_KING_ZONE, BLACK_PASSED_MASK, BLACK_PAWN_SCORE, BLACK_ROOK_SCORE, DOUBLE_PAWN_PENALTY, FILE_MASKS, ISOLATED_MASK, ISOLATED_PAWN_PENALTY, KING_SHIELD_BONUS, KNIGHT_SCORE, OPEN_FILE_BONUS, PASSED_PAWN_BONUS, RANK_7_BONUS, SEMI_OPEN_FILE_BONUS, WHITE_BISHOP_SCORE, WHITE_KING_SCORE, WHITE_KING_ZONE, WHITE_PASSED_MASK, WHITE_PAWN_SCORE, WHITE_ROOK_SCORE
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
        let white_mobile_pieces =
            self.pieces().white_pieces() & !self.pieces().kings() & !self.pieces().pawns();
        for from in white_mobile_pieces.iter() {
            let moves = self.generate_moves(Square::from(from));
            score += moves.count() as i32;
        }
        let black_mobile_pieces =
            self.pieces().black_pieces() & !self.pieces().kings() & !self.pieces().pawns();
        for from in black_mobile_pieces.iter() {
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

        score
    }

    fn double_pawn_penalty(&self) -> i32 {
        let mut score = 0;
        
        for file in &FILE_MASKS {
            let white_pawns = self.pieces().white_pieces()
                & self.pieces().pawns()
                & Bitboard::from(*file);
            let white_pawns_count = white_pawns.count();
            if white_pawns_count > 1 {
                score += DOUBLE_PAWN_PENALTY * (white_pawns_count - 1) as i32;
            }

            let black_pawns = self.pieces().black_pieces()
                & self.pieces().pawns()
                & Bitboard::from(*file);
            let black_pawns_count = black_pawns.count();
            if black_pawns_count > 1 {
                score -= DOUBLE_PAWN_PENALTY * (black_pawns_count - 1) as i32;
            }
        }
        score
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
        score
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
        score
    }

    fn rook_open_file_bonus(&self) -> i32 {
        let mut score = 0;
        let white_rooks = self.pieces().white_pieces() & self.pieces().rooks();
        let white_pawns = self.pieces().white_pieces() & self.pieces().pawns();
        let black_rooks = self.pieces().black_pieces() & self.pieces().rooks();
        let black_pawns = self.pieces().black_pieces() & self.pieces().pawns();
        for idx in white_rooks.iter() {
            let square = Square::from(idx);
            let file_mask = Bitboard::from(FILE_MASKS[square.file as usize]);
            if !white_pawns.intersects(file_mask) {
                score += SEMI_OPEN_FILE_BONUS;
                if !black_pawns.intersects(file_mask) {
                    score += OPEN_FILE_BONUS;
                }
            }
            // bonus for rooks on the 7th rank
            if square.rank == 6 {
                score += RANK_7_BONUS;
            }
        }
        for idx in black_rooks.iter() {
            let square = Square::from(idx);
            let file_mask = Bitboard::from(FILE_MASKS[square.file as usize]);
            if !black_pawns.intersects(file_mask) {
                score -= SEMI_OPEN_FILE_BONUS;
                if !white_pawns.intersects(file_mask) {
                    score -= OPEN_FILE_BONUS;
                }
            }
            if square.rank == 1 {
                score -= RANK_7_BONUS;
            }
        }
        score
    }

    fn king_safety(&self) -> i32 {
        let mut score = 0;
        let white_pieces = self.pieces().white_pieces();
        let black_pieces = self.pieces().black_pieces();
        let white_king = white_pieces & self.pieces().kings();
        let black_king = black_pieces & self.pieces().kings();
        let white_pawns = white_pieces & self.pieces().pawns();
        let black_pawns = black_pieces & self.pieces().pawns();
        
        // The king shield is the number of friendly pieces near the king
        score += (self.move_generator.generate_king_moves(Square::from(white_king)) & self.pieces().white_pieces()).count() as i32 * KING_SHIELD_BONUS;
        score -= (self.move_generator.generate_king_moves(Square::from(black_king)) & self.pieces().black_pieces()).count() as i32 * KING_SHIELD_BONUS;
        
        // semi-open and open file penalties
        let square = Square::from(white_king);
        let file_mask = Bitboard::from(FILE_MASKS[square.file as usize]);
        if !white_pawns.intersects(file_mask) {
            score -= SEMI_OPEN_FILE_BONUS;
            if !black_pawns.intersects(file_mask) {
                score -= OPEN_FILE_BONUS;
            }
        }
        let square = Square::from(black_king);
        let file_mask = Bitboard::from(FILE_MASKS[square.file as usize]);
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
            let moves = self.generate_moves(Square::from(square));
            if white_king_zone.intersects(moves) {
                attack_count += 1;
                let attacked_squares = (white_king_zone & moves).count() as i32;
                attack_value += attacked_squares * self.pieces().attack_value(Square::from(square));
            }
        }
        score -= attack_value * ATTACK_WEIGHT[attack_count] / 100;
        
        let black_king_zone = Bitboard::from(BLACK_KING_ZONE[black_king.get_lsb() as usize]);
        attack_count = 0;
        attack_value = 0;

        for square in (white_pieces & !white_king & !white_pawns).iter() {
            let moves = self.generate_moves(Square::from(square));
            if black_king_zone.intersects(moves) {
                attack_count += 1;
                let attacked_squares = (black_king_zone & moves).count() as i32;
                attack_value += attacked_squares * self.pieces().attack_value(Square::from(square));
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
