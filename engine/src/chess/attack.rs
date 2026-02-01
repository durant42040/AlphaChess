use crate::chess::Bitboard;

#[derive(Clone, Default)]
pub struct AttackState {
    pub attackers: Bitboard,
    pub num_checks: u8,
    pub pinned_pieces: Bitboard,
    pub attack_lines: Bitboard,
}
