#include "chessboard.h"
#include "engine.h"

char promotion_pieces[4] = {'q', 'r', 'b', 'n'};

std::vector<Move> moves;
ChessBoard board;

void init_engine() {
    init_keys();
    init_sliding_moves();
    reset_engine();
}

void reset_engine() {
    board = ChessBoard();
    moves.clear();
}

bool act(std::string move_string) {
    Move move = Move(move_string);
    if (is_legal_move(move)) {
        board.act(move);
        moves.push_back(move);
        return true;
    } else {
        return false;
    }
}

std::vector<Move> get_legal_moves() {
    std::vector<Move> legal_moves;

    for (auto from : board.our_pieces()) {
        Bitboard moves = board.generate_legal_moves(from);

        for (auto to : moves) {
            if (board.pawns_.get(from) && ((to.rank_ == 7) || (to.rank_ == 0))) {
                for (int i = 0; i < 4; i++) {
                    legal_moves.push_back(Move(from, to, promotion_pieces[i]));
                }
            } else {
                legal_moves.push_back(Move(from, to));
            }
        }
    }

    return legal_moves;
}

bool is_legal_move(Move move) {
    Square from = move.from_;
    Square to = move.to_;

    // promotion from non-pawn piece is illegal
    if (!board.pawns_.get(from) && move.promotion_ != '\0') {
        return false;
    }

    // non-promotion move to promotion square is illegal:q
    if (board.pawns_.get(from) && (to.rank_ == 7 || to.rank_ == 0) && move.promotion_ == '\0') {
        return false;
    }

    return board.generate_legal_moves(from).get(to) && board.our_pieces().get(from);
}