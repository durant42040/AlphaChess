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
        self.pawns.update_square(from, to);
        self.knights.update_square(from, to);
        self.bishops.update_square(from, to);
        self.rooks.update_square(from, to);
        self.queens.update_square(from, to);
        self.kings.update_square(from, to);
        self.white_pieces.update_square(from, to);
        self.black_pieces.update_square(from, to);
        self.all_pieces.update_square(from, to);
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

    pub fn has_mating_material(&self) -> bool {
        if !self.pawns.empty() || !self.rooks.empty() || !self.queens.empty() {
            return true;
        }

        let num_white_bishops = (self.bishops & self.white_pieces).count();
        let num_black_bishops = (self.bishops & self.black_pieces).count();
        let num_white_knights = (self.knights & self.white_pieces).count();
        let num_black_knights = (self.knights & self.black_pieces).count();

        num_white_bishops + num_white_knights > 1 || num_black_bishops + num_black_knights > 1
    }

    pub fn promote(&mut self, promotion: Option<char>, to: Square) {
        if promotion.is_none() {
            return;
        }
        let promotion = promotion.unwrap();
        match promotion {
            'q' => self.queens.set_square(to),
            'r' => self.rooks.set_square(to),
            'b' => self.bishops.set_square(to),
            'n' => self.knights.set_square(to),
            _ => {}
        }
        self.pawns.clear_square(to);
    }

    pub fn update_en_passant(&mut self, from: Square, to: Square) {
        if self.pawns.get_square(from) && self.en_passant.get_square(to) {
            let captured_square = if self.white_pieces.get_square(from) {
                to.square - 8
            } else {
                to.square + 8
            };
            self.pawns.clear(captured_square);
            self.all_pieces.clear(captured_square);
            if self.white_pieces.get_square(from) {
                self.black_pieces.clear(captured_square);
            } else {
                self.white_pieces.clear(captured_square);
            }
        }

        self.en_passant.reset();
        if self.pawns.get_square(from) && (from.rank as i8 - to.rank as i8).abs() == 2 {
            self.en_passant.set((from.square + to.square) / 2);
        }
    }
}
