use std::fmt;

use crate::bitboard::Bitboard;
use crate::square::Square;

enum GameState {
    Playing,
    WhiteWin,
    BlackWin,
    Draw,
}

enum Player {
    White,
    Black,
}

pub struct ChessBoard {
    game_state: GameState,
    player: Player,
    position_hash_history: Vec<u64>,
    fifty_move_rule: u8,
    fullmove_number: u8,
    castling_rights: u8,
    en_passant: Bitboard,

    all_pieces: Bitboard,
    white_pieces: Bitboard,
    black_pieces: Bitboard,

    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    kings: Bitboard,
}

impl ChessBoard {
    pub fn new() -> Self {
        let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::from_fen(starting_fen)
    }

    pub fn from_fen(fen: &str) -> Self {
        let mut parts = fen.split_whitespace();

        let mut all_pieces = Bitboard::new(0);
        let mut white_pieces = Bitboard::new(0);
        let mut black_pieces = Bitboard::new(0);
        let mut pawns = Bitboard::new(0);
        let mut knights = Bitboard::new(0);
        let mut bishops = Bitboard::new(0);
        let mut rooks = Bitboard::new(0);
        let mut queens = Bitboard::new(0);
        let mut kings = Bitboard::new(0);

        let board = parts.next().unwrap_or("");
        let mut rank = 7;
        let mut file = 0;

        for c in board.chars() {
            let i: u8 = rank * 8 + file;

            if c == '/' {
                rank -= 1;
                file = 0;
                continue;
            }

            if c.is_ascii_digit() {
                file += c.to_digit(10).unwrap() as u8;
                continue;
            }

            if c.is_uppercase() {
                white_pieces.set(i);
            } else {
                black_pieces.set(i);
            }
            all_pieces.set(i);

            match c {
                'k' | 'K' => kings.set(i),
                'q' | 'Q' => queens.set(i),
                'r' | 'R' => rooks.set(i),
                'b' | 'B' => bishops.set(i),
                'n' | 'N' => knights.set(i),
                'p' | 'P' => pawns.set(i),
                _ => {}
            }

            file += 1;
        }

        let player_str = parts.next().unwrap_or("w");
        let player = if player_str == "w" {
            Player::White
        } else {
            Player::Black
        };

        let castling_str = parts.next().unwrap_or("-");
        let mut castling_rights = 0u8;
        for c in castling_str.chars() {
            match c {
                'K' => castling_rights |= 1,
                'Q' => castling_rights |= 2,
                'k' => castling_rights |= 4,
                'q' => castling_rights |= 8,
                _ => {}
            }
        }

        // Parse en passant square (fourth part)
        let en_passant_str = parts.next().unwrap_or("-");
        let en_passant = if en_passant_str != "-" {
            let square = Square::from_string(en_passant_str.to_string());
            let mut ep_bitboard = Bitboard::new(0);
            ep_bitboard.set_square(square);
            ep_bitboard
        } else {
            Bitboard::new(0)
        };

        let fifty_move_rule = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

        let fullmove_number = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(1);

        Self {
            game_state: GameState::Playing,
            player,
            position_hash_history: Vec::new(),
            fifty_move_rule,
            fullmove_number,
            castling_rights,
            en_passant,
            all_pieces,
            white_pieces,
            black_pieces,
            pawns,
            knights,
            bishops,
            rooks,
            queens,
            kings,
        }
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board = String::new();

        for i in 0..64 {
            let mut piece_char = '.';

            if self.pawns.get(i) {
                piece_char = 'p';
            } else if self.knights.get(i) {
                piece_char = 'n';
            } else if self.bishops.get(i) {
                piece_char = 'b';
            } else if self.rooks.get(i) {
                piece_char = 'r';
            } else if self.queens.get(i) {
                piece_char = 'q';
            } else if self.kings.get(i) {
                piece_char = 'k';
            }

            if self.white_pieces.get(i) {
                piece_char = piece_char.to_ascii_uppercase();
            }

            board.push(piece_char);

            if i % 8 == 7 {
                board.push('\n');
            } else {
                board.push(' ');
            }
        }

        for rank in (0..8).rev() {
            write!(f, "{}  ", rank + 1)?;
            let start = rank * 16;
            let end = start + 16;
            write!(f, "{}", &board[start..end])?;
        }

        write!(f, "   a b c d e f g h\n\n")
    }
}
