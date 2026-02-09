use crate::chess::{Player, Square};

pub struct History {
    table: [[[i32; 64]; 64]; 2],
}

impl History {
    pub fn new() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }

    #[inline(always)]
    pub fn get(&self, player: Player, from: Square, to: Square) -> i32 {
        self.table[player as usize][from.square as usize][to.square as usize]
    }

    #[inline(always)]
    pub fn update(&mut self, player: Player, from: Square, to: Square, depth: u8) {
        const MAX_HISTORY: i32 = 8192;
        let mut bonus = depth as i32 * depth as i32;
        bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
        let value = self.get(player, from, to);
        self.table[player as usize][from.square as usize][to.square as usize] +=
            bonus - value * bonus.abs() / MAX_HISTORY;
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
