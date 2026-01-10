use std::{fmt, str::FromStr};

use crate::square::Square;

#[derive(Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<char>,
}

impl Move {
    pub fn new(from: Square, to: Square, promotion: char) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(p) = self.promotion {
            write!(f, "{}", p)?;
        }
        Ok(())
    }
}

impl FromStr for Move {
    type Err = ();

    fn from_str(move_string: &str) -> Result<Self, ()> {
        let from_string = &move_string[0..2];
        let to_string = &move_string[2..4];
        let promotion = move_string.chars().nth(4);

        let from = from_string.parse::<Square>().unwrap();
        let to = to_string.parse::<Square>().unwrap();

        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Move;
    use crate::square::Square;

    #[test]
    fn test_move() {
        let from = Square::new(1, 4);
        let to = Square::new(3, 4);
        let r#move = Move::new(from, to, 'q');
        let move_string = format!("{}", r#move);

        assert_eq!(r#move.from, from);
        assert_eq!(r#move.to, to);
        assert_eq!(r#move.promotion, Some('q'));
        assert_eq!(move_string, "e2e4q");
    }

    #[test]
    fn test_move_without_promotion() {
        let r#move = "e2e4".parse::<Move>().unwrap();
        let move_string = format!("{}", r#move);

        assert_eq!(r#move.from, "e2".parse::<Square>().unwrap());
        assert_eq!(r#move.to, "e4".parse::<Square>().unwrap());
        assert_eq!(r#move.promotion, None);
        assert_eq!(move_string, "e2e4");
    }

    #[test]
    fn test_move_with_promotion() {
        let r#move = "e7e8q".parse::<Move>().unwrap();
        let move_string = format!("{}", r#move);

        assert_eq!(r#move.from, "e7".parse::<Square>().unwrap());
        assert_eq!(r#move.to, "e8".parse::<Square>().unwrap());
        assert_eq!(r#move.promotion, Some('q'));
        assert_eq!(move_string, "e7e8q");
    }
}
