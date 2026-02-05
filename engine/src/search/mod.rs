pub mod eval;
pub mod perft;
pub mod see;
pub mod transposition;

use std::cmp::max;

pub use eval::Evaluation;
pub use perft::Perft;
pub use transposition::TranspositionTable;

use crate::Engine;
use crate::chess::r#move::MoveList;
use crate::chess::{Move, Piece};
use crate::search::transposition::Bound;

pub trait Search {
    fn order_moves(&mut self, moves: &mut MoveList);
    fn quiescence_search(&mut self, alpha: i32, beta: i32) -> i32;
    fn alpha_beta_search(&mut self, depth: u8, alpha: i32, beta: i32) -> i32;
    fn best_move(&mut self) -> Move;
}

impl Search for Engine {
    /// Move ordering improves search efficiency by prioritizing moves likely to cause beta cutoffs.
    /// Moves are sorted as follows:
    ///
    /// - **Captures**: Moves that capture opponent pieces are given higher priority, and within captures,
    ///   the most valuable victim is sorted first (MVV: Most Valuable Victim principle).
    fn order_moves(&mut self, moves: &mut MoveList) {
        let pieces = self.pieces();

        moves.sort_by_key(|r#move| {
            let is_en_passant =
                pieces.pawns().get_square(r#move.from) && pieces.en_passant().get_square(r#move.to);
            let is_capture = self.board.their_pieces().get_square(r#move.to);

            let value = if is_capture {
                pieces.value(r#move.to)
            } else if is_en_passant {
                Piece::Pawn.value()
            } else {
                0
            };

            (!is_capture && !is_en_passant, -value)
        });
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let score = self.eval();
        if score >= beta {
            return score;
        }
        alpha = max(alpha, score);

        let mut moves = self.generate_all_capture_moves();
        self.order_moves(&mut moves);

        for r#move in moves {
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

    /// Performs alpha-beta pruning search.
    ///
    /// - `alpha`: minimum score for the maximizing player
    /// - `beta`: maximum score for the minimizing player
    fn alpha_beta_search(&mut self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        let alpha_orig = alpha;
        let hash = self.board.position_hash();

        if let Some(score) = self.transposition_table.probe(hash, depth, alpha, beta) {
            return score;
        }

        if depth == 0 {
            return self.quiescence_search(alpha, beta);
        }

        let mut best_score = i32::MIN;
        let mut best_move = Move::none();

        let mut moves = self.generate_all_legal_moves();
        let tt_move: Move = self.transposition_table.get_best_move(hash);

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

        self.order_moves(&mut moves);

        for r#move in moves {
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

        self.transposition_table
            .store(hash, depth, best_score, bound, best_move);

        best_score
    }

    fn best_move(&mut self) -> Move {
        let depth = 6;
        let hash = self.board.position_hash();
        let alpha = i32::MAX.saturating_neg();
        let beta = i32::MIN.saturating_neg();

        self.alpha_beta_search(depth, alpha, beta);

        let best_move = self.transposition_table.get_best_move(hash);
        debug_assert!(!best_move.is_none(), "Best move is none");
        best_move
    }
}
