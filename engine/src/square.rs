#[derive(PartialEq, Copy, Clone)]
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

    pub fn from_string(square_string: String) -> Self {
        let rank = square_string.chars().nth(1).unwrap() as u8 - '1' as u8;
        let file = square_string.chars().nth(0).unwrap() as u8 - 'a' as u8;
        Self {
            rank,
            file,
            square: rank * 8 + file,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", self.file as char, self.rank as char)
    }
}
