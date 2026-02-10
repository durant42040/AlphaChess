pub mod eval;
pub mod history;
pub mod perft;
pub mod see;
pub mod transposition;

use std::cmp::max;
use std::time::{Duration, Instant};

pub use eval::Evaluation;

use crate::Engine;
use crate::chess::r#move::MoveList;
use crate::chess::{GameState, Move, Piece, Player};
use crate::constants::MATE_SCORE;
use crate::search::history::History;
use crate::search::transposition::{Bound, TranspositionTable};

pub struct Search {
    pub max_depth: u8,
    pub transposition_table: TranspositionTable,
    pub nodes: u64,
    pub start_time: Instant,
    pub ponder_time: Duration,
    pub max_depth_reached: u8,
    pub killer_moves: Vec<[Move; 2]>,
    pub history: History,
    /// used to evaluate ordering quality
    pub beta_cutoffs: u64,
}

impl Search {
    pub fn new() -> Self {
        Self {
            max_depth: u8::MAX,
            transposition_table: TranspositionTable::new(),
            nodes: 0,
            start_time: Instant::now(),
            ponder_time: Duration::from_millis(1000),
            max_depth_reached: 0,
            killer_moves: vec![[Move::none(); 2]; 4096],
            history: History::new(),
            // used to evaluate ordering quality
            beta_cutoffs: 0,
        }
    }

