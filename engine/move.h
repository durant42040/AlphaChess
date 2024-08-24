#pragma once

#include "square.h"

const std::string chess_positions[] = {
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"};


class Move {
public:
    Move(Square from, Square to, char promotion = '\0') : from_(from), to_(to), promotion_(promotion) {}
    Move(std::string move) : from_(move.substr(0, 2)), to_(move.substr(2, 2)), promotion_('\0') {
        if (move.size() == 5) {
            promotion_ = move[4];
        }
    }

    std::string to_string() const {
        std::string move = chess_positions[from_.square_] + chess_positions[to_.square_];
        if (promotion_ != '\0') {
            move += promotion_;
        }
        return move;
    }

    Square from_;
    Square to_;
    char promotion_;
        
};