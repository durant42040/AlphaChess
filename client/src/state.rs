//! Centralized app state and messages (TodoMVC-style).

use crate::utils::{to_board, Piece, STARTING_BOARD_STR};

/// All app state in one place.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub board: Vec<Vec<Option<Piece>>>,
    pub position_from: Option<[u8; 2]>,
    pub position_to: Option<[u8; 2]>,
    pub side: char,
    pub game: Option<char>,
    pub game_over: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            board: to_board(STARTING_BOARD_STR),
            position_from: None,
            position_to: None,
            side: 'w',
            game: None,
            game_over: "No".to_string(),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_board(&mut self) {
        self.board = to_board(STARTING_BOARD_STR);
        self.position_from = None;
        self.position_to = None;
        self.side = 'w';
        self.game_over = "No".to_string();
    }
}

/// All actions (message-driven updates like TodoMVC).
#[derive(Clone)]
pub enum Msg {
    ChooseSide(char),
    SquareClick([u8; 2]),
    DragStart([u8; 2]),
    Drop([u8; 2]),
    Rematch,
    Undo,
    PollGame,
    TryAct,
    GenerateMove,
    // API response messages
    ResetDone(Result<(), String>),
    ActDone(Result<(String, bool, bool), String>), // (board, is_check, was_capture)
    GenerateDone(Result<(String, bool, bool), String>), // (board, is_check, was_capture)
    UndoDone(Result<(String, bool), String>),       // (board, is_check)
    GameStateDone(Result<String, String>),          // game_state string
}
