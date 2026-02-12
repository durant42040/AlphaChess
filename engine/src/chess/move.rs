use std::{fmt, str::FromStr};

use arrayvec::ArrayVec;

use crate::chess::Piece;
use crate::chess::Square;
use crate::constants::MAX_LEGAL_MOVES;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move(pub u16);

impl Move {
    pub fn new(from: Square, to: Square, promotion: Option<Piece>) -> Self {
        let from = from.square as u16;
        let to = to.square as u16;
        let promotion: u16 = match promotion {
            Some(Piece::Knight) => 1,
            Some(Piece::Bishop) => 2,
            Some(Piece::Rook) => 3,
            Some(Piece::Queen) => 4,
            _ => 0,
        };

        Self(from | to << 6 | promotion << 12)
    }

    #[inline(always)]
    pub fn from(self) -> Square {
        Square::from((self.0 & 0b111111) as u8)
    }

    #[inline(always)]
    pub fn to(self) -> Square {
        Square::from(((self.0 >> 6) & 0b111111) as u8)
    }

    #[inline(always)]
    pub fn promotion(self) -> Option<Piece> {
        match (self.0 >> 12) & 0b1111 {
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn from_string(move_string: &str) -> Self {
        move_string.parse::<Move>().unwrap()
    }

    #[inline(always)]
    pub fn none() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;
        if let Some(p) = self.promotion() {
            debug_assert!(
                p != Piece::Pawn && p != Piece::King,
                "Pawn and king cannot be promoted"
            );
            write!(f, "{}", char::from(p))?;
        }
        Ok(())
    }
}

impl FromStr for Move {
    type Err = ();

    fn from_str(move_string: &str) -> Result<Self, ()> {
        // Move string must be at least 4 characters (e.g., "e2e4")
        if move_string.len() < 4 {
            return Err(());
        }

        let from_string = &move_string[0..2];
        let to_string = &move_string[2..4];
        let promotion_char = move_string.chars().nth(4);
        let promotion = promotion_char.and_then(Piece::from_promotion_char);

        let from = from_string.parse::<Square>()?;
        let to = to_string.parse::<Square>()?;

        Ok(Self::new(from, to, promotion))
    }
}

pub type MoveList = ArrayVec<Move, MAX_LEGAL_MOVES>;

#[cfg(test)]
mod tests {
    use super::Move;
    use crate::chess::Piece;
    use crate::chess::Square;

    #[test]
    fn test_move() {
        let from = Square::new(1, 4);
        let to = Square::new(3, 4);
        let r#move = Move::new(from, to, Some(Piece::Queen));
        let move_string = format!("{}", r#move);

        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.promotion(), Some(Piece::Queen));
        assert_eq!(move_string, "e2e4q");
    }

    #[test]
    fn test_move_without_promotion() {
        let r#move = Move::from_string("e2e4");
        let move_string = format!("{}", r#move);

        assert_eq!(r#move.from(), "e2".parse::<Square>().unwrap());
        assert_eq!(r#move.to(), "e4".parse::<Square>().unwrap());
        assert_eq!(r#move.promotion(), None);
        assert_eq!(move_string, "e2e4");
    }

    #[test]
    fn test_move_with_promotion() {
        let r#move = Move::from_string("e7e8q");
        let move_string = format!("{}", r#move);

        assert_eq!(r#move.from(), "e7".parse::<Square>().unwrap());
        assert_eq!(r#move.to(), "e8".parse::<Square>().unwrap());
        assert_eq!(r#move.promotion(), Some(Piece::Queen));
        assert_eq!(move_string, "e7e8q");
    }
}
