use std::fmt;
use std::ops::Not;

use crate::chess::Color;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
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

#[repr(u8)]
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Player {
    #[default]
    White,
    Black,
}

impl Player {
    #[inline(always)]
    pub fn color(self) -> Color {
        match self {
            Player::White => Color::White,
            Player::Black => Color::Black,
        }
    }
}

impl Not for Player {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}
