use crate::bitboard::Bitboard;
use crate::square::Square;

#[derive(Default, Copy, Clone)]
pub struct Pieces {
    pub pawns: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub kings: Bitboard,
    pub white_pieces: Bitboard,
    pub black_pieces: Bitboard,
    pub all_pieces: Bitboard,
    pub en_passant: Bitboard,
}

impl Pieces {
    pub fn new() -> Self {
        let mut pieces = Self::default();
        pieces.set('R', 0);
        pieces.set('N', 1);
        pieces.set('B', 2);
        pieces.set('Q', 3);
        pieces.set('K', 4);
        pieces.set('B', 5);
        pieces.set('N', 6);
        pieces.set('R', 7);

        for square in 8..16 {
            pieces.set('P', square);
        }

        for square in 48..56 {
            pieces.set('p', square);
        }

        pieces.set('r', 56);
        pieces.set('n', 57);
        pieces.set('b', 58);
        pieces.set('q', 59);
        pieces.set('k', 60);
        pieces.set('b', 61);
        pieces.set('n', 62);
        pieces.set('r', 63);

        pieces
    }

    pub fn update(&mut self, from: Square, to: Square) {
        self.pawns.update(from, to);
        self.knights.update(from, to);
        self.bishops.update(from, to);
        self.rooks.update(from, to);
        self.queens.update(from, to);
        self.kings.update(from, to);
        self.white_pieces.update(from, to);
        self.black_pieces.update(from, to);
        self.all_pieces.update(from, to);
    }

    pub fn set(&mut self, c: char, i: u8) {
        if c.is_uppercase() {
            self.white_pieces.set(i);
        } else {
            self.black_pieces.set(i);
        }
        self.all_pieces.set(i);

        match c {
            'k' | 'K' => self.kings.set(i),
            'q' | 'Q' => self.queens.set(i),
            'r' | 'R' => self.rooks.set(i),
            'b' | 'B' => self.bishops.set(i),
            'n' | 'N' => self.knights.set(i),
            'p' | 'P' => self.pawns.set(i),
            _ => {}
        }
    }

    pub fn get(&self, i: u8) -> char {
        let mut c = '.';
        if self.pawns.get(i) {
            c = 'p';
        } else if self.knights.get(i) {
            c = 'n';
        } else if self.bishops.get(i) {
            c = 'b';
        } else if self.rooks.get(i) {
            c = 'r';
        } else if self.queens.get(i) {
            c = 'q';
        } else if self.kings.get(i) {
            c = 'k';
        }

        if self.white_pieces.get(i) {
            c = c.to_ascii_uppercase();
        }

        c
    }
}
