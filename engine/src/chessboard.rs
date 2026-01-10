use std::fmt;

use crate::bitboard::Bitboard;
use crate::r#move::Move;
use crate::move_generator::MoveGenerator;
use crate::square::Square;

#[derive(Default, Copy, Clone)]
pub enum GameState {
    #[default]
    Playing,
    WhiteWin,
    BlackWin,
    Draw,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameState::Playing => "playing",
                GameState::WhiteWin => "checkmate",
                GameState::BlackWin => "checkmate",
                GameState::Draw => "draw",
            }
        )
    }
}

#[derive(Default)]
pub enum Player {
    #[default]
    White,
    Black,
}

#[derive(Default)]
pub struct ChessBoard {
    move_generator: MoveGenerator,
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
        let mut chessboard = Self::default();
        let mut parts = fen.split_whitespace();

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
                chessboard.white_pieces.set(i);
            } else {
                chessboard.black_pieces.set(i);
            }
            chessboard.all_pieces.set(i);

            match c {
                'k' | 'K' => chessboard.kings.set(i),
                'q' | 'Q' => chessboard.queens.set(i),
                'r' | 'R' => chessboard.rooks.set(i),
                'b' | 'B' => chessboard.bishops.set(i),
                'n' | 'N' => chessboard.knights.set(i),
                'p' | 'P' => chessboard.pawns.set(i),
                _ => {}
            }

            file += 1;
        }

        let player_str = parts.next().unwrap_or("w");
        if player_str == "b" {
            chessboard.player = Player::Black;
        }

        let castling_str = parts.next().unwrap_or("-");
        chessboard.castling_rights = 0u8;
        for c in castling_str.chars() {
            match c {
                'K' => chessboard.castling_rights |= 1,
                'Q' => chessboard.castling_rights |= 2,
                'k' => chessboard.castling_rights |= 4,
                'q' => chessboard.castling_rights |= 8,
                _ => {}
            }
        }

        let en_passant_str = parts.next().unwrap_or("-");
        if en_passant_str != "-" {
            let square = en_passant_str.parse::<Square>().unwrap();
            chessboard.en_passant.set_square(square);
        }

        chessboard.fifty_move_rule = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
        chessboard.fullmove_number = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(1);

        chessboard
    }

    pub fn act(&mut self, r#move: Move) -> bool {
        if !self.is_legal_move(r#move) {
            return false;
        }

        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;

        self.pawns.update(from, to);
        self.knights.update(from, to);
        self.bishops.update(from, to);
        self.rooks.update(from, to);
        self.queens.update(from, to);
        self.kings.update(from, to);
        self.white_pieces.update(from, to);
        self.black_pieces.update(from, to);
        self.all_pieces.update(from, to);

        self.player = match self.player {
            Player::White => Player::Black,
            Player::Black => Player::White,
        };

        true
    }

    pub fn is_legal_move(&self, r#move: Move) -> bool {
        todo!()
    }

    pub fn get_game_state(&self) -> String {
        self.game_state.to_string()
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

#[cfg(test)]
mod tests {
    use super::ChessBoard;

    #[test]
    fn test_chessboard_to_string() {
        let board: ChessBoard = ChessBoard::new();
        let board_string = format!("{}", board);

        let expected = r#"8  r n b q k b n r
7  p p p p p p p p
6  . . . . . . . .
5  . . . . . . . .
4  . . . . . . . .
3  . . . . . . . .
2  P P P P P P P P
1  R N B Q K B N R
   a b c d e f g h

"#;

        assert_eq!(board_string, expected);
    }
}
