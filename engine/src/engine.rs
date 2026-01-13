use std::fmt;

use crate::bitboard::Bitboard;
use crate::chessboard::ChessBoard;
use crate::constants::{
    BLACK_KINGSIDE_SQUARES, BLACK_QUEENSIDE_ATTACKED, BLACK_QUEENSIDE_SQUARES,
    WHITE_KINGSIDE_SQUARES, WHITE_QUEENSIDE_ATTACKED, WHITE_QUEENSIDE_SQUARES,
};
use crate::game::{GameState, Player};
use crate::r#move::Move;
use crate::move_generator::MoveGenerator;
use crate::pieces::{Piece, Pieces};
use crate::square::Square;

pub struct Engine {
    board: ChessBoard,
    move_generator: MoveGenerator,
    game_state: GameState,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: ChessBoard::new(),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
        }
    }

    pub fn load_from_fen(&mut self, fen: String) {
        self.board = ChessBoard::load_from_fen(fen);
        self.game_state = GameState::Playing;
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
        self.game_state = GameState::Playing;
    }

    pub fn act(&mut self, move_string: String) -> bool {
        let r#move = move_string.parse::<Move>().unwrap();
        if !self.is_legal_move(r#move) {
            return false;
        }
        self.board.act(r#move);
        self.update_game_state();
        true
    }

    fn get_pieces(&self) -> Pieces {
        self.board.get_pieces()
    }

    fn generate_moves(&self, from: Square) -> Bitboard {
        let pieces = self.get_pieces();
        let our_pieces = if pieces.white_pieces.get_square(from) {
            pieces.white_pieces
        } else {
            pieces.black_pieces
        };

        let mut moves = Bitboard::default();

        if pieces.pawns.get_square(from) {
            if pieces.white_pieces.get_square(from) {
                moves = self.move_generator.generate_white_pawn_moves(
                    from,
                    pieces.all_pieces,
                    pieces.black_pieces | pieces.en_passant,
                );
            } else {
                moves = self.move_generator.generate_black_pawn_moves(
                    from,
                    pieces.all_pieces,
                    pieces.white_pieces | pieces.en_passant,
                );
            }
        } else if pieces.knights.get_square(from) {
            moves = self.move_generator.generate_knight_moves(from);
        } else if pieces.bishops.get_square(from) {
            moves = self
                .move_generator
                .generate_bishop_moves(from, pieces.all_pieces);
        } else if pieces.rooks.get_square(from) {
            moves = self
                .move_generator
                .generate_rook_moves(from, pieces.all_pieces);
        } else if pieces.queens.get_square(from) {
            moves = self
                .move_generator
                .generate_queen_moves(from, pieces.all_pieces);
        } else if pieces.kings.get_square(from) {
            moves = self.move_generator.generate_king_moves(from);
        }

        moves &= !our_pieces;

        moves
    }

    fn generate_castling_moves(&self, from: Square) -> Bitboard {
        let pieces = self.get_pieces();
        let mut castling_moves = Bitboard::default();

        match from.square {
            4 => {
                // White king's castle
                let mut black_attacks = Bitboard::default();
                for idx in pieces.black_pieces.iter() {
                    let square = Square::from(idx);
                    black_attacks |= self.generate_moves(square);
                    if pieces.pawns.get(idx) {
                        if square.rank > 0 && square.file > 0 {
                            let target = Square::new(square.rank - 1, square.file - 1);
                            black_attacks.set_square(target);
                        }
                        if square.rank > 0 && square.file < 7 {
                            let target = Square::new(square.rank - 1, square.file + 1);
                            black_attacks.set_square(target);
                        }
                    }
                }
                let can_kingside = (black_attacks & Bitboard::from(WHITE_KINGSIDE_SQUARES)).empty()
                    && (pieces.all_pieces & Bitboard::from(WHITE_KINGSIDE_SQUARES) & !pieces.kings)
                        .empty()
                    && (self.board.get_castling_rights() & 1) != 0;
                let can_queenside = (black_attacks & Bitboard::from(WHITE_QUEENSIDE_ATTACKED))
                    .empty()
                    && (pieces.all_pieces
                        & Bitboard::from(WHITE_QUEENSIDE_SQUARES)
                        & !pieces.kings)
                        .empty()
                    && (self.board.get_castling_rights() & 2) != 0;
                if can_kingside {
                    castling_moves.set(6);
                }
                if can_queenside {
                    castling_moves.set(2);
                }
            }
            60 => {
                // Black king's castle
                let mut white_attacks = Bitboard::default();
                for idx in pieces.white_pieces.iter() {
                    let square = Square::from(idx);
                    white_attacks |= self.generate_moves(square);
                    if pieces.pawns.get(idx) {
                        if square.rank < 7 && square.file > 0 {
                            let target = Square::new(square.rank + 1, square.file - 1);
                            white_attacks.set_square(target);
                        }
                        if square.rank < 7 && square.file < 7 {
                            let target = Square::new(square.rank + 1, square.file + 1);
                            white_attacks.set_square(target);
                        }
                    }
                }
                let can_kingside = (white_attacks & Bitboard::from(BLACK_KINGSIDE_SQUARES)).empty()
                    && (pieces.all_pieces & Bitboard::from(BLACK_KINGSIDE_SQUARES) & !pieces.kings)
                        .empty()
                    && (self.board.get_castling_rights() & 4) != 0;
                let can_queenside = (white_attacks & Bitboard::from(BLACK_QUEENSIDE_ATTACKED))
                    .empty()
                    && (pieces.all_pieces
                        & Bitboard::from(BLACK_QUEENSIDE_SQUARES)
                        & !pieces.kings)
                        .empty()
                    && (self.board.get_castling_rights() & 8) != 0;
                if can_kingside {
                    castling_moves.set(62);
                }
                if can_queenside {
                    castling_moves.set(58);
                }
            }
            _ => {}
        }

        castling_moves
    }

    pub fn generate_legal_moves(&mut self, from: Square) -> Bitboard {
        let mut legal_moves = self.generate_moves(from);

        if self.get_pieces().kings.get_square(from) && (from == 4 || from == 60) {
            legal_moves |= self.generate_castling_moves(from);
        }

        // remove moves that would put our king in check
        // e.g. pins, illegal king moves
        for to in legal_moves.iter() {
            let r#move = Move::new(from, to.into(), None);
            self.board.act(r#move);
            if self.is_player_in_check(self.board.get_player().switch()) {
                legal_moves.clear(to);
            }
            self.board.undo(r#move.clone());
        }

        legal_moves
    }

    pub fn generate_all_legal_moves(&mut self) -> Vec<Move> {
        let mut all_legal_moves = Vec::new();
        for from in self.board.get_our_pieces().iter() {
            let moves = self.generate_legal_moves(Square::from(from));
            for to in moves.iter() {
                let from = Square::from(from);
                let to = Square::from(to);
                if self.get_pieces().pawns.get_square(from) && (to.rank == 7 || to.rank == 0) {
                    all_legal_moves.push(Move::new(from, to, Some(Piece::Queen)));
                    all_legal_moves.push(Move::new(from, to, Some(Piece::Rook)));
                    all_legal_moves.push(Move::new(from, to, Some(Piece::Bishop)));
                    all_legal_moves.push(Move::new(from, to, Some(Piece::Knight)));
                } else {
                    all_legal_moves.push(Move::new(from, to, None));
                }
            }
        }

        all_legal_moves
    }

    pub fn get_game_state(&self) -> String {
        self.game_state.to_string()
    }

    pub fn is_check(&self) -> bool {
        self.is_player_in_check(self.board.get_player())
    }

    fn is_player_in_check(&self, player: Player) -> bool {
        let pieces = self.get_pieces();
        let their_pieces = if player == Player::White {
            pieces.black_pieces
        } else {
            pieces.white_pieces
        };
        let our_king = if player == Player::White {
            pieces.white_pieces & pieces.kings
        } else {
            pieces.black_pieces & pieces.kings
        };
        let our_king_position = our_king.get_lsb();

        for from in their_pieces.iter() {
            let moves = self.generate_moves(Square::from(from));
            if moves.get(our_king_position) {
                return true;
            }
        }
        false
    }

    pub fn is_legal_move(&mut self, r#move: Move) -> bool {
        let from = r#move.from;
        let to = r#move.to;

        // move from our pieces
        if !self.board.get_our_pieces().get_square(from) {
            return false;
        }

        let is_pawn = self.get_pieces().pawns.get_square(from);

        // promotion from non-pawn piece is illegal
        if !is_pawn && r#move.promotion.is_some() {
            return false;
        }

        // non-promotion move to promotion square is illegal
        if is_pawn && (to.rank == 7 || to.rank == 0) && r#move.promotion.is_none() {
            return false;
        }

        self.generate_legal_moves(from).get_square(to)
    }

    fn update_game_state(&mut self) {
        let mut moves = Bitboard::default();
        for from in self.board.get_our_pieces().iter() {
            moves |= self.generate_legal_moves(Square::from(from));
        }

        if moves.empty() {
            if self.is_check() {
                // checkmate
                if self.board.get_player() == Player::White {
                    self.game_state = GameState::BlackWin;
                } else {
                    self.game_state = GameState::WhiteWin;
                }
            } else {
                // stalemate
                self.game_state = GameState::Draw;
            }
        }
        // insufficient material, 50-move rule, three-fold repetition
        if self.board.is_draw() {
            self.game_state = GameState::Draw;
        }
    }

    pub fn to_board_string(&self) -> String {
        let mut board_str = String::with_capacity(64);
        let pieces = self.get_pieces();

        for i in 0..64 {
            board_str.push(pieces.get_char(i));
        }

        board_str
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}

pub trait Perft {
    fn perft(&mut self, depth: u8) -> u64;
}

impl Perft for Engine {
    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }
        if self.get_game_state() != "playing" {
            return 0;
        }

        let moves = self.generate_all_legal_moves();
        let mut nodes = 0u64;

        let temp_board = self.board.clone();
        for r#move in moves {
            self.board.act(r#move);
            self.update_game_state();
            let ans = self.perft(depth - 1);
            nodes += ans;
            self.board = temp_board.clone();
            self.game_state = GameState::Playing;
        }

        nodes
    }
}
