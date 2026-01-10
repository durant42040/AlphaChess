use crate::{
    bitboard::Bitboard,
    constants::{BLACK_PAWN_CAPTURES, KING_ATTACKS, KNIGHT_ATTACKS, WHITE_PAWN_CAPTURES},
    square::Square,
};

pub fn generate_white_pawn_moves(
    from: Square,
    all_pieces: Bitboard,
    capture_pieces: Bitboard,
) -> Bitboard {
    let from_mask = Bitboard::from(1 << from.square);

    let one_step_moves = (from_mask << 8) & !all_pieces;
    let two_step_moves = ((one_step_moves & (0xFF << 16).into()) << 8) & !all_pieces;
    let capture_moves = Bitboard::from(WHITE_PAWN_CAPTURES[from]) & capture_pieces;

    one_step_moves | two_step_moves | capture_moves
}

pub fn generate_black_pawn_moves(
    from: Square,
    all_pieces: Bitboard,
    capture_pieces: Bitboard,
) -> Bitboard {
    let from_mask = Bitboard::from(1 << from.square);

    let one_step_moves = (from_mask >> 8) & !all_pieces;
    let two_step_moves = ((one_step_moves & (0xFF << 40).into()) >> 8) & !all_pieces;
    let capture_moves = Bitboard::from(BLACK_PAWN_CAPTURES[from]) & capture_pieces;

    one_step_moves | two_step_moves | capture_moves
}

pub fn generate_knight_moves(from: Square) -> Bitboard {
    KNIGHT_ATTACKS[from].into()
}

pub fn generate_king_moves(from: Square) -> Bitboard {
    KING_ATTACKS[from].into()
}

#[inline]
fn slide(moves: &mut Bitboard, all_pieces: Bitboard, mut i: i32, mut j: i32, di: i32, dj: i32) {
    i += di;
    j += dj;
    while (0..8).contains(&i) && (0..8).contains(&j) {
        let sq = (i * 8 + j) as u8;
        moves.set(sq);
        if all_pieces.get(sq) {
            break;
        }
        i += di;
        j += dj;
    }
}

pub fn generate_bishop_moves_slow(from: Square, all_pieces: Bitboard) -> Bitboard {
    let rank = from.rank as i32;
    let file = from.file as i32;

    let mut moves = Bitboard::default();
    slide(&mut moves, all_pieces, rank, file, 1, 1);
    slide(&mut moves, all_pieces, rank, file, -1, 1);
    slide(&mut moves, all_pieces, rank, file, 1, -1);
    slide(&mut moves, all_pieces, rank, file, -1, -1);
    moves
}

pub fn generate_rook_moves_slow(from: Square, all_pieces: Bitboard) -> Bitboard {
    let file = from.file as i32;
    let rank = from.rank as i32;
    let mut moves = Bitboard::default();
    slide(&mut moves, all_pieces, rank, file, 1, 0);
    slide(&mut moves, all_pieces, rank, file, -1, 0);
    slide(&mut moves, all_pieces, rank, file, 0, 1);
    slide(&mut moves, all_pieces, rank, file, 0, -1);
    moves
}

pub fn generate_bishop_moves(from: Square, all_pieces: Bitboard) -> Bitboard {
    todo!()
}

pub fn generate_rook_moves(from: Square, all_pieces: Bitboard) -> Bitboard {
    todo!()
}

pub fn generate_queen_moves(from: Square, all_pieces: Bitboard) -> Bitboard {
    generate_rook_moves(from, all_pieces) | generate_bishop_moves(from, all_pieces)
}
