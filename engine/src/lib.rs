pub fn init() {
}

pub fn reset() {
}

pub fn act(_move_string: String) -> bool {
    true
}

pub fn get_game_state() -> String {
    "playing".to_string()
}

pub fn is_check() -> bool {
    false
}

pub fn is_legal_move(_move_string: String) -> bool {
    true
}

pub fn get_legal_moves() -> Vec<String> {
    vec!["e2e4".to_string(), "e2e3".to_string()]
}

pub fn get_board() -> String {
    "RNBQKBNRPPPPPPPP................................pppppppprnbqkbnr".to_string()
}