use std::fmt;

use crate::chess::AttackState;
use crate::chess::chessboard::{Castling, ChessBoard};
use crate::chess::constants::*;
use crate::chess::r#move::MoveList;
use crate::chess::pieces::{Color, Piece, Pieces};
use crate::chess::{Bitboard, GameState, Move, MoveGenerator, Player, Square};

pub struct Engine {
    board: ChessBoard,
    move_generator: MoveGenerator,
    game_state: GameState,
    attack_state: AttackState,
}

impl Engine {
    pub fn new() -> Self {
        Bitboard::init();

        Self {
            board: ChessBoard::new(),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
            attack_state: AttackState::default(),
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        Bitboard::init();

        let mut engine = Self {
            board: ChessBoard::load_from_fen(fen),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
            attack_state: AttackState::default(),
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
                if let Some((piece, color)) = pieces.get_piece(square) {
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
    }

    pub fn make_move(&mut self, move_string: &str) -> bool {
        let r#move = move_string.parse::<Move>().unwrap();
        if !self.is_legal_move(r#move) {
            return false;
        }
        self.board.act(r#move);
        self.update_attack_state();
        self.update_game_state();
        true
    }

    pub fn act(&mut self, r#move: Move) {
        self.board.act(r#move);
        self.update_attack_state();
        self.update_game_state();
    }

    pub fn undo(&mut self) -> bool {
        if self.board.move_history().is_empty() {
            return false;
        }
        self.board.undo();
        self.update_attack_state();
        self.game_state = GameState::Playing;
        true
    }

    pub fn pieces(&self) -> Pieces {
        self.board.pieces()
    }

    fn generate_moves(&self, from: Square) -> Bitboard {
        let pieces = self.pieces();
        let our_pieces = if pieces.white_pieces().get_square(from) {
            pieces.white_pieces()
        } else {
            pieces.black_pieces()
        };

        let mut moves = Bitboard::default();

        if pieces.pawns().get_square(from) {
            if pieces.white_pieces().get_square(from) {
                moves = self.move_generator.generate_white_pawn_moves(
                    from,
                    pieces.all_pieces(),
                    pieces.black_pieces() | pieces.en_passant(),
                );
            } else {
                moves = self.move_generator.generate_black_pawn_moves(
                    from,
                    pieces.all_pieces(),
                    pieces.white_pieces() | pieces.en_passant(),
                );
            }
        } else if pieces.knights().get_square(from) {
            moves = self.move_generator.generate_knight_moves(from);
        } else if pieces.bishops().get_square(from) {
            moves = self
                .move_generator
                .generate_bishop_moves(from, pieces.all_pieces());
        } else if pieces.rooks().get_square(from) {
            moves = self
                .move_generator
                .generate_rook_moves(from, pieces.all_pieces());
        } else if pieces.queens().get_square(from) {
            moves = self
                .move_generator
                .generate_queen_moves(from, pieces.all_pieces());
        } else if pieces.kings().get_square(from) {
            moves = self.move_generator.generate_king_moves(from);
        }

        moves &= !our_pieces;

        moves
    }

    fn generate_castling_moves(&self, from: Square) -> Bitboard {
        let pieces = self.pieces();
        let mut castling_moves = Bitboard::default();

        match from.square {
            WHITE_KING_START => {
                // White king's castle
                let mut is_kingside_attacked = false;
                for idx in Bitboard::from(WHITE_KINGSIDE_SQUARES).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_kingside_attacked = true;
                        break;
                    }
                }
                let can_kingside = !is_kingside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(WHITE_KINGSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(WHITE_CASTLE_KINGSIDE);

                let mut is_queenside_attacked = false;
                for idx in Bitboard::from(WHITE_QUEENSIDE_ATTACKED).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_queenside_attacked = true;
                        break;
                    }
                }
                let can_queenside = !is_queenside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(WHITE_QUEENSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(WHITE_CASTLE_QUEENSIDE);

                if can_kingside {
                    castling_moves.set(WHITE_KINGSIDE_CASTLE_TO);
                }
                if can_queenside {
                    castling_moves.set(WHITE_QUEENSIDE_CASTLE_TO);
                }
            }
            BLACK_KING_START => {
                // Black king's castle
                let mut is_kingside_attacked = false;
                for idx in Bitboard::from(BLACK_KINGSIDE_SQUARES).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_kingside_attacked = true;
                        break;
                    }
                }
                let can_kingside = !is_kingside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(BLACK_KINGSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(BLACK_CASTLE_KINGSIDE);

                let mut is_queenside_attacked = false;
                for idx in Bitboard::from(BLACK_QUEENSIDE_ATTACKED).iter() {
                    if self.is_square_under_attack(Square::from(idx)) {
                        is_queenside_attacked = true;
                        break;
                    }
                }
                let can_queenside = !is_queenside_attacked
                    && !pieces
                        .all_pieces()
                        .intersects(Bitboard::from(BLACK_QUEENSIDE_SQUARES) & !pieces.kings())
                    && self.board.castling_rights().can(BLACK_CASTLE_QUEENSIDE);

                if can_kingside {
                    castling_moves.set(BLACK_KINGSIDE_CASTLE_TO);
                }
                if can_queenside {
                    castling_moves.set(BLACK_QUEENSIDE_CASTLE_TO);
                }
            }
            _ => {}
        }

        castling_moves
    }

    pub fn generate_legal_moves(&mut self, from: Square) -> Bitboard {
        let mut legal_moves = self.generate_moves(from);
        let pieces = self.pieces();

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
        if pieces.pawns().get_square(from)
            && !pieces.en_passant().empty()
            && legal_moves.intersects(pieces.en_passant())
        {
            let to = Square::from(pieces.en_passant());
            let r#move = Move::new(from, to, None);
            self.board.act(r#move);
            let our_king = self.board.their_pieces() & self.pieces().kings();

            if self.is_under_attack(
                Square::from(our_king),
                self.board.pieces().all_pieces(),
                self.board.player().switch().into(),
            ) {
                legal_moves.clear_square(to);
            } else {
                en_passant_legal = true;
            }
            self.board.undo();
        }

        if self.is_check() {
            // if in double check, no legal non-king moves
            if self.attack_state.num_checks == 2 {
                return Bitboard::default();
            }

            // pinned pieces cannot resolve check
            if self.attack_state.pinned_pieces.get_square(from) {
                return Bitboard::default();
            }

            // if not double check, non-king move must block the check or capture
            legal_moves &= self.attack_state.attack_lines | self.attack_state.attackers;

            if en_passant_legal {
                legal_moves |= pieces.en_passant();
            }

            return legal_moves;
        }

        if self.attack_state.pinned_pieces.get_square(from) {
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

    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    /// Checks if the given square is under attack by the opponent.
    fn is_under_attack(&self, square: Square, all_pieces: Bitboard, color: Color) -> bool {
        self.generate_attacks(square, all_pieces, color).count() > 0
    }

    /// Checks if the given square is under attack by the opponent in the current position.
    fn is_square_under_attack(&self, square: Square) -> bool {
        let pieces = self.pieces();
        self.is_under_attack(square, pieces.all_pieces(), self.board.player().into())
    }

    /// returns all attackers to the given square
    fn generate_attacks(&self, square: Square, all_pieces: Bitboard, color: Color) -> Bitboard {
        let pieces = self.pieces();
        let their_pieces = if color == Color::White {
            pieces.black_pieces()
        } else {
            pieces.white_pieces()
        };

        (self.move_generator.generate_king_moves(square) & (pieces.kings() & their_pieces))
            | (self.move_generator.generate_rook_moves(square, all_pieces)
                & (pieces.rooks() & their_pieces))
            | (self
                .move_generator
                .generate_bishop_moves(square, all_pieces)
                & (pieces.bishops() & their_pieces))
            | (self.move_generator.generate_queen_moves(square, all_pieces)
                & (pieces.queens() & their_pieces))
            | (self.move_generator.generate_knight_moves(square)
                & (pieces.knights() & their_pieces))
            | (if color == Color::White {
                Bitboard::from(WHITE_PAWN_CAPTURES[square]) & (pieces.pawns() & their_pieces)
            } else {
                Bitboard::from(BLACK_PAWN_CAPTURES[square]) & (pieces.pawns() & their_pieces)
            })
    }

    pub fn is_check(&self) -> bool {
        self.attack_state.num_checks > 0
    }

    fn is_legal_move(&mut self, r#move: Move) -> bool {
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

    pub fn to_board_string(&self) -> String {
        let mut board_str = String::with_capacity(64);
        let pieces = self.pieces();

        for i in 0..64 {
            board_str.push(pieces.get_char(i));
        }

        board_str
    }

    pub fn update_attack_state(&mut self) {
        let mut pinned_pieces = Bitboard::default();
        let pieces = self.pieces();
        let our_pieces = self.board.our_pieces();
        let their_pieces = self.board.their_pieces();
        let our_king = Square::from(our_pieces & pieces.kings());

        let rook_rays = self
            .move_generator
            .generate_rook_moves(our_king, Bitboard::default());
        let bishop_rays = self
            .move_generator
            .generate_bishop_moves(our_king, Bitboard::default());

        let mut snipers = their_pieces
            & ((pieces.queens() | pieces.rooks()) & rook_rays
                | (pieces.queens() | pieces.bishops()) & bishop_rays);

        let mut attack_lines = Bitboard::default();
        while !snipers.empty() {
            let sniper = snipers.pop_lsb();
            let blockers = pieces.all_pieces() & Bitboard::between(Square::from(sniper), our_king);
            if blockers.count() == 1 && our_pieces.intersects(blockers) {
                pinned_pieces |= blockers;
            } else if blockers.count() == 0 {
                // If no blockers, this is a check from a sliding piece
                attack_lines |= Bitboard::between(Square::from(sniper), our_king);
            }
        }

        let attackers =
            self.generate_attacks(our_king, pieces.all_pieces(), self.board.player().into());
        let num_checks = attackers.count();
        debug_assert!(num_checks <= 2);

        self.attack_state = AttackState {
            attackers,
            num_checks,
            pinned_pieces,
            attack_lines,
        };
    }
}

pub trait Evaluation {
    fn eval(&self) -> i32;
}

impl Evaluation for Engine {
    /// Evaluate the position as white
    fn eval(&self) -> i32 {
        if self.game_state == GameState::Draw {
            return 0;
        }
        if self.game_state == GameState::WhiteWin {
            return i32::MAX;
        }
        if self.game_state == GameState::BlackWin {
            return i32::MIN;
        }
        self.board.material_score()
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
