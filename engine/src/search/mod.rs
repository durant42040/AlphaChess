pub mod eval;
pub mod perft;
pub mod see;
pub mod transposition;

use std::cmp::max;

pub use eval::Evaluation;
pub use perft::Perft;

use crate::Engine;
use crate::chess::r#move::MoveList;
use crate::chess::{GameState, Move, Piece};
use crate::search::transposition::{Bound, TranspositionTable};

pub struct Search {
    transposition_table: TranspositionTable,
    depth: u8,
    nodes: u64,
}

impl Search {
    pub fn new() -> Self {
        Self {
            transposition_table: TranspositionTable::new(),
            depth: 7,
            nodes: 0,
        }
    }

    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }

    pub fn nodes(&self) -> u64 {
        self.nodes
    }
}

impl Engine {
    /// Move ordering improves search efficiency by prioritizing moves likely to cause beta cutoffs.
    /// Moves are sorted as follows:
    ///
    /// - good captures: moves that are captures and have a SEE score > 0
    /// - equal captures: moves that are captures and have a SEE score = 0
    /// - quiet moves: moves that are not captures
    /// - bad captures: moves that are captures and have a SEE score < 0
    fn order_moves(&mut self, moves: &MoveList) -> MoveList {
        let pieces = self.pieces();
        let our_pawns = pieces.pawns();
        let en_passant = pieces.en_passant();
        let their_pieces = self.board.their_pieces();

        let mut good_captures = Vec::with_capacity(moves.len());
        let mut bad_captures = Vec::with_capacity(moves.len());
        let mut quiet_moves = Vec::with_capacity(moves.len());

        for &r#move in moves.iter() {
            let is_en_passant =
                our_pawns.get_square(r#move.from) && en_passant.get_square(r#move.to);
            let is_capture = their_pieces.get_square(r#move.to);

            if is_capture {
                let score = self.see(r#move);
                if score >= 0 {
                    good_captures.push((score, r#move));
                } else {
                    bad_captures.push((score, r#move));
                }
            } else if is_en_passant {
                good_captures.push((Piece::Pawn.value(), r#move));
            } else {
                quiet_moves.push(r#move);
            }
        }
        good_captures.sort_by_key(|(score, _)| -score);
        bad_captures.sort_by_key(|(score, _)| -score);

        let mut ordered_moves = MoveList::new();
        ordered_moves.extend(good_captures.iter().map(|(_, r#move)| *r#move));
        ordered_moves.extend(quiet_moves);
        ordered_moves.extend(bad_captures.iter().map(|(_, r#move)| *r#move));

        debug_assert_eq!(ordered_moves.len(), moves.len());

        ordered_moves
    }

    /// alpha-beta search for the captures only. bad captures are pruned. Capture scores are compared against current position evaluation.
    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        self.search.nodes += 1;
        let score = self.board.score();
        if score >= beta {
            return score;
        }
        alpha = max(alpha, score);

        let capture_moves = self.generate_all_capture_moves();
        if capture_moves.is_empty() && self.generate_all_legal_moves().is_empty() {
            if self.is_check() {
                return i32::MIN;
            } else {
                return 0;
            }
        }

        for r#move in capture_moves {
            if self.see(r#move) < 0 {
                continue;
            }
            self.act(r#move);
            let score = self.quiescence_search(-beta, -alpha).saturating_neg();
            self.undo();
            if score >= beta {
                return beta;
            }
            alpha = max(alpha, score);
        }

        alpha
    }

    /// Performs alpha-beta pruning search. Returns the best score for the current player.
    ///
    /// - `alpha`: minimum score for the maximizing player
    /// - `beta`: maximum score for the minimizing player
    ///
    /// The evauluation result is stored in the transposition table. Essentially, this is the memoization step in dynamic programming.
    /// Alpha–beta search usually does not compute the true value of a node. It stops early if the score is outside the alpha-beta window.
    /// The bound of a position denotes whether the score is exact, upper, or lower bound of the true value.
    /// 1. Exact: the score is the true value of the position.
    /// 2. Lower: the score is a lower bound of the position, and is stopped early when `score <= alpha`.
    /// 3. Upper: the score is a upper bound of the position, when `score >= beta`.
    fn alpha_beta_search(&mut self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        self.search.nodes += 1;
        let alpha_orig = alpha;
        let hash = self.board.position_hash();

        if let Some(score) = self
            .search
            .transposition_table
            .probe(hash, depth, alpha, beta)
        {
            return score;
        }

        if depth == 0 {
            return self.quiescence_search(alpha, beta);
        }

        let mut best_score = i32::MIN;
        let mut best_move = Move::none();

        let moves = self.generate_all_legal_moves();

        // if there are no legal moves, check for checkmate or stalemate
        if moves.is_empty() {
            if self.is_check() {
                return i32::MIN;
            } else {
                return 0;
            }
        }

        let tt_move: Move = self.search.transposition_table.get_best_move(hash);

        if !tt_move.is_none() {
            self.act(tt_move);
            let tt_score = self
                .alpha_beta_search(depth - 1, -beta, -alpha)
                .saturating_neg();
            self.undo();

            if tt_score > best_score {
                best_score = tt_score;
                best_move = tt_move;
            }
            alpha = max(alpha, tt_score);
            if alpha >= beta {
                return best_score;
            }
        }

        let ordered_moves = self.order_moves(&moves);

        for r#move in ordered_moves {
            if r#move == tt_move {
                continue;
            }
            self.act(r#move);
            let score = self
                .alpha_beta_search(depth - 1, -beta, -alpha)
                .saturating_neg();
            self.undo();

            if score > best_score {
                best_score = score;
                best_move = r#move;
            }
            alpha = max(alpha, score);

            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        self.search
            .transposition_table
            .store(hash, depth, best_score, bound, best_move);

        best_score
    }

    pub fn best_move(&mut self) -> Move {
        debug_assert!(self.game_state == GameState::Playing);
        self.search.nodes = 0;
        let hash = self.board.position_hash();

        let alpha = i32::MAX.saturating_neg();
        let beta = i32::MIN.saturating_neg();

        self.alpha_beta_search(self.search.depth, alpha, beta);

        let best_move = self.search.transposition_table.get_best_move(hash);
        println!("searched {} nodes", self.search.nodes);
        debug_assert!(!best_move.is_none(), "Best move is none");
        best_move
    }
}
