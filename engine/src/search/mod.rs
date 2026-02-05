pub mod eval;
pub mod perft;
pub mod see;
pub mod transposition;

use std::cmp::max;
use std::time::{Duration, Instant};

pub use eval::Evaluation;
pub use perft::Perft;

use crate::Engine;
use crate::chess::r#move::MoveList;
use crate::chess::{GameState, Move, Piece};
use crate::search::transposition::{Bound, TranspositionTable};

pub struct Search {
    pub max_depth: u8,
    pub transposition_table: TranspositionTable,
    pub nodes: u64,
    pub start_time: Instant,
    pub time_limit: Duration,
    pub max_depth_reached: u8,
}

impl Search {
    pub fn new() -> Self {
        Self {
            max_depth: u8::MAX,
            transposition_table: TranspositionTable::new(),
            nodes: 0,
            start_time: Instant::now(),
            time_limit: Duration::from_secs(1),
            max_depth_reached: 0,
        }
    }

    pub fn set_max_depth(&mut self, max_depth: u8) {
        self.max_depth = max_depth;
    }

    pub fn reset_timer(&mut self, time_limit: Duration) {
        self.start_time = Instant::now();
        self.time_limit = time_limit;
        self.max_depth_reached = 0;
    }

    pub fn time_up(&self) -> bool {
        self.start_time.elapsed() >= self.time_limit
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
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

    /// alpha-beta search for captures only. bad captures are pruned. Capture scores are compared against current position evaluation.
    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        if self.search.time_up() {
            return self.board.score();
        }
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
        if self.search.time_up() {
            return self.board.score();
        }
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

        let tt_move = self.search.transposition_table.get_best_move(hash);

        // Validate that the transposition table move is legal before playing it
        if !tt_move.is_none() && moves.contains(&tt_move) {
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
        assert!(self.game_state == GameState::Playing);
        self.search.nodes = 0;
        let hash = self.board.position_hash();

        let alpha = i32::MAX.saturating_neg();
        let beta = i32::MIN.saturating_neg();

        self.search.reset_timer(Duration::from_secs(1));

        let mut best_move = Move::none();

        let mut depth: u8 = 1;
        while !self.search.time_up() {
            self.alpha_beta_search(depth, alpha, beta);

            self.search.max_depth_reached = depth;

            if self.search.time_up() {
                break;
            }
            best_move = self.search.transposition_table.get_best_move(hash);

            if depth == self.search.max_depth {
                break;
            }

            depth += 1;
        }

        println!(
            "searched {} nodes\nmax depth: {}",
            self.search.nodes, self.search.max_depth_reached
        );
        assert!(!best_move.is_none(), "Best move is none");
        best_move
    }
}
