use crate::chess::Bitboard;
use crate::chess::Player;
use crate::chess::Square;

/// Represents the color of a chess piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    White,
    Black,
}

impl From<Player> for Color {
    fn from(player: Player) -> Self {
        match player {
            Player::White => Color::White,
            Player::Black => Color::Black,
        }
    }
}

impl Color {
    /// Convert a boolean to Color (true = White, false = Black).
    pub fn from_bool(is_white: bool) -> Self {
        if is_white { Color::White } else { Color::Black }
    }

    /// Convert Color to boolean (White = true, Black = false).
    pub fn to_bool(self) -> bool {
        matches!(self, Color::White)
    }

    /// Get the opposite color.
    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

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
    pub fn value(self) -> i32 {
        match self {
            Piece::Pawn => 1,
            Piece::Knight => 3,
            Piece::Bishop => 3,
            Piece::Rook => 5,
            Piece::Queen => 9,
            Piece::King => 0,
        }
    }

    /// Create a `(Piece, Color)` pair from a FEN board character.
    pub fn from_char(c: char) -> Option<(Self, Color)> {
        let color = if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let piece = match c.to_ascii_lowercase() {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => return None,
        };

        Some((piece, color))
    }

    /// Convert this piece type and color into a FEN board character.
    pub fn to_char(self, color: Color) -> char {
        let base = match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        };

        match color {
            Color::White => base.to_ascii_uppercase(),
            Color::Black => base,
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
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    kings: Bitboard,
    white_pieces: Bitboard,
    black_pieces: Bitboard,
    all_pieces: Bitboard,
    en_passant: Bitboard,
}

impl Pieces {
    pub fn new() -> Self {
        let mut pieces = Self::default();

        // White back rank
        pieces.set(Piece::Rook, Color::White, 0);
        pieces.set(Piece::Knight, Color::White, 1);
        pieces.set(Piece::Bishop, Color::White, 2);
        pieces.set(Piece::Queen, Color::White, 3);
        pieces.set(Piece::King, Color::White, 4);
        pieces.set(Piece::Bishop, Color::White, 5);
        pieces.set(Piece::Knight, Color::White, 6);
        pieces.set(Piece::Rook, Color::White, 7);

        // White pawns
        for square in 8..16 {
            pieces.set(Piece::Pawn, Color::White, square);
        }

        // Black pawns
        for square in 48..56 {
            pieces.set(Piece::Pawn, Color::Black, square);
        }

        // Black back rank
        pieces.set(Piece::Rook, Color::Black, 56);
        pieces.set(Piece::Knight, Color::Black, 57);
        pieces.set(Piece::Bishop, Color::Black, 58);
        pieces.set(Piece::Queen, Color::Black, 59);
        pieces.set(Piece::King, Color::Black, 60);
        pieces.set(Piece::Bishop, Color::Black, 61);
        pieces.set(Piece::Knight, Color::Black, 62);
        pieces.set(Piece::Rook, Color::Black, 63);

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
    pub fn set(&mut self, piece: Piece, color: Color, i: u8) {
        match color {
            Color::White => self.white_pieces.set(i),
            Color::Black => self.black_pieces.set(i),
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

    /// Clear the given piece type and color on square `i`.
    pub fn clear(&mut self, piece: Piece, color: Color, i: u8) {
        match color {
            Color::White => self.white_pieces.clear(i),
            Color::Black => self.black_pieces.clear(i),
        }
        self.all_pieces.clear(i);

        match piece {
            Piece::King => self.kings.clear(i),
            Piece::Queen => self.queens.clear(i),
            Piece::Rook => self.rooks.clear(i),
            Piece::Bishop => self.bishops.clear(i),
            Piece::Knight => self.knights.clear(i),
            Piece::Pawn => self.pawns.clear(i),
        }
    }

    /// Get the `(Piece, Color)` at index `i`, if any.
    pub fn get_piece(&self, i: u8) -> Option<(Piece, Color)> {
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

        let color = if self.white_pieces.get(i) {
            Color::White
        } else {
            Color::Black
        };
        Some((piece, color))
    }

    /// Convenience helper to get a FEN-style character for the board display.
    pub fn get_char(&self, i: u8) -> char {
        if let Some((piece, color)) = self.get_piece(i) {
            piece.to_char(color)
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
            let capturing_color = if self.white_pieces.get_square(from) {
                Color::White
            } else {
                Color::Black
            };
            match capturing_color {
                Color::White => self.black_pieces.clear_square(captured_square),
                Color::Black => self.white_pieces.clear_square(captured_square),
            }
        }

        self.en_passant.reset();
        if self.pawns.get_square(from) && (from.rank as i8 - to.rank as i8).abs() == 2 {
            self.en_passant.set((from.square + to.square) / 2);
        }
    }

    // Getters for all fields
    pub fn pawns(&self) -> Bitboard {
        self.pawns
    }

    pub fn knights(&self) -> Bitboard {
        self.knights
    }

    pub fn bishops(&self) -> Bitboard {
        self.bishops
    }

    pub fn rooks(&self) -> Bitboard {
        self.rooks
    }

    pub fn queens(&self) -> Bitboard {
        self.queens
    }

    pub fn kings(&self) -> Bitboard {
        self.kings
    }

    pub fn white_pieces(&self) -> Bitboard {
        self.white_pieces
    }

    pub fn black_pieces(&self) -> Bitboard {
        self.black_pieces
    }

    pub fn all_pieces(&self) -> Bitboard {
        self.all_pieces
    }

    pub fn en_passant(&self) -> Bitboard {
        self.en_passant
    }

    // Setter for en_passant (needed for undo)
    pub fn set_en_passant(&mut self, en_passant: Bitboard) {
        self.en_passant = en_passant;
    }

    // Set en_passant square (needed for FEN loading)
    pub fn set_en_passant_square(&mut self, square: Square) {
        self.en_passant.set_square(square);
    }
}
