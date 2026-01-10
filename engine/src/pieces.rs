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

    pub fn get_char(&self, i: u8) -> char {
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
