use std::sync::atomic::{AtomicU64, Ordering};

use crate::chess::Move;
use crate::constants::TRANSPOSITION_TABLE_SIZE;

/// Lock-free transposition table designed for thread-safe concurrent access.
pub struct TranspositionTable {
    size: usize,
    keys: Box<[AtomicU64]>,
    table: Box<[AtomicU64]>,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum Bound {
    #[default]
    Exact,
    Upper,
    Lower,
}

#[derive(Clone)]
pub struct Entry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Move,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            hash: 0,
            depth: 0,
            score: 0,
            bound: Bound::default(),
            best_move: Move::none(),
        }
    }
}

impl Entry {
    pub fn new(hash: u64, depth: u8, score: i32, bound: Bound, best_move: Move) -> Self {
        Self {
            hash,
            depth,
            score,
            bound,
            best_move,
        }
    }
}

// Data word layout (64 bits):
// [0-15]:   move (16 bits)
// [16-17]:  bound (2 bits): Exact=0, Upper=1, Lower=2
// [18-25]:  depth (8 bits)
// [26-41]:  score (16 bits, i16)
// [42-63]:  unused
const BOUND_SHIFT: u32 = 16;
const DEPTH_SHIFT: u32 = 18;
const SCORE_SHIFT: u32 = 26;
const MOVE_MASK: u64 = 0xFFFF;

#[inline]
fn pack_data(depth: u8, score: i32, bound: Bound, best_move: Move) -> u64 {
    let score = score as u16 as u64;
    let bound_bits = match bound {
        Bound::Exact => 0u64,
        Bound::Upper => 1u64,
        Bound::Lower => 2u64,
    };
    (score << SCORE_SHIFT)
        | ((depth as u64) << DEPTH_SHIFT)
        | (bound_bits << BOUND_SHIFT)
        | (best_move.0 as u64 & MOVE_MASK)
}

#[inline]
fn unpack_move(data: u64) -> Move {
    Move((data & MOVE_MASK) as u16)
}

#[inline]
fn unpack_bound(data: u64) -> Bound {
    match (data >> BOUND_SHIFT) & 0x3 {
        0 => Bound::Exact,
        1 => Bound::Upper,
        _ => Bound::Lower,
    }
}

#[inline]
fn unpack_depth(data: u64) -> u8 {
    ((data >> DEPTH_SHIFT) & 0xFF) as u8
}

#[inline]
fn unpack_score(data: u64) -> i32 {
    ((data >> SCORE_SHIFT) & 0xFFFF) as i16 as i32
}

impl TranspositionTable {
    /// Transposition table is implemented as a preallocated vector of size 64MB. The key of each entry is the zobrist hash of the position.
    pub fn new() -> Self {
        Self {
            size: TRANSPOSITION_TABLE_SIZE,
            keys: (0..TRANSPOSITION_TABLE_SIZE)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            table: (0..TRANSPOSITION_TABLE_SIZE)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn with_size(size: usize) -> Self {
        Self {
            size,
            keys: (0..size)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            table: (0..size)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    /// Probe the transposition table for a score. If the entry is not found or the depth is less than the stored depth, None is returned. Return only when bound is useful for pruning.
    #[inline]
    pub fn probe(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let idx = hash as usize % self.size;
        if self.keys[idx].load(Ordering::Acquire) != hash {
            return None;
        }
        let data = self.table[idx].load(Ordering::Acquire);
        let entry_depth = unpack_depth(data);
        if entry_depth < depth {
            return None;
        }
        let score = unpack_score(data);
        let bound = unpack_bound(data);
        if bound == Bound::Exact
            || (bound == Bound::Upper && score <= alpha)
            || (bound == Bound::Lower && score >= beta)
        {
            return Some(score);
        }
        None
    }

    /// Get the best move for a position. Lock-free; safe to call from multiple threads.
    #[inline]
    pub fn get_best_move(&self, hash: u64) -> Move {
        let idx = hash as usize % self.size;
        if self.keys[idx].load(Ordering::Acquire) != hash {
            return Move::none();
        }
        unpack_move(self.table[idx].load(Ordering::Acquire))
    }

    /// Store an entry. Lock-free; safe to call from multiple threads.
    /// Write order: table first, then keys (ensures readers see consistent pairs).
    #[inline]
    pub fn store(&self, hash: u64, depth: u8, score: i32, bound: Bound, best_move: Move) {
        let idx = hash as usize % self.size;
        let current_key = self.keys[idx].load(Ordering::Acquire);
        let current_data = self.table[idx].load(Ordering::Acquire);
        let current_depth = unpack_depth(current_data);
        let current_bound = unpack_bound(current_data);

        let should_replace = current_key == 0
            || current_key != hash
            || (bound == Bound::Exact && current_bound != Bound::Exact)
            || depth >= current_depth;

        if !should_replace {
            return;
        }

        let new_data = pack_data(depth, score, bound, best_move);
        self.table[idx].store(new_data, Ordering::Release);
        self.keys[idx].store(hash, Ordering::Release);
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        let moves = [Move::none(), Move(1), Move(1234), Move(u16::MAX)];

        let depths = [0u8, 1, 10, 255];
        let scores = [-40000, -30000, -123, 0, 42, 30000, 40000];
        let bounds = [Bound::Exact, Bound::Upper, Bound::Lower];

        for &mv in &moves {
            for &depth in &depths {
                for &score in &scores {
                    for &bound in &bounds {
                        let data = pack_data(depth, score, bound, mv);

                        assert_eq!(unpack_depth(data), depth);
                        assert_eq!(unpack_bound(data), bound);
                        assert_eq!(unpack_move(data), mv);

                        let unpacked_score = unpack_score(data);
                        assert!(
                            unpacked_score >= i16::MIN as i32 && unpacked_score <= i16::MAX as i32
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn simple_store_and_probe() {
        let tt = TranspositionTable::with_size(1024);
        let hash = 0x1234_5678_9ABC_DEF0;

        tt.store(hash, 5, 42, Bound::Exact, Move(123));
        let res = tt.probe(hash, 5, -100, 100);

        assert_eq!(res, Some(42));
        assert_eq!(tt.get_best_move(hash), Move(123));
    }

    #[test]
    fn depth_replacement_rule() {
        let tt = TranspositionTable::with_size(1024);
        let hash = 0xCAFEBABE;

        tt.store(hash, 10, 10, Bound::Exact, Move(1));
        tt.store(hash, 5, 20, Bound::Upper, Move(2)); // should NOT replace

        assert_eq!(tt.probe(hash, 10, -100, 100), Some(10));
        assert_eq!(tt.get_best_move(hash), Move(1));
    }
}
