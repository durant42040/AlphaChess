pub mod eval;
pub mod perft;
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
    fn max_search(&mut self, depth: u8) -> i32;
    fn minimax_search(&mut self, depth: u8) -> i32;
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
                pieces.get_piece(r#move.to).unwrap().0.value()
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

    /// Performs max search.
    fn max_search(&mut self, depth: u8) -> i32 {
        if depth == 0 {
            let score = self.eval();
            return score;
        }

        let mut score = i32::MIN;

        let mut moves = self.generate_all_legal_moves();
        self.order_moves(&mut moves);

        for r#move in moves {
            self.act(r#move);
            score = max(score, self.max_search(depth - 1));
            self.undo();
        }

        score
    }

    /// Performs minimax search.
    fn minimax_search(&mut self, depth: u8) -> i32 {
        if depth == 0 {
            return self.eval();
        }

        let mut score = i32::MIN;
        let mut moves = self.generate_all_legal_moves();
        self.order_moves(&mut moves);

        for r#move in moves {
            self.act(r#move);
            score = max(score, -self.minimax_search(depth - 1));
            self.undo();
        }

        score
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

        let mut moves = self.generate_all_legal_moves();
        self.order_moves(&mut moves);

        for mv in moves {
            self.act(mv);
            let score = self
                .alpha_beta_search(depth - 1, -beta, -alpha)
                .saturating_neg();
            self.undo();

            best_score = max(best_score, score);
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
            .store(hash, depth, best_score, bound);

        best_score
    }

    fn best_move(&mut self) -> Move {
        let depth = 6;
        let mut moves = self.generate_all_legal_moves();
        self.order_moves(&mut moves);

        let alpha = i32::MAX.saturating_neg();
        let beta = i32::MIN.saturating_neg();

        let (best_move, _best_score) = moves
            .iter()
            .map(|&r#move| {
                self.act(r#move);
                let score = self
                    .alpha_beta_search(depth - 1, alpha, beta)
                    .saturating_neg();
                self.undo();
                (r#move, score)
            })
            .max_by_key(|&(_, score)| score)
            .expect("at least one legal move");

        best_move
    }
}

#[test]
fn test_max_search() {
    // look for 4-move checkmate
    let mut engine = Engine::new();
    let best_score = engine.max_search(5);
    assert_eq!(best_score, i32::MAX);
}
