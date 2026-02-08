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
use crate::chess::{GameState, Move, Piece, Player};
use crate::constants::MATE_SCORE;
use crate::search::transposition::{Bound, TranspositionTable};

pub struct Search {
    pub max_depth: u8,
    pub transposition_table: TranspositionTable,
    pub nodes: u64,
    pub start_time: Instant,
    pub ponder_time: Duration,
    pub max_depth_reached: u8,
    pub killer_moves: Vec<[Move; 2]>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            max_depth: u8::MAX,
            transposition_table: TranspositionTable::new(),
            nodes: 0,
            start_time: Instant::now(),
            ponder_time: Duration::from_millis(100),
            max_depth_reached: 0,
            killer_moves: vec![[Move::none(); 2]; 4096],
        }
    }

    pub fn set_max_depth(&mut self, max_depth: u8) {
        self.max_depth = max_depth;
    }

    pub fn reset_timer(&mut self) {
        self.start_time = Instant::now();
        self.max_depth_reached = 0;
    }

    pub fn time_up(&self) -> bool {
        self.start_time.elapsed() >= self.ponder_time
    }

    fn is_killer_move(&self, r#move: Move, ply: usize) -> bool {
        self.killer_moves[ply][0] == r#move || self.killer_moves[ply][1] == r#move
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
    /// - promotion moves: moves that are promotions
    /// - good captures: moves that are captures and have a SEE score > 0
    /// - equal captures: moves that are captures and have a SEE score = 0
    /// - killer moves: quiet moves that causes a beta cutoff
    /// - quiet moves: moves that are not captures
    /// - bad captures: moves that are captures and have a SEE score < 0
    fn order_moves(&mut self, moves: &MoveList, ply: usize) -> (MoveList, usize, usize) {
        let pieces = self.pieces();
        let our_pawns = pieces.pawns();
        let en_passant = pieces.en_passant();
        let their_pieces = self.board.their_pieces();

        let mut promotion_moves = Vec::with_capacity(moves.len());
        let mut good_captures = Vec::with_capacity(moves.len());
        let mut bad_captures = Vec::with_capacity(moves.len());
        let mut killer_moves = Vec::with_capacity(2);
        let mut quiet_moves = Vec::with_capacity(moves.len());

        for &r#move in moves.iter() {
            let is_capture = their_pieces.get_square(r#move.to);
            let is_en_passant =
                our_pawns.get_square(r#move.from) && en_passant.get_square(r#move.to);

            if r#move.promotion.is_some() {
                promotion_moves.push(r#move);
            } else if is_capture {
                let score = self.see(r#move);
                if score >= 0 {
                    good_captures.push((score, r#move));
                } else {
                    bad_captures.push((score, r#move));
                }
            } else if is_en_passant {
                good_captures.push((Piece::Pawn.value(), r#move));
            } else if self.search.is_killer_move(r#move, ply) {
                killer_moves.push(r#move);
            } else {
                quiet_moves.push(r#move);
            }
        }
        good_captures.sort_by_key(|(score, _)| -score);
        bad_captures.sort_by_key(|(score, _)| -score);

        let mut ordered_moves = MoveList::new();
        ordered_moves.extend(promotion_moves);
        ordered_moves.extend(good_captures.iter().map(|(_, r#move)| *r#move));
        let quiet_start = ordered_moves.len();
        ordered_moves.extend(killer_moves);
        ordered_moves.extend(quiet_moves);
        let quiet_end = ordered_moves.len();
        ordered_moves.extend(bad_captures.iter().map(|(_, r#move)| *r#move));

        debug_assert_eq!(ordered_moves.len(), moves.len());

        (ordered_moves, quiet_start, quiet_end)
    }

    /// alpha-beta search for captures only. bad captures are pruned. Capture scores are compared against current position evaluation.
    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        self.search.nodes += 1;
        if self.board.is_draw() {
            return 0;
        }
        let score = self.eval();
        if score >= beta {
            return score;
        }
        alpha = max(alpha, score);

        let capture_moves = self.generate_all_capture_moves();
        if capture_moves.is_empty() && self.generate_all_legal_moves().is_empty() {
            if self.is_check() {
                return -MATE_SCORE;
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
    fn alpha_beta_search(&mut self, depth: u8, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        self.search.nodes += 1;
        let alpha_orig = alpha;
        let hash = self.board.position_hash();
        if self.board.is_draw() {
            return 0;
        }

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
        // mate in 1 have higher score than mate in 2
        if moves.is_empty() {
            if self.is_check() {
                return -MATE_SCORE - depth as i32;
            } else {
                return 0;
            }
        }

        let tt_move = self.search.transposition_table.get_best_move(hash);

        // Validate that the transposition table move is legal before playing it
        if !tt_move.is_none() && self.is_legal_move(tt_move) {
            self.act(tt_move);
            let tt_score = self
                .alpha_beta_search(depth - 1, ply + 1, -beta, -alpha)
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

        let (ordered_moves, quiet_start, quiet_end) = self.order_moves(&moves, ply);

        for (i, &r#move) in ordered_moves.iter().enumerate() {
            if r#move == tt_move {
                continue;
            }
            debug_assert!(self.is_legal_move(r#move));
            self.act(r#move);

            let mut score;
            if depth >= 5
                // TODO: order quiet moves
                && i >= quiet_start + 2
                && i < quiet_end
                && !self.search.is_killer_move(r#move, ply)
                && !self.is_check()
            {
                // Move reduction: If a quiet move is not a killer move and not a check, search at reduced depth
                let r: u8 = 1;
                score = self
                    .alpha_beta_search(depth - 1 - r, ply + 1, -alpha - 1, -alpha)
                    .saturating_neg();
                if score > alpha {
                    score = self
                        .alpha_beta_search(depth - 1, ply + 1, -beta, -alpha)
                        .saturating_neg();
                }
            } else {
                score = self
                    .alpha_beta_search(depth - 1, ply + 1, -beta, -alpha)
                    .saturating_neg();
            }
            self.undo();

            if score > best_score {
                best_score = score;
                best_move = r#move;
            }
            alpha = max(alpha, score);

            if alpha >= beta {
                // record killer moves for quiet moves (not captures, not promotions, not en passant)
                if i >= quiet_start && i < quiet_end {
                    let k = &mut self.search.killer_moves[ply];
                    if k[0] != r#move {
                        k[1] = k[0];
                        k[0] = r#move;
                    }
                }
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

        self.search.reset_timer();

        let mut best_move = Move::none();
        let mut best_score = 0;

        let mut depth: u8 = 1;
        while !self.search.time_up() {
            best_score = self.alpha_beta_search(depth, 0, alpha, beta);

            self.search.max_depth_reached = depth;
            best_move = self.search.transposition_table.get_best_move(hash);

            if depth == self.search.max_depth {
                break;
            }
            depth += 1;
        }
        let eval = if self.board.player() == Player::White {
            best_score
        } else {
            -best_score
        };

        println!(
            "\x1b[1;32m[Engine]\x1b[0m\n\x1b[1msearched\x1b[0m \x1b[32m{}\x1b[0m nodes\n\
          \x1b[1mmax depth\x1b[0m \x1b[33m{}\x1b[0m\n\
          \x1b[1mbest move\x1b[0m \x1b[33m{}\x1b[0m\n\
          \x1b[1meval\x1b[0m \x1b[1;34m{}\x1b[0m",
            self.search.nodes, self.search.max_depth_reached, best_move, eval
        );

        assert!(!best_move.is_none(), "Best move is none");
        best_move
    }
}

#[cfg(test)]
mod tests {
    use crate::{Engine, chess::GameState};

    #[test]
    fn test_middlegame() {
        let mut engine =
            Engine::from_fen("rnbqkbnr/5ppp/1p6/4p3/p1p5/8/PPPPPPPP/1NBQKBNR b Kkq - 0 1");
        engine.set_ponder_time(10);
        let best_move = engine.best_move();
        println!("best move: {}", best_move);
    }

    #[test]
    fn test_endgame() {
        let mut engine = Engine::from_fen("Q7/3k5/8/8/6KP/2p5/2P3P1/8 b - - 0 1");
        engine.set_ponder_time(10);
        while engine.game_state() == GameState::Playing {
            let best_move = engine.best_move();
            engine.act(best_move);
            engine.update_game_state();
            println!("{}", engine);
        }
        assert_eq!(engine.game_state(), GameState::WhiteWin);
    }

    #[test]
    fn test_mate_in_one() {
        let mut engine = Engine::from_fen("8/8/8/8/8/q6k/8/7K b - - 0 1");
        engine.set_ponder_time(10);
        let best_move = engine.best_move();
        engine.act(best_move);
        engine.update_game_state();
        println!("{}", engine);
        assert_eq!(engine.game_state(), GameState::BlackWin);
    }

    #[test]
    fn test_nodes_searched() {
        let mut engine =
            Engine::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
        engine.set_ponder_time(1000);
        engine.alpha_beta_search(8, 0, i32::MAX.saturating_neg(), i32::MIN.saturating_neg());
        // 2191248 
        println!("nodes searched: {}", engine.search.nodes);
    }
}
