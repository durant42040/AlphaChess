pub mod chess;
pub mod constants;
pub mod play;
pub mod search;

use std::{fmt, time::Duration};

use crate::chess::AttackState;
use crate::chess::chessboard::ChessBoard;
use crate::chess::r#move::MoveList;
use crate::chess::pieces::{Piece, Pieces};
use crate::chess::{Bitboard, GameState, Move, MoveGenerator, Player, Square};
use crate::constants::{
    ALL_CASTLING_RIGHTS, BLACK_CASTLE_KINGSIDE, BLACK_CASTLE_QUEENSIDE, BLACK_KING_START,
    WHITE_CASTLE_KINGSIDE, WHITE_CASTLE_QUEENSIDE, WHITE_KING_START,
};
use crate::search::Search;

pub struct Engine {
    board: ChessBoard,
    move_generator: MoveGenerator,
    game_state: GameState,
    attack_states: Vec<AttackState>,
    search: Search,
}

impl Engine {
    pub fn new() -> Self {
        Bitboard::init();

        let mut engine = Self {
            board: ChessBoard::new(),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
            attack_states: Vec::with_capacity(8192),
            search: Search::new(),
        };

        engine.attack_states.push(AttackState::default());
        engine
    }

    pub fn from_fen(fen: &str) -> Self {
        Bitboard::init();

        let mut engine = Self {
            board: ChessBoard::load_from_fen(fen),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
            attack_states: Vec::with_capacity(8192),
            search: Search::new(),
        };

        engine.update_attack_state();
        engine.update_game_state();
        engine
    }

    pub fn get_fen(&self) -> String {
        let pieces = self.pieces();
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let square = rank * 8 + file;
                if let Some((piece, color)) = pieces.piece(Square::from(square)) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(piece.to_char(color));
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(if self.board.player() == Player::White {
            'w'
        } else {
            'b'
        });

        fen.push(' ');
        if !self.board.castling_rights().can(ALL_CASTLING_RIGHTS) {
            fen.push('-');
        } else {
            if self.board.castling_rights().can(WHITE_CASTLE_KINGSIDE) {
                fen.push('K');
            }
            if self.board.castling_rights().can(WHITE_CASTLE_QUEENSIDE) {
                fen.push('Q');
            }
            if self.board.castling_rights().can(BLACK_CASTLE_KINGSIDE) {
                fen.push('k');
            }
            if self.board.castling_rights().can(BLACK_CASTLE_QUEENSIDE) {
                fen.push('q');
            }
        }

        fen.push(' ');
        if pieces.en_passant().empty() {
            fen.push('-');
        } else {
            let ep_square = Square::from(pieces.en_passant());
            fen.push_str(&ep_square.to_string());
        }

        fen.push(' ');
        fen.push_str(&self.board.fifty_move_rule().to_string());

        fen.push(' ');
        fen.push('1');

        fen
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
        self.game_state = GameState::Playing;
        self.attack_states.clear();
        self.update_attack_state();
        self.search = Search::new();
    }

    pub fn set_ponder_time(&mut self, time: u64) {
        self.search.ponder_time = Duration::from_millis(time);
    }

