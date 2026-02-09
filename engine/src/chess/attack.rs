use crate::{
    Engine,
    chess::{Bitboard, Color, Square},
    constants::{BLACK_PAWN_CAPTURES, WHITE_PAWN_CAPTURES},
};

#[derive(Default, Clone, Copy)]
pub struct AttackState {
    pub attackers: Bitboard,
    pub num_checks: u8,
    pub pinned_pieces: Bitboard,
    pub attack_lines: Bitboard,
}

impl Engine {
    /// Checks if the given square is under attack by the opponent.
    pub fn is_under_attack(&self, square: Square, all_pieces: Bitboard, color: Color) -> bool {
        self.generate_attacks(square, all_pieces, color).count() > 0
    }

    /// Checks if the given square is under attack by the opponent in the current position.
    pub fn is_square_under_attack(&self, square: Square) -> bool {
        let pieces = self.pieces();
        self.is_under_attack(square, pieces.all_pieces(), self.board.player().into())
    }

    /// returns all attackers to the given square
    pub fn generate_attacks(&self, square: Square, all_pieces: Bitboard, color: Color) -> Bitboard {
        let pieces = self.pieces();
        let their_pieces = if color == Color::White {
            pieces.black_pieces()
        } else {
            pieces.white_pieces()
        };

        (self.move_generator.generate_king_moves(square) & (pieces.kings() & their_pieces))
            | (self.move_generator.generate_rook_moves(square, all_pieces)
                & (pieces.rooks() & their_pieces))
            | (self
                .move_generator
                .generate_bishop_moves(square, all_pieces)
                & (pieces.bishops() & their_pieces))
            | (self.move_generator.generate_queen_moves(square, all_pieces)
                & (pieces.queens() & their_pieces))
            | (self.move_generator.generate_knight_moves(square)
                & (pieces.knights() & their_pieces))
            | (if color == Color::White {
                Bitboard::from(WHITE_PAWN_CAPTURES[square]) & (pieces.pawns() & their_pieces)
            } else {
                Bitboard::from(BLACK_PAWN_CAPTURES[square]) & (pieces.pawns() & their_pieces)
            })
    }

    pub fn update_attack_state(&mut self) {
        let mut pinned_pieces = Bitboard::zero();
        let pieces = self.pieces();
        let our_pieces = self.board.our_pieces();
        let their_pieces = self.board.their_pieces();
        let our_king = Square::from(our_pieces & pieces.kings());

        let rook_rays = self
            .move_generator
            .generate_rook_moves(our_king, Bitboard::zero());
        let bishop_rays = self
            .move_generator
            .generate_bishop_moves(our_king, Bitboard::zero());

        let mut snipers = their_pieces
            & ((pieces.queens() | pieces.rooks()) & rook_rays
                | (pieces.queens() | pieces.bishops()) & bishop_rays);

        let mut attack_lines = Bitboard::zero();
        while !snipers.empty() {
            let sniper = snipers.pop_lsb();
            let blockers = pieces.all_pieces() & Bitboard::between(Square::from(sniper), our_king);
            if blockers.count() == 1 && our_pieces.intersects(blockers) {
                pinned_pieces |= blockers;
            } else if blockers.count() == 0 {
                // If no blockers, this is a check from a sliding piece
                attack_lines |= Bitboard::between(Square::from(sniper), our_king);
            }
        }

        let attackers =
            self.generate_attacks(our_king, pieces.all_pieces(), self.board.player().into());
        let num_checks = attackers.count();
        debug_assert!(num_checks <= 2);

        let attack_state = AttackState {
            attackers,
            num_checks,
            pinned_pieces,
            attack_lines,
        };

        self.attack_states.push(attack_state);
    }
}