    pub fn set_max_depth(&mut self, max_depth: u8) {
        self.max_depth = max_depth;
    }

    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.max_depth_reached = 0;
        self.nodes = 0;
        self.beta_cutoffs = 0;
    }

    pub fn time_up(&self) -> bool {
        self.start_time.elapsed() >= self.ponder_time
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    fn print_search_info(&mut self, best_score: i32, best_move: Move) {
        let eval = if self.board.player() == Player::White {
            best_score
        } else {
            -best_score
        };

        // Principal variation
        let pv = self.principal_variation();
        let pv_str = if pv.is_empty() {
            "-".to_string()
        } else {
            pv.iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };

        // Score in pawns with sign
        let eval = eval as f32 / 100.0;
        let eval_str = if eval.is_sign_negative() {
            format!("{:.2}", eval)
        } else {
            format!("+{:.2}", eval)
        };
        let material = self.material_score() as f32 / 100.0;
        let material_str = if material.is_sign_negative() {
            format!("{:.2}", material)
        } else {
            format!("+{:.2}", material)
        };

        let endgame_color = if self.is_endgame() {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        };

        println!(
            "\n\x1b[1;32m[Engine]\x1b[0m\n\
             \x1b[90m────────────────────────────────────────────\x1b[0m\n\
             \x1b[1mDepth     \x1b[0m \x1b[33m{}\x1b[0m\n\
             \x1b[1mSearched  \x1b[0m \x1b[32m{}\x1b[0m \x1b[1mnodes\n\
             \x1b[1mEval      \x1b[0m \x1b[1;34m{}\x1b[0m\n\
             \x1b[1mBest Move \x1b[0m \x1b[33m{}\x1b[0m\n\
             \x1b[1mPV        \x1b[0m \x1b[33m{}\x1b[0m\n\
             \x1b[1mMaterial  \x1b[0m \x1b[1;34m{}\x1b[0m\n\
             \x1b[1mEndgame   \x1b[0m {}{}\x1b[0m\n\
             \x1b[90m────────────────────────────────────────────\x1b[0m",
            self.search.max_depth_reached,
            self.search.nodes,
            eval_str,
            best_move,
            pv_str,
            material_str,
            endgame_color,
            self.is_endgame()
        );
    }
    fn principal_variation(&mut self) -> Vec<Move> {
        let mut pv = Vec::new();
        let original_hash = self.board.position_hash();

        for _ in 0..self.search.max_depth_reached {
            let hash = self.board.position_hash();
            let r#move = self.search.transposition_table.get_best_move(hash);
            if r#move.is_none() || !self.is_legal_move(r#move) {
                break;
            }

            pv.push(r#move);
            assert!(self.is_legal_move(r#move));
            self.act(r#move);
        }

        for _ in 0..pv.len() {
            self.undo();
        }
        debug_assert_eq!(self.board.position_hash(), original_hash);

        pv
    }

    /// Move ordering improves search efficiency by prioritizing moves likely to cause beta cutoffs.
    /// Moves are sorted as follows:
    ///
    /// - promotion moves: moves that are promotions
    /// - good captures: moves that are captures and have a SEE score > 0
    /// - equal captures: moves that are captures and have a SEE score = 0
    /// - killer moves: 2 quiet moves that causes a beta cutoff
    /// - quiet moves: moves that are not captures, ordered by history heuristic
    /// - bad captures: moves that are captures and have a SEE score < 0
    fn order_moves(&mut self, moves: &MoveList, ply: usize) -> (MoveList, usize, usize) {
        let mut ordered_moves = MoveList::new();
        let pieces = self.pieces();
        let our_pawns = pieces.pawns();
        let en_passant = pieces.en_passant();
        let their_pieces = self.board.their_pieces();

        let tt_move = self
            .search
            .transposition_table
            .get_best_move(self.board.position_hash());
        if !tt_move.is_none() && self.is_legal_move(tt_move) {
            ordered_moves.push(tt_move);
        }

        let mut promotion_moves = Vec::with_capacity(moves.len());
        let mut good_captures = Vec::with_capacity(moves.len());
        let mut bad_captures = Vec::with_capacity(moves.len());
        let mut killer_moves = Vec::with_capacity(2);
        let mut quiet_moves = Vec::with_capacity(moves.len());

        for &r#move in moves.iter() {
            if r#move == tt_move {
                continue;
            }
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
            } else if r#move == self.search.killer_moves[ply][0] {
                // prioritize killer move 0 over killer move 1
                killer_moves.push((1, r#move));
            } else if r#move == self.search.killer_moves[ply][1] {
                killer_moves.push((0, r#move));
            } else {
                // order quiet moves by history score
                let score = self
                    .search
                    .history
                    .get(self.board.player(), r#move.from, r#move.to);
                quiet_moves.push((score, r#move));
            }
        }
        good_captures.sort_by_key(|(score, _)| -score);
        bad_captures.sort_by_key(|(score, _)| -score);
        killer_moves.sort_by_key(|(score, _)| -score);
        quiet_moves.sort_by_key(|(score, _)| -score);

        ordered_moves.extend(promotion_moves);
        ordered_moves.extend(good_captures.iter().map(|(_, r#move)| *r#move));
        let quiet_start = ordered_moves.len();

        ordered_moves.extend(killer_moves.iter().map(|(_, r#move)| *r#move));
        ordered_moves.extend(quiet_moves.iter().map(|(_, r#move)| *r#move));

        let quiet_end = ordered_moves.len();
        ordered_moves.extend(bad_captures.iter().map(|(_, r#move)| *r#move));

        debug_assert_eq!(ordered_moves.len(), moves.len());

        (ordered_moves, quiet_start, quiet_end)
    }

    /// alpha-beta search for captures only. bad captures are pruned. Capture scores are compared against current position evaluation.
    fn quiescence_search(&mut self, ply: usize, mut alpha: i32, beta: i32) -> i32 {
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
                return -MATE_SCORE + ply as i32;
            } else {
                return 0;
            }
        }

        for r#move in capture_moves {
            if self.see(r#move) < 0 {
                continue;
            }
            self.act(r#move);
            let score = -self.quiescence_search(ply + 1, -beta, -alpha);
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
            return self.quiescence_search(ply, alpha, beta);
        }

        // Null move pruning
        // assumes a "pass" is always worse than the best move
        // which is generally true with the exception of pawn endgames
        let in_check = self.is_check();
        if depth >= 3
            && !in_check
            && ply > 0
            // if there are friendly non-pawn pieces
            && ((self.board.our_pieces() & !self.pieces().pawns() & !self.pieces().kings()).count()
                > 0)
        {
            let r = 2;
            let prev_en_passant = self.make_null_move();
            let score = -self.alpha_beta_search(depth - 1 - r, ply + 1, -beta, -beta + 1);
            self.undo_null_move(prev_en_passant);

            if score >= beta {
                return beta;
            }
        }

        let mut best_score = i32::MIN;
        let mut best_move = Move::none();

        let moves = self.generate_all_legal_moves();

        // if there are no legal moves, check for checkmate or stalemate
        // mate in 1 have higher score than mate in 2 to encourage finding the fastest checkmate
        if moves.is_empty() {
            if self.is_check() {
                return -MATE_SCORE + ply as i32;
            } else {
                return 0;
            }
        }

        let (ordered_moves, quiet_start, quiet_end) = self.order_moves(&moves, ply);

        for (i, &r#move) in ordered_moves.iter().enumerate() {
            debug_assert!(self.is_legal_move(r#move));
            self.act(r#move);

            let mut score;
            // Principal Variation Search: Perform full search on the best move
            // Otherwise, search with null window
            if i == 0 {
                score = -self.alpha_beta_search(depth - 1, ply + 1, -beta, -alpha)
            } else {
                // Move reduction: If a quiet move is not a killer move, not a check, and not a top 3 move, search at reduced depth
                let gives_check = self.is_check();
                let r = if depth >= 5
                    && i >= quiet_start + 3
                    && i < quiet_end
                    && !gives_check
                    && !in_check
                {
                    1
                } else {
                    0
                };

                score = -self.alpha_beta_search(depth - 1 - r, ply + 1, -alpha - 1, -alpha);

                if score > alpha && score < beta {
                    score = -self.alpha_beta_search(depth - 1, ply + 1, -beta, -alpha);
                }
            }

            self.undo();

            if score > best_score {
                best_score = score;
                best_move = r#move;
            }

            alpha = max(alpha, score);

            if alpha >= beta {
                self.search.beta_cutoffs += 1;
                if i >= quiet_start && i < quiet_end {
                    // record killer moves for quiet moves (not captures, not promotions, not en passant) if it causes beta cutoff
                    let k = &mut self.search.killer_moves[ply];
                    if k[0] != r#move {
                        k[1] = k[0];
                        k[0] = r#move;
                    }

                    // increase history score by depth^2 for quiet moves
                    // The shallower cutoff (>depth) prunes more nodes
                    self.search
                        .history
                        .update(self.board.player(), r#move.from, r#move.to, depth);
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
        self.search.reset();

        let hash = self.board.position_hash();

        let mut alpha = -MATE_SCORE;
        let mut beta = MATE_SCORE;

        let mut best_move = Move::none();
        let mut best_score = 0;

        let mut depth: u8 = 1;
        let delta = 50;
        while !self.search.time_up() {
            best_score = self.alpha_beta_search(depth, 0, alpha, beta);

            // if the score is outside the alpha-beta window, research with full window and same depth
            if best_score <= alpha || best_score >= beta {
                best_score = self.alpha_beta_search(depth, 0, -MATE_SCORE, MATE_SCORE);
            }

            // It appears gradual widening window is slower than full window.
            // loop {
            //     // if the score is outside the alpha-beta window, research with expanded window and same depth
            //     // At depth one, it searches the full window. So the loop will only run once.
            //     let score = self.alpha_beta_search(depth, 0, alpha, beta);
            //     if score <= alpha {
            //         alpha -= window;
            //         window *= 2;
            //         continue;
            //     } else if score >= beta {
            //         beta += window;
            //         window *= 2;
            //         continue;
            //     } else {
            //         best_score = score;
            //         break;
            //     }
            // }

            alpha = best_score - delta;
            beta = best_score + delta;

            self.search.max_depth_reached = depth;
            best_move = self.search.transposition_table.get_best_move(hash);

            if depth >= self.search.max_depth || best_score >= MATE_SCORE - 100 {
                break;
            }
            depth += 1;
        }

        self.print_search_info(best_score, best_move);
        assert!(!best_move.is_none(), "Best move is none");

        best_move
    }
}

#[cfg(test)]
mod tests {
    use crate::{Engine, chess::GameState, constants::MATE_SCORE};

    #[test]
    fn test_middlegame() {
        let mut engine =
            Engine::from_fen("r1b2rk1/p1pp1p2/2p2p1p/2b5/4P3/2N5/PPP2PPP/R3KB1R w KQ - 0 11");
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
        engine.set_ponder_time(1000);
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
        engine.alpha_beta_search(8, 0, -MATE_SCORE, MATE_SCORE);
        // 1300834
        println!("nodes searched: {}", engine.search.nodes);
        // 48471
        println!("beta cutoffs: {}", engine.search.beta_cutoffs);
    }

    #[test]
    fn test_pv() {
        let mut engine =
            Engine::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2");
        engine.set_ponder_time(1000);
        engine.best_move();
        let pv = engine.principal_variation();
        println!(
            "pv: {}",
            pv.iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}
