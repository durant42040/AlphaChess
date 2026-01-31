/// Board is row-major: board[rank][file], rank 0 = bottom (row 8), file 0 = a.
/// Piece: type is lowercase (k,q,r,b,n,p), color is 'w' or 'b'.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub piece_type: char,
    pub color: char,
}

/// Parse 64-char board string (row-major from top-left = a8) into 8x8.
/// Same logic as client/src/utils.js toBoard.
pub fn to_board(board_str: &str) -> Vec<Vec<Option<Piece>>> {
    let mut board = vec![vec![None; 8]; 8];
    for (index, ch) in board_str.chars().enumerate() {
        if ch == '.' {
            continue;
        }
        let row = 7 - (index / 8);
        let col = index % 8;
        let piece_type = ch.to_lowercase().next().unwrap_or(ch);
        let color = if ch.is_lowercase() { 'b' } else { 'w' };
        board[row][col] = Some(Piece { piece_type, color });
    }
    board
}

pub fn to_move_string(from: [u8; 2], to: [u8; 2]) -> String {
    let file = |c: u8| (b'a' + c) as char;
    let rank = |r: u8| (8 - r).to_string();
    format!(
        "{}{}{}{}",
        file(from[1]),
        rank(from[0]),
        file(to[1]),
        rank(to[0])
    )
}

pub fn is_equal(a: [u8; 2], b: [u8; 2]) -> bool {
    a[0] == b[0] && a[1] == b[1]
}

/// Starting position board string (same as React client).
pub const STARTING_BOARD_STR: &str =
    "RNBQKBNRPPPPPPPP................................pppppppprnbqkbnr";
