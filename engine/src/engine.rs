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
use crate::pieces::{Color, Piece, Pieces};
use crate::square::Square;

pub struct Engine {
    board: ChessBoard,
    move_generator: MoveGenerator,
    game_state: GameState,
    move_history: Vec<Move>,
}

impl Engine {
    pub fn new() -> Self {
        Bitboard::init();

        Self {
            board: ChessBoard::new(),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
            move_history: Vec::new(),
        }
    }

    pub fn from_fen(fen: String) -> Self {
        Bitboard::init();

        Self {
            board: ChessBoard::load_from_fen(fen),
            move_generator: MoveGenerator::new(),
            game_state: GameState::Playing,
            move_history: Vec::new(),
        }
    }

    pub fn get_fen(&self) -> String {
        let pieces = self.get_pieces();
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
        fen.push(if self.board.get_player() == Player::White {
            'w'
        } else {
            'b'
        });

        fen.push(' ');
        let castling_rights = self.board.get_castling_rights();
        if castling_rights == 0 {
            fen.push('-');
        } else {
            if (castling_rights & 1) != 0 {
                fen.push('K');
            }
            if (castling_rights & 2) != 0 {
                fen.push('Q');
            }
            if (castling_rights & 4) != 0 {
                fen.push('k');
            }
            if (castling_rights & 8) != 0 {
                fen.push('q');
            }
        }

        fen.push(' ');
        if pieces.en_passant.empty() {
            fen.push('-');
        } else {
            let ep_square = Square::from(pieces.en_passant);
            fen.push_str(&ep_square.to_string());
        }

        fen.push(' ');
        fen.push_str(&self.board.get_fifty_move_rule().to_string());

        fen.push(' ');
        fen.push('1');

        fen
    }

    pub fn reset(&mut self) {
        self.board = ChessBoard::new();
        self.game_state = GameState::Playing;
        self.move_history.clear();
    }

