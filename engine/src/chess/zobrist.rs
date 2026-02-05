use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::chess::castling::CastlingRights;
use crate::chess::chessboard::State;
use crate::chess::{Bitboard, ChessBoard, Color, Move, Piece, Player, Square};

#[derive(Clone)]
pub struct Zobrist {
    piece: [[[u64; 64]; 6]; 2],
    color: u64,
    castling_rights: [u64; 16],
    en_passant: [u64; 64],
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(67);

        let piece =
            std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| rng.random())));

        Self {
            piece,
            color: rng.random(),
            castling_rights: std::array::from_fn(|_| rng.random()),
            en_passant: std::array::from_fn(|_| rng.random()),
        }
    }

    /// Full hash of a position from piece placement, side to move, castling rights, and en passant.
    pub fn full_hash(&self, board: &ChessBoard) -> u64 {
        let mut hash = 0u64;
        for sq in 0..64u8 {
            let square = Square::from(sq);
            if let Some((piece, color)) = board.pieces().piece(square) {
                hash ^= self.piece[color as usize][piece as usize][sq as usize];
            }
        }
        if board.player() == Player::Black {
            hash ^= self.color;
        }
        hash ^= self.castling_rights[board.castling_rights().get() as usize];
        if !board.pieces().en_passant().empty() {
            hash ^= self.en_passant[board.pieces().en_passant().get_lsb() as usize];
        }
        hash
    }

    // TODO: Account for rook move in castling, promotion.
    /// Incremental update: new hash from prev_hash after making the given move.
    pub fn hash(
        &self,
        r#move: Move,
        moving_piece: (Piece, Color),
        castling_rights: CastlingRights,
        en_passant: Bitboard,
        prev_state: State,
        prev_hash: u64,
    ) -> u64 {
        let mut hash = prev_hash;
        hash ^= self.piece[moving_piece.1 as usize][moving_piece.0 as usize]
            [r#move.from.square as usize];

        if let Some((captured_piece, captured_color)) = prev_state.captured_piece {
            hash ^= self.piece[captured_color as usize][captured_piece as usize]
                [r#move.to.square as usize];
        }

        hash ^=
            self.piece[moving_piece.1 as usize][moving_piece.0 as usize][r#move.to.square as usize];

        hash ^= self.color;

        hash ^= self.castling_rights[prev_state.prev_castling_rights.get() as usize];
        hash ^= self.castling_rights[castling_rights.get() as usize];

        if !prev_state.prev_en_passant.empty() {
            hash ^= self.en_passant[prev_state.prev_en_passant.get_lsb() as usize];
        }
        if !en_passant.empty() {
            hash ^= self.en_passant[en_passant.get_lsb() as usize];
        }

        hash
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}
