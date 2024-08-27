#include "engine.h"
#include "chessboard.h"

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
            if (board.pawns_.get(from) &&
                ((to.rank_ == 7) || (to.rank_ == 0))) {
                legal_moves.push_back(Move(from, to, 'q'));
                legal_moves.push_back(Move(from, to, 'r'));
                legal_moves.push_back(Move(from, to, 'b'));
                legal_moves.push_back(Move(from, to, 'n'));
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
    if (board.pawns_.get(from) && (to.rank_ == 7 || to.rank_ == 0) &&
        move.promotion_ == '\0') {
        return false;
    }

    return board.generate_legal_moves(from).get(to) &&
           board.our_pieces().get(from);
}

std::string get_game_state() {
    switch (board.game_state_) {
    case GameState::Playing:
        return "playing";
    case GameState::WhiteWin:
        return "checkmate";
    case GameState::BlackWin:
        return "checkmate";
    case GameState::Draw:
        return "draw";
    default:
        return "playing";
    }
}

bool is_check() { return board.is_player_in_check(board.player_); }

std::string get_board() {
    std::string board_str;
    for (int i = 0; i < 64; i++) {
        if (board.pawns_.get(i)) {
            board_str += board.white_pieces_.get(i) ? 'P' : 'p';
        } else if (board.knights_.get(i)) {
            board_str += board.white_pieces_.get(i) ? 'N' : 'n';
        } else if (board.bishops_.get(i)) {
            board_str += board.white_pieces_.get(i) ? 'B' : 'b';
        } else if (board.rooks_.get(i)) {
            board_str += board.white_pieces_.get(i) ? 'R' : 'r';
        } else if (board.queens_.get(i)) {
            board_str += board.white_pieces_.get(i) ? 'Q' : 'q';
        } else if (board.kings_.get(i)) {
            board_str += board.white_pieces_.get(i) ? 'K' : 'k';
        } else {
            board_str += '.';
        }
    }

    return board_str;
}