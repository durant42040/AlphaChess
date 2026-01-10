use std::fmt;

use crate::bitboard::Bitboard;
use crate::game::Player;
use crate::r#move::Move;
use crate::pieces::Pieces;

#[derive(Default)]
pub struct ChessBoard {
    position_hash_history: Vec<u64>,
    fifty_move_rule: u8,
    fullmove_number: u8,
    castling_rights: u8,
    player: Player,
    pieces: Pieces,
}

impl ChessBoard {
    pub fn new() -> Self {
        let pieces = Pieces::new();
        let player = Player::White;

        Self {
            position_hash_history: Vec::new(),
            fifty_move_rule: 0,
            fullmove_number: 1,
            castling_rights: 0b1111,
            player,
            pieces,
        }
    }

    pub fn act(&mut self, r#move: Move) {
        let from = r#move.from;
        let to = r#move.to;
        let promotion = r#move.promotion;

        self.pieces.update(from, to);
        self.player = self.player.switch();
    }

    pub fn get_pieces(&self) -> Pieces {
        self.pieces
    }

    pub fn get_our_pieces(&self) -> Bitboard {
        if self.player == Player::White {
            self.pieces.white_pieces
        } else {
            self.pieces.black_pieces
        }
    }

    pub fn is_check(&self) -> bool {
        self.is_player_in_check(self.player)
    }

    pub fn is_player_in_check(&self, player: Player) -> bool {
        todo!();
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board = String::new();

        for i in 0..64 {
            let c = self.pieces.get(i);

            board.push(c);

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
