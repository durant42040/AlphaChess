use crate::Engine;

pub struct Perft {
    engine: Engine,
}

impl Perft {
    pub fn new(fen: &str) -> Self {
        Self {
            engine: Engine::from_fen(fen),
        }
    }

    pub fn search(&mut self, depth: u8) -> u64 {
        if depth == 1 {
            return self.engine.generate_all_legal_moves().len() as u64;
        }

        let mut nodes = 0u64;
        for r#move in self.engine.generate_all_legal_moves() {
            self.engine.act(r#move);
            nodes += self.search(depth - 1);
            self.engine.undo();
        }

        nodes
    }
}
