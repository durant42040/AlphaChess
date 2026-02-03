use std::{fmt, ops::Index, str::FromStr};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Square {
    pub rank: u8,
    pub file: u8,
    pub square: u8,
}

impl Square {
    pub fn new(rank: u8, file: u8) -> Self {
        Self {
            rank,
            file,
            square: rank * 8 + file,
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (self.file + b'a') as char,
            (self.rank + b'1') as char
        )
    }
}

impl From<u8> for Square {
    fn from(square: u8) -> Self {
        Self {
            rank: square / 8,
            file: square % 8,
            square,
        }
    }
}

impl FromStr for Square {
    type Err = ();

    fn from_str(square_string: &str) -> Result<Self, ()> {
        let bytes = square_string.as_bytes();
        if bytes.len() != 2 {
            return Err(());
        }
        Ok(Self::new(bytes[1] - b'1', bytes[0] - b'a'))
    }
}

impl Index<Square> for [u64; 64] {
    type Output = u64;
    fn index(&self, sq: Square) -> &Self::Output {
        &self[sq.square as usize]
    }
}

impl Index<Square> for Vec<Vec<u64>> {
    type Output = Vec<u64>;
    fn index(&self, s: Square) -> &Self::Output {
        &self[s.square as usize]
    }
}

impl PartialEq<u8> for Square {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.square == *other
    }
}

#[cfg(test)]
mod tests {
    use super::Square;

    #[test]
    fn test_square_new() {
        let square = Square::new(0, 0);
        let square_string = format!("{}", square);

        assert_eq!(square_string, "a1");
    }

    #[test]
    fn test_square_from_string() {
        let square = "e4".parse::<Square>().unwrap();
        let square_string = format!("{}", square);

        assert_eq!(square_string, "e4");
    }
}
