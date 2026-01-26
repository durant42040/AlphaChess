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
        if depth == 0 {
            return 1;
        }
        if self.engine.get_game_state() != "playing" {
            return 0;
        }

        let moves = self.engine.generate_all_legal_moves();
        let mut nodes = 0u64;

        for r#move in moves {
            self.engine.act(&r#move.to_string());
            nodes += self.search(depth - 1);
            self.engine.undo();
        }

        nodes
    }
}
