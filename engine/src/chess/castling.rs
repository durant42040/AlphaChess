use crate::chess::{
    constants::{
        ALL_CASTLING_RIGHTS, BLACK_CASTLE_KINGSIDE, BLACK_CASTLE_QUEENSIDE, BLACK_KING_START,
        BLACK_KINGSIDE_ROOK_START, BLACK_QUEENSIDE_ROOK_START, WHITE_CASTLE_KINGSIDE,
        WHITE_CASTLE_QUEENSIDE, WHITE_KING_START, WHITE_KINGSIDE_ROOK_START,
        WHITE_QUEENSIDE_ROOK_START,
    },
    square::Square,
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
