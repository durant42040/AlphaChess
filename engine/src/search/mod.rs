pub mod perft;

use std::cmp::max;

pub use perft::Perft;

use crate::chess::Move;
use crate::chess::r#move::MoveList;
use crate::engine::{Engine, Evaluation};

pub trait Search {
    fn order_moves(&mut self, moves: &mut MoveList);
    fn max_search(&mut self, depth: u8) -> i32;
    fn minimax_search(&mut self, depth: u8) -> i32;
    fn alpha_beta_search(&mut self, depth: u8, alpha: i32, beta: i32) -> i32;
    /// Returns the best move for the current player using alpha-beta search, or None if no legal moves.
    fn best_move(&mut self) -> Move;
}

impl Search for Engine {
    fn order_moves(&mut self, moves: &mut MoveList) {
        let pieces = self.pieces();
        moves.sort_by_key(|r#move| {
            let is_en_passant =
                pieces.pawns().get_square(r#move.from) && pieces.en_passant().get_square(r#move.to);
            let is_capture = self.board().their_pieces().get_square(r#move.to);
            let mut value = 0;
            if is_capture || is_en_passant {
                value = pieces.get_piece(r#move.to).unwrap().0.value()
            } else if self.is_square_under_attack(r#move.to) {
                value = -pieces.get_piece(r#move.from).unwrap().0.value();
            }
            (!is_capture && !is_en_passant, -value)
        });
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
            let score = self.eval();
            return score;
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
        if depth == 0 {
            return self.eval();
        }

        let mut moves = self.generate_all_legal_moves();
        self.order_moves(&mut moves);

        for r#move in moves {
            self.act(r#move);
            let score = self
                .alpha_beta_search(depth - 1, beta.saturating_neg(), alpha.saturating_neg())
                .saturating_neg();
            self.undo();
            if score >= beta {
                return beta;
            }
            alpha = max(alpha, score);
        }

        alpha
    }

    fn best_move(&mut self) -> Move {
        let depth = 5;
        let mut moves = self.generate_all_legal_moves();
        self.order_moves(&mut moves);

        let mut best_move = moves[0];
        let mut best_score = i32::MIN;

        let alpha = i32::MAX.saturating_neg();
        let beta = i32::MIN.saturating_neg();

        for r#move in moves {
            self.act(r#move);
            let score = self
                .alpha_beta_search(depth - 1, alpha, beta)
                .saturating_neg();
            self.undo();
            if score > best_score {
                best_score = score;
                best_move = r#move;
            }
        }
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
