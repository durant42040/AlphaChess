use crate::square::Square;

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

    pub fn from_string(&self, move_string: String) -> Self {
        let from_string = move_string.chars().take(2).collect();
        let to_string = move_string.chars().skip(2).take(2).collect();
        let promotion = move_string.chars().nth(4);

        let from = Square::from_string(from_string);
        let to = Square::from_string(to_string);

        Self {
            from,
            to,
            promotion,
        }
    }

    pub fn to_string(&self) -> String {
        let mut move_string = format!("{}{}", self.from.to_string(), self.to.to_string());
        if self.promotion.is_some() {
            move_string.push(self.promotion.unwrap());
        }
        move_string
    }
}
