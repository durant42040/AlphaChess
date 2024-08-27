#pragma once

#include "bitboard.h"
#include "square.h"

extern const Bitboard white_pawn_captures[64];
extern const Bitboard black_pawn_captures[64];
extern const Bitboard knight_attacks[64];
extern const Bitboard king_attacks[64];
extern const Bitboard bishop_masks[64];
extern const Bitboard rook_masks[64];
extern Bitboard bishop_table[64][1024];
extern Bitboard rook_table[64][4096];
extern const int rook_shift_bits[64];
extern const int bishop_shift_bits[64];

uint64_t get_blocker(int index, Bitboard mask);

void init_sliding_moves();

Bitboard generate_white_pawn_moves(Square from, Bitboard all_pieces,
                                   Bitboard capture_pieces);

Bitboard generate_black_pawn_moves(Square from, Bitboard all_pieces,
                                   Bitboard capture_pieces);

Bitboard generate_knight_moves(Square from);

Bitboard generate_king_moves(Square from);

Bitboard generate_rook_moves(Square from, Bitboard all_pieces);

Bitboard generate_bishop_moves(Square from, Bitboard all_pieces);

Bitboard generate_queen_moves(Square from, Bitboard all_pieces);

Bitboard generate_rook_moves_slow(Square from, Bitboard all_pieces);

Bitboard generate_bishop_moves_slow(Square from, Bitboard all_pieces);
