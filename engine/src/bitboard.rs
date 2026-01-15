use crate::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr};
use std::sync::OnceLock;

static BETWEEN_BITBOARD: OnceLock<[[Bitboard; 64]; 64]> = OnceLock::new();
static RAY_BITBOARD: OnceLock<[[Bitboard; 64]; 64]> = OnceLock::new();

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bitboard {
    pub bitboard: u64,
}

impl Bitboard {
    pub fn get_square(&self, square: Square) -> bool {
        self.bitboard & (1 << square.square) != 0
    }

    pub fn get(&self, index: u8) -> bool {
        self.bitboard & (1 << index) != 0
    }

    pub fn set_square(&mut self, square: Square) {
        self.bitboard |= 1 << square.square;
    }

    pub fn set(&mut self, index: u8) {
        self.bitboard |= 1 << index;
    }

    pub fn reset(&mut self) {
        self.bitboard = 0;
    }

    pub fn clear_square(&mut self, square: Square) {
        self.bitboard &= !(1 << square.square);
    }

    pub fn clear(&mut self, index: u8) {
        self.bitboard &= !(1 << index);
    }

    pub fn empty(&self) -> bool {
        self.bitboard == 0
    }

    pub fn intersects(&self, other: Bitboard) -> bool {
        self.bitboard & other.bitboard != 0
    }

    pub fn update(&mut self, from: u8, to: u8) {
        self.clear(to);
        if self.get(from) {
            self.clear(from);
            self.set(to);
        }
    }

    pub fn update_square(&mut self, from: Square, to: Square) {
        self.clear_square(to);
        if self.get_square(from) {
            self.clear_square(from);
            self.set_square(to);
        }
    }

    pub fn get_lsb(&self) -> u8 {
        self.bitboard.trailing_zeros() as u8
    }

    pub fn pop_lsb(&mut self) -> u8 {
        let lsb = self.get_lsb();
        self.clear(lsb);
        lsb
    }

    pub fn count(&self) -> u8 {
        self.bitboard.count_ones() as u8
    }

    #[inline]
    pub fn iter(self) -> BitboardIter {
        BitboardIter {
            bitboard: self.bitboard,
        }
    }

    fn init_between_table() -> [[Bitboard; 64]; 64] {
        let mut table = [[Bitboard::default(); 64]; 64];

        #[allow(clippy::needless_range_loop)]
        for sq1 in 0..64 {
            for sq2 in 0..64 {
                if sq1 == sq2 {
                    table[sq1][sq2] = Bitboard::default();
                    continue;
                }

                let s1 = Square::from(sq1 as u8);
                let s2 = Square::from(sq2 as u8);

                let rank1 = s1.rank;
                let file1 = s1.file;
                let rank2 = s2.rank;
                let file2 = s2.file;

                let mut between = Bitboard::default();

                if rank1 == rank2 {
                    let start_file = file1.min(file2);
                    let end_file = file1.max(file2);
                    for file in (start_file + 1)..end_file {
                        between.set_square(Square::new(rank1, file));
                    }
                } else if file1 == file2 {
                    let start_rank = rank1.min(rank2);
                    let end_rank = rank1.max(rank2);
                    for rank in (start_rank + 1)..end_rank {
                        between.set_square(Square::new(rank, file1));
                    }
                } else {
                    let rank_diff = rank1 as i8 - rank2 as i8;
                    let file_diff = file1 as i8 - file2 as i8;

                    if rank_diff.abs() == file_diff.abs() {
                        let rank_step = if rank1 < rank2 { 1 } else { -1 };
                        let file_step = if file1 < file2 { 1 } else { -1 };

                        let mut current_rank = rank1 as i8 + rank_step;
                        let mut current_file = file1 as i8 + file_step;
                        let target_rank = rank2 as i8;

                        while current_rank != target_rank {
                            between.set_square(Square::new(current_rank as u8, current_file as u8));
                            current_rank += rank_step;
                            current_file += file_step;
                        }
                    }
                }

                table[sq1][sq2] = between;
            }
        }

        table
    }

    fn init_ray_table() -> [[Bitboard; 64]; 64] {
        let mut table = [[Bitboard::default(); 64]; 64];

        #[allow(clippy::needless_range_loop)]
        for sq1 in 0..64 {
            for sq2 in 0..64 {
                let s1 = Square::from(sq1 as u8);
                let s2 = Square::from(sq2 as u8);

                let rank1 = s1.rank;
                let file1 = s1.file;
                let rank2 = s2.rank;
                let file2 = s2.file;

                let mut ray = Bitboard::default();

                if rank1 == rank2 {
                    for file in 0..8 {
                        ray.set_square(Square::new(rank1, file));
                    }
                } else if file1 == file2 {
                    for rank in 0..8 {
                        ray.set_square(Square::new(rank, file1));
                    }
                } else {
                    let rank_diff = rank1 as i8 - rank2 as i8;
                    let file_diff = file1 as i8 - file2 as i8;

                    if rank_diff.abs() == file_diff.abs() {
                        let rank_step = if rank1 < rank2 { 1 } else { -1 };
                        let file_step = if file1 < file2 { 1 } else { -1 };

                        let mut current_rank = rank1 as i8;
                        let mut current_file = file1 as i8;

                        while current_rank - rank_step >= 0
                            && (0..8).contains(&(current_rank - rank_step))
                            && (0..8).contains(&(current_file - file_step))
                        {
                            current_rank -= rank_step;
                            current_file -= file_step;
                        }

                        while (0..8).contains(&current_rank) && (0..8).contains(&current_file) {
                            ray.set_square(Square::new(current_rank as u8, current_file as u8));
                            current_rank += rank_step;
                            current_file += file_step;
                        }
                    }
                }

                table[sq1][sq2] = ray;
            }
        }

        table
    }

