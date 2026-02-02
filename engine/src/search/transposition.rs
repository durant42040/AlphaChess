use arrayvec::ArrayVec;

use crate::chess::constants::TRANSPOSITION_TABLE_SIZE;

#[derive(Clone)]
pub struct TranspositionTable {
    table: ArrayVec<Entry, TRANSPOSITION_TABLE_SIZE>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Flag {
    Exact,
    Upper,
    Lower,
}

#[derive(Clone)]
pub struct Entry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub flag: Flag,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: ArrayVec::<Entry, TRANSPOSITION_TABLE_SIZE>::new(),
        }
    }

    pub fn probe(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let entry = &self.table[hash as usize % TRANSPOSITION_TABLE_SIZE];
        if entry.hash != hash || entry.depth < depth {
            return None;
        }

        if entry.flag == Flag::Exact
            || (entry.flag == Flag::Upper && entry.score <= alpha)
            || (entry.flag == Flag::Lower && entry.score >= beta)
        {
            return Some(entry.score);
        }

        None
    }

    pub fn store(&mut self, hash: u64, depth: u8, score: i32, flag: Flag) {
        let entry = Entry::new(hash, depth, score, flag);
        self.table[hash as usize % TRANSPOSITION_TABLE_SIZE] = entry;
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Entry {
    pub fn new(hash: u64, depth: u8, score: i32, flag: Flag) -> Self {
        Self {
            hash,
            depth,
            score,
            flag,
        }
    }
}
