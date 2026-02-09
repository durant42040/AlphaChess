use crate::Engine;

impl Engine {
    pub fn perft(&mut self, depth: u8) -> u64 {
        if depth == 1 {
            return self.generate_all_legal_moves().len() as u64;
        }

        let mut nodes = 0u64;
        for r#move in self.generate_all_legal_moves() {
            self.act(r#move);
            nodes += self.perft(depth - 1);
            self.undo();
        }

        nodes
    }
}
