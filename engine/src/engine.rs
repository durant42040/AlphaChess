use std::fmt;

use crate::bitboard::Bitboard;
use crate::chessboard::ChessBoard;
use crate::constants::{
    BLACK_KINGSIDE_SQUARES, BLACK_PAWN_CAPTURES, BLACK_QUEENSIDE_ATTACKED, BLACK_QUEENSIDE_SQUARES,
    WHITE_KINGSIDE_SQUARES, WHITE_PAWN_CAPTURES, WHITE_QUEENSIDE_ATTACKED, WHITE_QUEENSIDE_SQUARES,
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
                let mut is_kingside_attacked = false;
                for idx in Bitboard::from(WHITE_KINGSIDE_SQUARES).iter() {
                    if self.is_under_attack(Square::from(idx)) {
                        is_kingside_attacked = true;
                        break;
                    }
                }
                let can_kingside = !is_kingside_attacked
                    && !pieces
                        .all_pieces
                        .intersects(Bitboard::from(WHITE_KINGSIDE_SQUARES) & !pieces.kings)
                    && (self.board.get_castling_rights() & 1) != 0;

                let mut is_queenside_attacked = false;
                for idx in Bitboard::from(WHITE_QUEENSIDE_ATTACKED).iter() {
                    if self.is_under_attack(Square::from(idx)) {
                        is_queenside_attacked = true;
                        break;
                    }
                }
                let can_queenside = !is_queenside_attacked
                    && !pieces
                        .all_pieces
                        .intersects(Bitboard::from(WHITE_QUEENSIDE_SQUARES) & !pieces.kings)
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
                let mut is_kingside_attacked = false;
                for idx in Bitboard::from(BLACK_KINGSIDE_SQUARES).iter() {
                    if self.is_under_attack(Square::from(idx)) {
                        is_kingside_attacked = true;
                        break;
                    }
                }
                let can_kingside = !is_kingside_attacked
                    && !pieces
                        .all_pieces
                        .intersects(Bitboard::from(BLACK_KINGSIDE_SQUARES) & !pieces.kings)
                    && (self.board.get_castling_rights() & 4) != 0;

                let mut is_queenside_attacked = false;
                for idx in Bitboard::from(BLACK_QUEENSIDE_ATTACKED).iter() {
                    if self.is_under_attack(Square::from(idx)) {
                        is_queenside_attacked = true;
                        break;
                    }
                }
                let can_queenside = !is_queenside_attacked
                    && !pieces
                        .all_pieces
                        .intersects(Bitboard::from(BLACK_QUEENSIDE_SQUARES) & !pieces.kings)
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
            self.board.switch_player();
            if self.is_check() {
                legal_moves.clear(to);
            }
            self.board.switch_player();
            self.board.undo(r#move);
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

    /// Checks if the given square is under attack by the opponent.
    pub fn is_under_attack(&self, square: Square) -> bool {
        let pieces = self.get_pieces();
        let their_pieces = self.board.get_their_pieces();

        let their_king = pieces.kings & their_pieces;
        if self
            .move_generator
            .generate_king_moves(square)
            .intersects(their_king)
        {
            return true;
        }

        let their_rooks = pieces.rooks & their_pieces;
        if self
            .move_generator
            .generate_rook_moves(square, pieces.all_pieces)
            .intersects(their_rooks)
        {
            return true;
        }

        let their_bishops = pieces.bishops & their_pieces;
        if self
            .move_generator
            .generate_bishop_moves(square, pieces.all_pieces)
            .intersects(their_bishops)
        {
            return true;
        }

        let their_queen = pieces.queens & their_pieces;
        if self
            .move_generator
            .generate_queen_moves(square, pieces.all_pieces)
            .intersects(their_queen)
        {
            return true;
        }

        let their_knights = pieces.knights & their_pieces;
        if self
            .move_generator
            .generate_knight_moves(square)
            .intersects(their_knights)
        {
            return true;
        }

        let their_pawns = pieces.pawns & their_pieces;
        if self.board.get_player() == Player::White {
            if Bitboard::from(WHITE_PAWN_CAPTURES[square]).intersects(their_pawns) {
                return true;
            }
        } else if Bitboard::from(BLACK_PAWN_CAPTURES[square]).intersects(their_pawns) {
            return true;
        }

        false
    }

    pub fn is_check(&self) -> bool {
        let our_king = self.board.get_our_pieces() & self.get_pieces().kings;
        self.is_under_attack(Square::from(our_king.get_lsb()))
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
        // insufficient material, 50-move rule, three-fold repetition
        if self.board.is_draw() {
            self.game_state = GameState::Draw;
            return;
        }

        // if there are legal moves, continue playing
        for from in self.board.get_our_pieces().iter() {
            if !self.generate_legal_moves(Square::from(from)).empty() {
                return;
            }
        }

        // if there are no legal moves, check for checkmate or stalemate
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

        for r#move in moves {
            self.board.act(r#move);
            self.update_game_state();
            nodes += self.perft(depth - 1);
            self.board.undo(r#move);
            self.game_state = GameState::Playing;
        }

        nodes
    }
}
