use crate::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr};

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
}

impl From<u64> for Bitboard {
    fn from(x: u64) -> Self {
        Self { bitboard: x }
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
}
