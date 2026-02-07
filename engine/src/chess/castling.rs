use crate::{
    Engine,
    chess::{Bitboard, Square},
    constants::*,
};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct CastlingRights {
    bits: u8,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            bits: ALL_CASTLING_RIGHTS,
        }
    }

    pub fn get(&self) -> u8 {
        self.bits
    }

    #[inline]
    pub fn can(&self, rights: u8) -> bool {
        (self.bits & rights) != 0
    }

    #[inline]
    pub fn set(&mut self, rights: u8) {
        self.bits |= rights;
    }

    #[inline]
    pub fn revoke(&mut self, rights: u8) {
        self.bits &= !rights;
    }

    #[inline]
    pub fn revoke_castling_rights(&mut self, from: Square, to: Square) {
        // Revoke white rook castling rights when the a1 or h1 rook moves or is captured.
        if from == WHITE_QUEENSIDE_ROOK_START || to == WHITE_QUEENSIDE_ROOK_START {
            self.revoke(WHITE_CASTLE_QUEENSIDE);
        } else if from == WHITE_KINGSIDE_ROOK_START || to == WHITE_KINGSIDE_ROOK_START {
            self.revoke(WHITE_CASTLE_KINGSIDE);
        }

        // Revoke white king castling rights when the king moves or is captured.
        if from == WHITE_KING_START || to == WHITE_KING_START {
            self.revoke(WHITE_CASTLE_KINGSIDE | WHITE_CASTLE_QUEENSIDE);
        }

        // Revoke black rook castling rights when the a8 or h8 rook moves or is captured.
        if from == BLACK_QUEENSIDE_ROOK_START || to == BLACK_QUEENSIDE_ROOK_START {
            self.revoke(BLACK_CASTLE_QUEENSIDE);
        } else if from == BLACK_KINGSIDE_ROOK_START || to == BLACK_KINGSIDE_ROOK_START {
            self.revoke(BLACK_CASTLE_KINGSIDE);
        }

        // Revoke black king castling rights when the king moves or is captured.
        if from == BLACK_KING_START || to == BLACK_KING_START {
            self.revoke(BLACK_CASTLE_KINGSIDE | BLACK_CASTLE_QUEENSIDE);
        }
    }
}

impl Engine {
    pub fn generate_castling_moves(&self, from: Square) -> Bitboard {
        let pieces = self.pieces();
        let mut castling_moves = Bitboard::zero();

        match from.square {
            WHITE_KING_START => {
                // White king's castle
                let mut is_kingside_attacked = false;
                for idx in Bitboard::from(WHITE_KINGSIDE_SQUARES).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_kingside_attacked = true;
                        break;
                    }
                }
                let can_kingside = !is_kingside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(WHITE_KINGSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(WHITE_CASTLE_KINGSIDE);

                let mut is_queenside_attacked = false;
                for idx in Bitboard::from(WHITE_QUEENSIDE_ATTACKED).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_queenside_attacked = true;
                        break;
                    }
                }
                let can_queenside = !is_queenside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(WHITE_QUEENSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(WHITE_CASTLE_QUEENSIDE);

                if can_kingside {
                    castling_moves.set(WHITE_KINGSIDE_CASTLE_TO);
                }
                if can_queenside {
                    castling_moves.set(WHITE_QUEENSIDE_CASTLE_TO);
                }
            }
            BLACK_KING_START => {
                // Black king's castle
                let mut is_kingside_attacked = false;
                for idx in Bitboard::from(BLACK_KINGSIDE_SQUARES).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_kingside_attacked = true;
                        break;
                    }
                }
                let can_kingside = !is_kingside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(BLACK_KINGSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(BLACK_CASTLE_KINGSIDE);

                let mut is_queenside_attacked = false;
                for idx in Bitboard::from(BLACK_QUEENSIDE_ATTACKED).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_queenside_attacked = true;
                        break;
                    }
                }
                let can_queenside = !is_queenside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(BLACK_QUEENSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(BLACK_CASTLE_QUEENSIDE);

                if can_kingside {
                    castling_moves.set(BLACK_KINGSIDE_CASTLE_TO);
                }
                if can_queenside {
                    castling_moves.set(BLACK_QUEENSIDE_CASTLE_TO);
                }
            }
            _ => {}
        }

        castling_moves
    }
}
