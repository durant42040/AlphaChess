use crate::chess::Move;
use crate::chess::constants::TRANSPOSITION_TABLE_SIZE;

pub struct TranspositionTable {
    table: Vec<Entry>,
}

#[derive(Default, PartialEq, Eq, Copy, Clone)]
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

impl TranspositionTable {
    /// Transposition table is implemented as a preallocated vector of size 4MB. The key of each entry is the zobrist hash of the position.
    pub fn new() -> Self {
        Self {
            table: vec![Entry::default(); TRANSPOSITION_TABLE_SIZE],
        }
    }

    /// Probe the transposition table for a score. If the entry is not found or the depth is less than the stored depth, None is returned. Return only when bound is useful for pruning.
    pub fn probe(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let idx = hash as usize % TRANSPOSITION_TABLE_SIZE;
        let entry = &self.table[idx];
        if entry.hash != hash || entry.depth < depth {
            return None;
        }

        if entry.bound == Bound::Exact
            || (entry.bound == Bound::Upper && entry.score <= alpha)
            || (entry.bound == Bound::Lower && entry.score >= beta)
        {
            return Some(entry.score);
        }

        None
    }

    pub fn get_best_move(&self, hash: u64) -> Move {
        let idx = hash as usize % TRANSPOSITION_TABLE_SIZE;
        let entry = &self.table[idx];
        if entry.hash != hash {
            return Move::none();
        }
        entry.best_move
    }

    pub fn store(&mut self, hash: u64, depth: u8, score: i32, bound: Bound, best_move: Move) {
        let idx = hash as usize % TRANSPOSITION_TABLE_SIZE;
        if self.table[idx].depth >= depth {
            return;
        }

        let entry = Entry::new(hash, depth, score, bound, best_move);
        self.table[idx] = entry;
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
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