    pub fn act(&mut self, move_string: String) -> bool {
        let r#move = move_string.parse::<Move>().unwrap();
        if !self.is_legal_move(r#move) {
            return false;
        }
        self.board.act(r#move);
        self.move_history.push(r#move);
        self.update_game_state();
        true
    }

    pub fn undo(&mut self) -> bool {
        if self.move_history.is_empty() {
            return false;
        }

        let prev_move = self.move_history.pop().unwrap();
        self.board.undo(prev_move);
        self.game_state = GameState::Playing;
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
                    if self.is_square_under_attack(Square::from(idx)) {
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
                    if self.is_square_under_attack(Square::from(idx)) {
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
                    if self.is_square_under_attack(Square::from(idx)) {
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
                    if self.is_square_under_attack(Square::from(idx)) {
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
        let pieces = self.get_pieces();

        // King moves
        if pieces.kings.get_square(from) {
            for to in legal_moves.iter() {
                if self.is_under_attack(
                    to.into(),
                    pieces.all_pieces & !Bitboard::from(from),
                    self.board.get_player().into(),
                ) {
                    legal_moves.clear(to);
                }
            }
            if from == 4 || from == 60 {
                legal_moves |= self.generate_castling_moves(from);
            }
            return legal_moves;
        }

        // is en passant legal?
        let mut en_passant_legal = false;
        if pieces.pawns.get_square(from)
            && !pieces.en_passant.empty()
            && legal_moves.intersects(pieces.en_passant)
        {
            let to = Square::from(pieces.en_passant);
            let r#move = Move::new(from, to, None);
            self.board.act(r#move);
            let our_king = self.board.get_their_pieces() & self.get_pieces().kings;

            if self.is_under_attack(
                Square::from(our_king),
                self.board.get_pieces().all_pieces,
                self.board.get_player().switch().into(),
            ) {
                legal_moves.clear_square(to);
            } else {
                en_passant_legal = true;
            }

            self.board.undo(r#move);
        }

        let (pinned_pieces, attack_lines) = self.find_pinned_pieces();
        let our_king = self.board.get_our_pieces() & self.get_pieces().kings;
        let attacks = self.generate_attacks(
            Square::from(our_king),
            pieces.all_pieces,
            self.board.get_player().into(),
        );
        let num_checks = attacks.count();

        if num_checks > 0 {
            debug_assert!(num_checks <= 2);

            // if in double check, no legal non-king moves
            if num_checks == 2 {
                return Bitboard::default();
            }

            // pinned pieces cannot resolve check
            if pinned_pieces.get_square(from) {
                return Bitboard::default();
            }

            // if not double check, non-king move must block the check or capture
            legal_moves &= attack_lines | attacks;

            if en_passant_legal {
                legal_moves |= pieces.en_passant;
            }

            return legal_moves;
        }

        if pinned_pieces.get_square(from) {
            // if a piece is pinned, only the moves that are along the pin ray are legal
            let our_king = self.board.get_our_pieces() & self.get_pieces().kings;
            legal_moves &= Bitboard::ray(from, Square::from(our_king));
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
    fn is_under_attack(&self, square: Square, all_pieces: Bitboard, color: Color) -> bool {
        self.generate_attacks(square, all_pieces, color).count() > 0
    }

    /// Checks if the given square is under attack by the opponent in the current position.
    fn is_square_under_attack(&self, square: Square) -> bool {
        let pieces = self.get_pieces();
        self.is_under_attack(square, pieces.all_pieces, self.board.get_player().into())
    }

    /// returns all attackers to the given square
    fn generate_attacks(&self, square: Square, all_pieces: Bitboard, color: Color) -> Bitboard {
        let pieces = self.get_pieces();
        let their_pieces = if color == Color::White {
            pieces.black_pieces
        } else {
            pieces.white_pieces
        };

        (self.move_generator.generate_king_moves(square) & (pieces.kings & their_pieces))
            | (self.move_generator.generate_rook_moves(square, all_pieces)
                & (pieces.rooks & their_pieces))
            | (self
                .move_generator
                .generate_bishop_moves(square, all_pieces)
                & (pieces.bishops & their_pieces))
            | (self.move_generator.generate_queen_moves(square, all_pieces)
                & (pieces.queens & their_pieces))
            | (self.move_generator.generate_knight_moves(square) & (pieces.knights & their_pieces))
            | (if color == Color::White {
                Bitboard::from(WHITE_PAWN_CAPTURES[square]) & (pieces.pawns & their_pieces)
            } else {
                Bitboard::from(BLACK_PAWN_CAPTURES[square]) & (pieces.pawns & their_pieces)
            })
    }

    pub fn is_check(&self) -> bool {
        let our_king = self.board.get_our_pieces() & self.get_pieces().kings;
        self.is_square_under_attack(Square::from(our_king))
    }

    fn is_legal_move(&mut self, r#move: Move) -> bool {
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

    pub fn find_pinned_pieces(&self) -> (Bitboard, Bitboard) {
        let mut pinned_pieces = Bitboard::default();
        let pieces = self.get_pieces();
        let our_pieces = self.board.get_our_pieces();
        let their_pieces = self.board.get_their_pieces();
        let our_king = Square::from(our_pieces & pieces.kings);

        let rook_rays = self
            .move_generator
            .generate_rook_moves(our_king, Bitboard::default());
        let bishop_rays = self
            .move_generator
            .generate_bishop_moves(our_king, Bitboard::default());

        let mut snipers = their_pieces
            & ((pieces.queens | pieces.rooks) & rook_rays
                | (pieces.queens | pieces.bishops) & bishop_rays);

        let mut attack_lines = Bitboard::default();
        while !snipers.empty() {
            let sniper = snipers.pop_lsb();
            let blockers = pieces.all_pieces & Bitboard::between(Square::from(sniper), our_king);
            if blockers.count() == 1 && our_pieces.intersects(blockers) {
                pinned_pieces |= blockers;
            } else if blockers.count() == 0 {
                // If no blockers, this is a check from a sliding piece
                attack_lines |= Bitboard::between(Square::from(sniper), our_king);
            }
        }

        (pinned_pieces, attack_lines)
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
