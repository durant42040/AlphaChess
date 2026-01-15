use crate::{
    bitboard::Bitboard,
    constants::{
        BISHOP_MAGIC_NUMBERS, BISHOP_MASKS, BISHOP_SHIFT_BITS, BLACK_PAWN_CAPTURES, KING_ATTACKS,
        KNIGHT_ATTACKS, ROOK_MAGIC_NUMBERS, ROOK_MASKS, ROOK_SHIFT_BITS, WHITE_PAWN_CAPTURES,
    },
    square::Square,
};

fn get_blockers(index: usize, mask: Bitboard) -> u64 {
    let mut blockers = 0u64;
    let bits = mask.count();
    let mut mask_copy = mask;

    for i in 0..bits {
        let bit = mask_copy.pop_lsb();
        if index & (1 << i) != 0 {
            blockers |= 1u64 << bit;
        }
    }

    blockers
}

#[inline]
fn slide(moves: &mut Bitboard, all_pieces: Bitboard, mut i: i32, mut j: i32, di: i32, dj: i32) {
    i += di;
    j += dj;
    while (0..8).contains(&i) && (0..8).contains(&j) {
        let sq = (i * 8 + j) as u8;
        moves.set(sq);
        if all_pieces.get(sq) {
            break;
        }
        i += di;
        j += dj;
    }
}

pub struct MoveGenerator {
    rook: Vec<Vec<u64>>,
    bishop: Vec<Vec<u64>>,
    knight: [u64; 64],
    king: [u64; 64],
}

impl MoveGenerator {
    pub fn new() -> Self {
        let mut move_generator = Self {
            rook: vec![vec![0; 4096]; 64],
            bishop: vec![vec![0; 1024]; 64],
            knight: KNIGHT_ATTACKS,
            king: KING_ATTACKS,
        };
        move_generator.init_sliding_moves();
        move_generator
    }

    fn init_sliding_moves(&mut self) {
        for square in 0..64 {
            let num_configs = 1 << BISHOP_SHIFT_BITS[square];
            for i in 0..num_configs {
                let mask = Bitboard::from(BISHOP_MASKS[square]);
                let blockers = get_blockers(i, mask);
                let key = ((blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[square]))
                    >> (64 - BISHOP_SHIFT_BITS[square])) as usize;
                self.bishop[square][key] = self
                    .generate_bishop_moves_slow(
                        Square::from(square as u8),
                        Bitboard::from(blockers),
                    )
                    .bitboard;
            }
        }

        for square in 0..64 {
            let num_configs = 1 << ROOK_SHIFT_BITS[square];
            for i in 0..num_configs {
                let mask = Bitboard::from(ROOK_MASKS[square]);
                let blockers = get_blockers(i, mask);
                let key = ((blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[square]))
                    >> (64 - ROOK_SHIFT_BITS[square])) as usize;
                self.rook[square][key] = self
                    .generate_rook_moves_slow(Square::from(square as u8), Bitboard::from(blockers))
                    .bitboard;
            }
        }
    }

    pub fn generate_white_pawn_moves(
        &self,
        from: Square,
        all_pieces: Bitboard,
        capture_pieces: Bitboard,
    ) -> Bitboard {
        let from_mask = Bitboard::from(from);

        let one_step_moves = (from_mask << 8) & !all_pieces;
        let two_step_moves = ((one_step_moves & (0xFF << 16).into()) << 8) & !all_pieces;
        let capture_moves = Bitboard::from(WHITE_PAWN_CAPTURES[from]) & capture_pieces;

        one_step_moves | two_step_moves | capture_moves
    }

    pub fn generate_black_pawn_moves(
        &self,
        from: Square,
        all_pieces: Bitboard,
        capture_pieces: Bitboard,
    ) -> Bitboard {
        let from_mask = Bitboard::from(from);

        let one_step_moves = (from_mask >> 8) & !all_pieces;
        let two_step_moves = ((one_step_moves & (0xFF << 40).into()) >> 8) & !all_pieces;
        let capture_moves = Bitboard::from(BLACK_PAWN_CAPTURES[from]) & capture_pieces;

        one_step_moves | two_step_moves | capture_moves
    }

    pub fn generate_knight_moves(&self, from: Square) -> Bitboard {
        Bitboard::from(self.knight[from])
    }

    pub fn generate_king_moves(&self, from: Square) -> Bitboard {
        Bitboard::from(self.king[from])
    }

    fn generate_bishop_moves_slow(&self, from: Square, all_pieces: Bitboard) -> Bitboard {
        let rank = from.rank as i32;
        let file = from.file as i32;

        let mut moves = Bitboard::default();
        slide(&mut moves, all_pieces, rank, file, 1, 1);
        slide(&mut moves, all_pieces, rank, file, -1, 1);
        slide(&mut moves, all_pieces, rank, file, 1, -1);
        slide(&mut moves, all_pieces, rank, file, -1, -1);
        moves
    }

    fn generate_rook_moves_slow(&self, from: Square, all_pieces: Bitboard) -> Bitboard {
        let file = from.file as i32;
        let rank = from.rank as i32;
        let mut moves = Bitboard::default();
        slide(&mut moves, all_pieces, rank, file, 1, 0);
        slide(&mut moves, all_pieces, rank, file, -1, 0);
        slide(&mut moves, all_pieces, rank, file, 0, 1);
        slide(&mut moves, all_pieces, rank, file, 0, -1);
        moves
    }

    pub fn generate_bishop_moves(&self, from: Square, all_pieces: Bitboard) -> Bitboard {
        let blockers = all_pieces.bitboard & BISHOP_MASKS[from];
        let key = ((blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[from]))
            >> (64 - BISHOP_SHIFT_BITS[from])) as usize;

        Bitboard::from(self.bishop[from][key])
    }

    pub fn generate_rook_moves(&self, from: Square, all_pieces: Bitboard) -> Bitboard {
        let blockers = all_pieces.bitboard & ROOK_MASKS[from];
        let key = ((blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[from]))
            >> (64 - ROOK_SHIFT_BITS[from])) as usize;

        Bitboard::from(self.rook[from][key])
    }

    pub fn generate_queen_moves(&self, from: Square, all_pieces: Bitboard) -> Bitboard {
        self.generate_rook_moves(from, all_pieces) | self.generate_bishop_moves(from, all_pieces)
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_bitboards() {
        let move_generator = MoveGenerator::new();

        // Test bishop moves for all squares with an empty board.
        for sq in 0u8..64u8 {
            let from = Square::from(sq);
            let all_pieces = Bitboard::from(0);
            let fast = move_generator.generate_bishop_moves(from, all_pieces);
            let slow = move_generator.generate_bishop_moves_slow(from, all_pieces);
            assert_eq!(
                fast, slow,
                "Bishop mismatch at square {}: fast={:?}, slow={:?}",
                sq, fast, slow
            );
        }

        // Test rook moves for all squares with an empty board.
        for sq in 0u8..64u8 {
            let from = Square::from(sq);
            let all_pieces = Bitboard::from(0);
            let fast = move_generator.generate_rook_moves(from, all_pieces);
            let slow = move_generator.generate_rook_moves_slow(from, all_pieces);
            assert_eq!(
                fast, slow,
                "Rook mismatch at square {}: fast={:?}, slow={:?}",
                sq, fast, slow
            );
        }
    }
}
