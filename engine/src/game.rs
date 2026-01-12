use std::fmt;

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

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Player {
    #[default]
    White,
    Black,
}

impl Player {
    pub fn switch(&self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}
