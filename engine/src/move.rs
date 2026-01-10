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
