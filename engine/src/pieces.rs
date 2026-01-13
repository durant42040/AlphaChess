use crate::bitboard::Bitboard;
use crate::square::Square;

/// Represents the type of a chess piece (color is handled separately).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    /// Create a `(Piece, is_white)` pair from a FEN board character.
    pub fn from_char(c: char) -> Option<(Self, bool)> {
        let is_white = c.is_uppercase();
        let piece = match c.to_ascii_lowercase() {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => return None,
        };

        Some((piece, is_white))
    }

    /// Convert this piece type and color into a FEN board character.
    pub fn to_char(self, is_white: bool) -> char {
        let base = match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        };

        if is_white {
            base.to_ascii_uppercase()
        } else {
            base
        }
    }

    /// Create a promotion piece type from a move string character (e.g. 'q', 'n').
    pub fn from_promotion_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'q' => Some(Piece::Queen),
            'r' => Some(Piece::Rook),
            'b' => Some(Piece::Bishop),
            'n' => Some(Piece::Knight),
            _ => None,
        }
    }

    /// Convert this piece type into a promotion character used in move strings.
    pub fn to_promotion_char(self) -> char {
        match self {
            Piece::Queen => 'q',
            Piece::Rook => 'r',
            Piece::Bishop => 'b',
            Piece::Knight => 'n',
            // Pawns and kings are never used as promotion pieces.
            Piece::Pawn | Piece::King => ' ',
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
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

        // White back rank
        pieces.set(Piece::Rook, true, 0);
        pieces.set(Piece::Knight, true, 1);
        pieces.set(Piece::Bishop, true, 2);
        pieces.set(Piece::Queen, true, 3);
        pieces.set(Piece::King, true, 4);
        pieces.set(Piece::Bishop, true, 5);
        pieces.set(Piece::Knight, true, 6);
        pieces.set(Piece::Rook, true, 7);

        // White pawns
        for square in 8..16 {
            pieces.set(Piece::Pawn, true, square);
        }

        // Black pawns
        for square in 48..56 {
            pieces.set(Piece::Pawn, false, square);
        }

        // Black back rank
        pieces.set(Piece::Rook, false, 56);
        pieces.set(Piece::Knight, false, 57);
        pieces.set(Piece::Bishop, false, 58);
        pieces.set(Piece::Queen, false, 59);
        pieces.set(Piece::King, false, 60);
        pieces.set(Piece::Bishop, false, 61);
        pieces.set(Piece::Knight, false, 62);
        pieces.set(Piece::Rook, false, 63);

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

    /// Set the given piece type and color on square `i`.
    pub fn set(&mut self, piece: Piece, is_white: bool, i: u8) {
        if is_white {
            self.white_pieces.set(i);
        } else {
            self.black_pieces.set(i);
        }
        self.all_pieces.set(i);

        match piece {
            Piece::King => self.kings.set(i),
            Piece::Queen => self.queens.set(i),
            Piece::Rook => self.rooks.set(i),
            Piece::Bishop => self.bishops.set(i),
            Piece::Knight => self.knights.set(i),
            Piece::Pawn => self.pawns.set(i),
        }
    }

    /// Get the `(Piece, is_white)` at index `i`, if any.
    pub fn get_piece(&self, i: u8) -> Option<(Piece, bool)> {
        let piece = if self.pawns.get(i) {
            Piece::Pawn
        } else if self.knights.get(i) {
            Piece::Knight
        } else if self.bishops.get(i) {
            Piece::Bishop
        } else if self.rooks.get(i) {
            Piece::Rook
        } else if self.queens.get(i) {
            Piece::Queen
        } else if self.kings.get(i) {
            Piece::King
        } else {
            return None;
        };

        let is_white = self.white_pieces.get(i);
        Some((piece, is_white))
    }

    /// Convenience helper to get a FEN-style character for the board display.
    pub fn get_char(&self, i: u8) -> char {
        if let Some((piece, is_white)) = self.get_piece(i) {
            piece.to_char(is_white)
        } else {
            '.'
        }
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

    pub fn promote(&mut self, promotion: Option<Piece>, from: Square) {
        let Some(promotion) = promotion else {
            return;
        };

        match promotion {
            Piece::Queen => self.queens.set_square(from),
            Piece::Rook => self.rooks.set_square(from),
            Piece::Bishop => self.bishops.set_square(from),
            Piece::Knight => self.knights.set_square(from),
            // Pawns and kings are never valid promotion targets.
            Piece::Pawn | Piece::King => {}
        }
        self.pawns.clear_square(from);
    }

    pub fn undo_promote(&mut self, to: Square) {
        self.pawns.set_square(to);
        self.queens.clear_square(to);
        self.rooks.clear_square(to);
        self.bishops.clear_square(to);
        self.knights.clear_square(to);
    }

    pub fn update_en_passant(&mut self, from: Square, to: Square) {
        if self.pawns.get_square(from) && self.en_passant.get_square(to) {
            let captured_square = Square::new(from.rank, to.file);
            self.pawns.clear_square(captured_square);
            self.all_pieces.clear_square(captured_square);
            if self.white_pieces.get_square(from) {
                self.black_pieces.clear_square(captured_square);
            } else {
                self.white_pieces.clear_square(captured_square);
            }
        }

        self.en_passant.reset();
        if self.pawns.get_square(from) && (from.rank as i8 - to.rank as i8).abs() == 2 {
            self.en_passant.set((from.square + to.square) / 2);
        }
    }

    pub fn undo_en_passant(&mut self, from: Square, to: Square) {}
}
