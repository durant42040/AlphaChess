use std::{fmt, str::FromStr};

#[derive(PartialEq, Copy, Clone, Debug)]
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
