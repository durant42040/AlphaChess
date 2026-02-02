use crate::chess::constants::TRANSPOSITION_TABLE_SIZE;

#[derive(Clone)]
pub struct TranspositionTable {
    table: Vec<Entry>,
}

#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub enum Flag {
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
    pub flag: Flag,
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

        if entry.flag == Flag::Exact
            || (entry.flag == Flag::Upper && entry.score <= alpha)
            || (entry.flag == Flag::Lower && entry.score >= beta)
        {
            return Some(entry.score);
        }

        None
    }

    pub fn store(&mut self, hash: u64, depth: u8, score: i32, flag: Flag) {
        let idx = hash as usize % TRANSPOSITION_TABLE_SIZE;
        if self.table[idx].depth >= depth {
            return;
        }

        let entry = Entry::new(hash, depth, score, flag);
        self.table[idx] = entry;
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