    pub fn make_move(&mut self, r#move: Move) -> bool {
        if !self.is_legal_move(r#move) {
            return false;
        }
        self.board.act(r#move);
        self.update_attack_state();
        self.update_game_state();
        true
    }

    pub fn act(&mut self, r#move: Move) {
        assert!(!r#move.is_none());
        self.board.act(r#move);
        self.update_attack_state();
    }

    pub fn undo(&mut self) -> bool {
        if self.board.move_history().is_empty() {
            return false;
        }
        self.board.undo();
        assert!(!self.attack_states.is_empty());
        self.attack_states.pop();
        true
    }

    pub fn make_null_move(&mut self) -> Bitboard {
        let prev_en_passant = self.board.make_null_move();
        self.update_attack_state();
        prev_en_passant
    }

    pub fn undo_null_move(&mut self, en_passant: Bitboard) {
        self.board.undo_null_move(en_passant);
        assert!(!self.attack_states.is_empty());
        self.attack_states.pop();
    }

    #[inline(always)]
    pub fn pieces(&self) -> Pieces {
        self.board.pieces()
    }

    #[inline(always)]
    pub fn player(&self) -> Player {
        self.board.player()
    }

    pub fn generate_legal_moves(&mut self, from: Square) -> Bitboard {
        let mut legal_moves = self.generate_moves(from);

        let pieces = self.pieces();
        let our_pieces = if pieces.white_pieces().get_square(from) {
            pieces.white_pieces()
        } else {
            pieces.black_pieces()
        };

        legal_moves &= !our_pieces;

        // King moves
        if pieces.kings().get_square(from) {
            for to in legal_moves.iter() {
                if self.is_under_attack(
                    to.into(),
                    pieces.all_pieces() & !Bitboard::from(from),
                    self.board.player().into(),
                ) {
                    legal_moves.clear(to);
                }
            }
            if from == WHITE_KING_START || from == BLACK_KING_START {
                legal_moves |= self.generate_castling_moves(from);
            }
            return legal_moves;
        }

        // is en passant legal?
        let mut en_passant_legal = false;
        if pieces.pawns().get_square(from) && legal_moves.intersects(pieces.en_passant()) {
            let to = Square::from(pieces.en_passant());
            let r#move = Move::new(from, to, None);
            self.board.act(r#move);
            let our_king = self.board.their_pieces() & self.pieces().kings();

            if self.is_under_attack(
                Square::from(our_king),
                self.board.pieces().all_pieces(),
                (!self.board.player()).into(),
            ) {
                legal_moves.clear_square(to);
            } else {
                en_passant_legal = true;
            }
            self.board.undo();
        }

        let attack_state = self.attack_state();
        if self.is_check() {
            // if in double check, no legal non-king moves
            if attack_state.num_checks == 2 {
                return Bitboard::zero();
            }

            // pinned pieces cannot resolve check
            if attack_state.pinned_pieces.get_square(from) {
                return Bitboard::zero();
            }

            // if not double check, non-king move must block the check or capture
            legal_moves &= attack_state.attack_lines | attack_state.attackers;

            if en_passant_legal {
                legal_moves |= pieces.en_passant();
            }

            return legal_moves;
        }

        if attack_state.pinned_pieces.get_square(from) {
            // if a piece is pinned, only the moves that are along the pin ray are legal
            let our_king = self.board.our_pieces() & self.pieces().kings();
            legal_moves &= Bitboard::ray(from, Square::from(our_king));
        }

        legal_moves
    }

    pub fn generate_all_legal_moves(&mut self) -> MoveList {
        let mut all_legal_moves = MoveList::new();

        for from in self.board.our_pieces().iter() {
            let moves = self.generate_legal_moves(Square::from(from));
            for to in moves.iter() {
                let from = Square::from(from);
                let to = Square::from(to);
                if (to.rank == 7 || to.rank == 0) && self.pieces().pawns().get_square(from) {
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

    pub fn generate_all_capture_moves(&mut self) -> MoveList {
        let mut all_capture_moves = MoveList::new();

        for from in self.board.our_pieces().iter() {
            let capture_moves = self.generate_legal_moves(Square::from(from))
                & (self.board.pieces().all_pieces() | self.board.pieces().en_passant());
            for to in capture_moves.iter() {
                let from = Square::from(from);
                let to = Square::from(to);
                if (to.rank == 7 || to.rank == 0) && self.pieces().pawns().get_square(from) {
                    all_capture_moves.push(Move::new(from, to, Some(Piece::Queen)));
                    all_capture_moves.push(Move::new(from, to, Some(Piece::Rook)));
                    all_capture_moves.push(Move::new(from, to, Some(Piece::Bishop)));
                    all_capture_moves.push(Move::new(from, to, Some(Piece::Knight)));
                } else {
                    all_capture_moves.push(Move::new(from, to, None));
                }
            }
        }

        all_capture_moves
    }

    pub fn is_legal_move(&mut self, r#move: Move) -> bool {
        let from = r#move.from;
        let to = r#move.to;

        // move from our pieces
        if !self.board.our_pieces().get_square(from) {
            return false;
        }

        let is_pawn = self.pieces().pawns().get_square(from);

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

    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    pub fn update_game_state(&mut self) {
        // insufficient material, 50-move rule, three-fold repetition
        if self.board.is_draw() {
            self.game_state = GameState::Draw;
            return;
        }

        // if there are legal moves, continue playing
        for from in self.board.our_pieces().iter() {
            if !self.generate_legal_moves(Square::from(from)).empty() {
                return;
            }
        }
        // if there are no legal moves, check for checkmate or stalemate
        if self.is_check() {
            // checkmate
            if self.board.player() == Player::White {
                self.game_state = GameState::BlackWin;
            } else {
                self.game_state = GameState::WhiteWin;
            }
        } else {
            // stalemate
            self.game_state = GameState::Draw;
        }
    }

    pub fn board(&self) -> &ChessBoard {
        &self.board
    }

    pub fn to_board_string(&self) -> String {
        let mut board_str = String::with_capacity(64);
        let pieces = self.pieces();

        for i in 0..64 {
            board_str.push(pieces.get_char(i));
        }

        board_str
    }

    pub fn attack_state(&self) -> &AttackState {
        self.attack_states.last().unwrap()
    }

    pub fn is_check(&self) -> bool {
        self.attack_state().num_checks > 0
    }

    pub fn is_endgame(&self) -> bool {
        self.board.non_pawn_material() <= 2000
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
