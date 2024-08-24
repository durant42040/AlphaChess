#pragma once

#include "bitboard.h"
#include "move.h"
#include "move_generator.h"
#include <iostream>
#include <string>
#include <vector>

extern uint64_t piece_keys[2][6][64];
extern uint64_t castle_keys[4];
extern uint64_t en_passant_keys[8];
extern uint64_t white_to_move_key;

const uint64_t white_kingside_squares = 0x0000000000000070;
const uint64_t white_queenside_squares = 0x000000000000001C;
const uint64_t black_kingside_squares = 0x7000000000000000;
const uint64_t black_queenside_squares = 0x1C00000000000000;
const std::string starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

enum class GameState {
    Playing,
    WhiteWin,
    BlackWin,
    Draw,
};

enum class Player {
    White,
    Black
};

void init_keys();

class ChessBoard {
public:
    ChessBoard() : game_state_(GameState::Playing)
    {
        set_fen(starting_fen);
    }

    ChessBoard(std::string fen) : game_state_(GameState::Playing)
    {
        set_fen(fen);
    }

    std::string to_string() const;
    bool act(Move move, bool update = true);
    void check_en_passant(Square from, Square to);
    void check_promotion(char promotion, Square from);
    bool has_mating_material() const;
    bool is_player_in_check(Player player) const;
    // generate pseudolegal moves
    Bitboard generate_moves(Square square) const;
    // remove moves from generateMoves that would leave the king in check
    Bitboard generate_legal_moves(Square square) const;
    std::vector<uint64_t> get_position_info() const;
    void update_game_state();
    void update_draw_condition(Square from, Square to);
    void castling(Square from, Square to);
    void set_fen(std::string fen);
    uint64_t generate_hash() const;
    inline Bitboard our_pieces(Player player) const
    {
        return player == Player::White ? white_pieces_ : black_pieces_;
    }
    inline Bitboard our_pieces() const
    {
        return our_pieces(player_);
    }
    inline Bitboard their_pieces(Player player) const
    {
        return player == Player::White ? black_pieces_ : white_pieces_;
    }
    inline Bitboard their_pieces() const
    {
        return their_pieces(player_);
    }
    inline int get_repetition_count() const
    {
        int repetitions = 0;
        for (int i = position_history_.size() - 3; i >= 0; i -= 2) {
            if (position_history_[i] == position_history_.back()) {
                repetitions++;
            }
        }
        return repetitions;
    }

    GameState game_state_;
    Player player_;
    std::vector<uint64_t> position_history_;
    int fifty_move_rule_;
    int fullmove_number_;
    int castling_rights_;
    Bitboard en_passant_;

    Bitboard all_pieces_;

    Bitboard white_pieces_;
    Bitboard black_pieces_;

    Bitboard pawns_;
    Bitboard knights_;
    Bitboard bishops_;
    Bitboard rooks_;
    Bitboard queens_;
    Bitboard kings_;
};
