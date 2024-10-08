#pragma once

#include "chessboard.h"
#include "move.h"
#include "move_generator.h"

extern std::vector<Move> moves;
extern ChessBoard board;

void init_engine();
void reset_engine();
std::string get_game_state();
bool is_check();
bool is_legal_move(Move move);
std::vector<Move> get_legal_moves();
bool act(std::string move);
std::string get_board();