    pub fn init() {
        BETWEEN_BITBOARD.get_or_init(Bitboard::init_between_table);
        RAY_BITBOARD.get_or_init(Bitboard::init_ray_table);
    }

    pub fn between(sq1: Square, sq2: Square) -> Bitboard {
        let table = BETWEEN_BITBOARD.get_or_init(Bitboard::init_between_table);
        table[sq1.square as usize][sq2.square as usize]
    }

    pub fn ray(sq1: Square, sq2: Square) -> Bitboard {
        let table = RAY_BITBOARD.get_or_init(Bitboard::init_ray_table);
        table[sq1.square as usize][sq2.square as usize]
    }
}

impl From<u64> for Bitboard {
    fn from(x: u64) -> Self {
        Self { bitboard: x }
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Self {
            bitboard: 1 << square.square,
        }
    }
}

impl From<Bitboard> for Square {
    fn from(bitboard: Bitboard) -> Self {
        debug_assert!(bitboard.count() == 1,);
        Square::from(bitboard.get_lsb())
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard {
            bitboard: self.bitboard & rhs.bitboard,
        }
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard {
            bitboard: self.bitboard | rhs.bitboard,
        }
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn not(self) -> Bitboard {
        Bitboard {
            bitboard: !self.bitboard,
        }
    }
}

impl Shl<u32> for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn shl(self, shift: u32) -> Bitboard {
        Bitboard {
            bitboard: self.bitboard << shift,
        }
    }
}

impl Shr<u32> for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn shr(self, shift: u32) -> Bitboard {
        Bitboard {
            bitboard: self.bitboard >> shift,
        }
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard) {
        self.bitboard |= rhs.bitboard;
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard) {
        self.bitboard &= rhs.bitboard;
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in (0..8).rev() {
            for j in 0..8 {
                let idx = (i * 8 + j) as u8;
                let ch = if self.get(idx) { 'X' } else { '-' };
                write!(f, "{} ", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct BitboardIter {
    bitboard: u64,
}

impl Iterator for BitboardIter {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bitboard == 0 {
            return None;
        }

        let lsb = self.bitboard & (!self.bitboard + 1);
        let idx = lsb.trailing_zeros() as u8;

        self.bitboard ^= lsb;

        Some(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator() {
        let mut bitboard = Bitboard::default();
        let positions = [1, 4, 6, 7, 18, 43, 63];

        for &pos in &positions {
            bitboard.set(pos);
        }

        let result: Vec<u8> = bitboard.iter().collect();
        assert_eq!(result, positions);
    }

    #[test]
    fn test_between_diagonal() {
        let sq1 = Square::from(0);
        let sq2 = Square::from(63);
        let between = Bitboard::between(sq1, sq2);
        println!("{}", between);
        assert!(
            between.get(9)
                && between.get(18)
                && between.get(27)
                && between.get(36)
                && between.get(45)
                && between.get(54)
        );
    }

    #[test]
    fn test_ray_horizontal() {
        let e4 = "e4".parse::<Square>().unwrap();
        let f4 = "f4".parse::<Square>().unwrap();
        let ray = Bitboard::ray(e4, f4);

        for file in 0..8 {
            let square = Square::new(3, file);
            assert!(
                ray.get_square(square),
                "Square {} should be on the ray",
                square
            );
        }
        assert_eq!(ray.count(), 8, "Ray should contain all 8 squares on rank 4");
    }

    #[test]
    fn test_ray_vertical() {
        let e4 = "e4".parse::<Square>().unwrap();
        let e5 = "e5".parse::<Square>().unwrap();
        let ray = Bitboard::ray(e4, e5);

        for rank in 0..8 {
            let square = Square::new(rank, 4);
            assert!(
                ray.get_square(square),
                "Square {} should be on the ray",
                square
            );
        }
        assert_eq!(ray.count(), 8, "Ray should contain all 8 squares on file e");
    }

    #[test]
    fn test_ray_same_square() {
        let sq = Square::from(20);
        let ray = Bitboard::ray(sq, sq);
        assert!(ray.get_square(sq), "Ray should contain the square itself");
    }

    #[test]
    fn test_square_to_bitboard() {
        let sq = Square::from(20);
        let bb: Bitboard = sq.into();
        assert!(bb.get(20));
        assert_eq!(bb.count(), 1);
    }

    #[test]
    fn test_bitboard_to_square() {
        let mut bb = Bitboard::default();
        bb.set(42);
        let sq: Square = bb.into();
        assert_eq!(sq.square, 42);
    }
}
