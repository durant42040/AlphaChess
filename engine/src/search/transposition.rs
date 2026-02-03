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

#[derive(Default, Clone)]
pub struct Entry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub bound: Bound,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: vec![Entry::default(); TRANSPOSITION_TABLE_SIZE],
        }
    }

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

    pub fn store(&mut self, hash: u64, depth: u8, score: i32, bound: Bound) {
        let idx = hash as usize % TRANSPOSITION_TABLE_SIZE;
        if self.table[idx].depth >= depth {
            return;
        }

        let entry = Entry::new(hash, depth, score, bound);
        self.table[idx] = entry;
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Entry {
    pub fn new(hash: u64, depth: u8, score: i32, bound: Bound) -> Self {
        Self {
            hash,
            depth,
            score,
            bound,
        }
    }
}
