use std::cmp::max;

use crate::{
    Engine,
    chess::{Bitboard, Color, Move, Piece},
};

impl Engine {
    /// Static Exchange Evaluation (SEE)
    pub fn see(&mut self, r#move: Move) -> i32 {
        let from = r#move.from;
        let to = r#move.to;
        let mut pieces = self.pieces();
        debug_assert!(self.is_legal_move(r#move), "Move is not legal");

        let mut color = self.board.player().color();
        let mut occupied = pieces.all_pieces() & !Bitboard::from(from);
        let mut all_attackers = self.generate_attacks(to, occupied, Color::White)
            | self.generate_attacks(to, occupied, Color::Black);

        let mut gain = [0i32; 32];
        gain[0] = pieces.value(to);
        gain[1] = pieces.value(from) - pieces.value(to);
        let mut depth = 1;

        loop {
            color = color.opposite();
            all_attackers &= occupied;

            let our_pieces = self.board.pieces_of_color(color);
            let their_pieces = self.board.pieces_of_color(color.opposite());

            let our_attackers = all_attackers & our_pieces;
            // TODO: remove pinned pieces from all_attackers
            if our_attackers.empty() {
                break;
            }

            depth += 1;

            if our_attackers.intersects(pieces.pawns()) {
                let pawns = pieces.pawns() & our_attackers;
                let pawn = pawns.get_lsb();

                occupied.clear(pawn);
                pieces.clear(Piece::Pawn, color, pawn);

                all_attackers |= self.move_generator.generate_bishop_moves(to, occupied)
                    & ((pieces.bishops() | pieces.queens()) & their_pieces);

                gain[depth] = Piece::Pawn.value() - gain[depth - 1];
            } else if our_attackers.intersects(pieces.knights()) {
                let knights = pieces.knights() & our_attackers;
                let knight = knights.get_lsb();

                occupied.clear(knight);
                pieces.clear(Piece::Knight, color, knight);

                gain[depth] = Piece::Knight.value() - gain[depth - 1];
            } else if our_attackers.intersects(pieces.bishops()) {
                let bishops = pieces.bishops() & our_attackers;
                let bishop = bishops.get_lsb();

                occupied.clear(bishop);
                pieces.clear(Piece::Bishop, color, bishop);

                all_attackers |= self.move_generator.generate_bishop_moves(to, occupied)
                    & ((pieces.bishops() | pieces.queens()) & their_pieces);

                gain[depth] = Piece::Bishop.value() - gain[depth - 1];
            } else if our_attackers.intersects(pieces.rooks()) {
                let rooks = pieces.rooks() & our_attackers;
                let rook = rooks.get_lsb();

                occupied.clear(rook);
                pieces.clear(Piece::Rook, color, rook);

                all_attackers |= self.move_generator.generate_rook_moves(to, occupied)
                    & (pieces.rooks() | pieces.queens());

                gain[depth] = Piece::Rook.value() - gain[depth - 1];
            } else if our_attackers.intersects(pieces.queens()) {
                let queens = pieces.queens() & our_attackers;
                let queen = queens.get_lsb();

                occupied.clear(queen);
                pieces.clear(Piece::Queen, color, queen);

                all_attackers |= self.move_generator.generate_rook_moves(to, occupied)
                    & ((pieces.rooks() | pieces.queens()) & their_pieces);
                all_attackers |= self.move_generator.generate_bishop_moves(to, occupied)
                    & ((pieces.bishops() | pieces.queens()) & their_pieces);

                gain[depth] = Piece::Queen.value() - gain[depth - 1];
            } else {
                // king can not recapture if there are still enemy attackers
                if !(all_attackers & occupied & their_pieces).empty() {
                    depth -= 1;
                }
                break;
            }
        }

        while depth - 1 > 0 {
            depth -= 1;
            gain[depth - 1] = -max(-gain[depth - 1], gain[depth]);
        }

        gain[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_see_1() {
        let mut engine = Engine::from_fen("1k2q3/1ppn3p/pr6/4b3/5B2/P2N2P1/1PP1Q2P/2K5 w - - 0 1");
        let r#move = Move::from("d3e5");
        let see = engine.see(r#move);
        assert_eq!(see, 330);
    }

    #[test]
    fn test_see_2() {
        let mut engine = Engine::from_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -");
        let r#move = Move::from("d3e5");
        let see = engine.see(r#move);
        assert_eq!(see, -220);
    }

    #[test]
    fn test_see_3() {
        let mut engine = Engine::from_fen("1kr2R2/1b6/8/5B2/8/8/8/1K6 w - - 0 1");
        let r#move = Move::from("f8c8");
        let see = engine.see(r#move);
        assert_eq!(see, 0);
    }

    #[test]
    fn test_see_4() {
        let mut engine =
            Engine::from_fen("rnbqk2r/pppp1ppp/8/2bQP3/4n3/5N2/PPP2PPP/RNB1KB1R b KQkq - 2 5");
        let r#move = Move::from("c5f2");
        let see = engine.see(r#move);
        assert_eq!(see, 100);
    }

    #[test]
    fn test_see_5() {
        let mut engine =
            Engine::from_fen("r3kbr1/1p1b1p2/4p3/p2pp3/1P2P3/P1NR1N1P/2P2P2/4K2R w Kq - 0 23");
        let r#move = Move::from("f3e5");
        let see = engine.see(r#move);
        println!("see: {}", see);
    }
}
