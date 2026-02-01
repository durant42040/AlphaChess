use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::chess::chessboard::{Castling, ChessBoard};
use crate::chess::game::Player;

pub struct Zobrist {
    piece: [[[u64; 64]; 6]; 2], // [color][piece][square]
    color: u64,
    castling_rights: [u64; 16],
    en_passant: [u64; 8],
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(42);

        let piece =
            std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| rng.random())));

        Self {
            piece,
            color: rng.random(),
            castling_rights: std::array::from_fn(|_| rng.random()),
            en_passant: std::array::from_fn(|_| rng.random()),
        }
    }

    /// Computes the Zobrist hash for the given board position.
    pub fn hash(&self, board: &ChessBoard) -> u64 {
        let pieces = board.pieces();
        let mut h = 0u64;

        for square in 0..64u8 {
            if let Some((piece, color)) = pieces.get_piece(square) {
                h ^= self.piece[color as usize][piece as usize][square as usize];
            }
        }

        if board.player() == Player::Black {
            h ^= self.color;
        }

        h ^= self.castling_rights[board.castling_rights().get() as usize];

        if !pieces.en_passant().empty() {
            let ep_square = pieces.en_passant().bitboard.trailing_zeros() as u8;
            let ep_file = ep_square % 8;
            h ^= self.en_passant[ep_file as usize];
        }

        h
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}
