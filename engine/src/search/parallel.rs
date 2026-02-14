use std::{
    cmp::max,
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    Engine,
    chess::{Move, Player},
    constants::MATE_SCORE,
    search::Evaluation,
};

pub type SearchResult = (Move, i32, u8, u64);

impl Engine {
    fn print_parallel_search_info(
        &mut self,
        best_move: Move,
        best_score: i32,
        max_depth: u8,
        total_nodes: u64,
    ) {
        let eval = if self.board.player() == Player::White {
            best_score
        } else {
            -best_score
        };
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

        let pv = self.principal_variation_from(best_move, max_depth);
        let pv_str = if pv.is_empty() {
            "-".to_string()
        } else {
            pv.iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };

        println!(
            "\n\x1b[1;32m[Engine]\x1b[0m\n\
             \x1b[90m────────────────────────────────────────────\x1b[0m\n\
             \x1b[1mDepth     \x1b[0m \x1b[33m{}\x1b[0m\n\
             \x1b[1mSearched  \x1b[0m \x1b[32m{}\x1b[0m \x1b[1mnodes\n\
             \x1b[1mBest Move \x1b[0m \x1b[33m{}\x1b[0m\n\
             \x1b[1mPV        \x1b[0m \x1b[33m{}\x1b[0m\n\
             \x1b[1mEval      \x1b[0m \x1b[1;34m{}\x1b[0m\n\
             \x1b[1mMaterial  \x1b[0m \x1b[1;34m{}\x1b[0m\n\
             \x1b[1mEndgame   \x1b[0m {}{}\x1b[0m\n\
             \x1b[90m────────────────────────────────────────────\x1b[0m",
            max_depth,
            total_nodes,
            best_move,
            pv_str,
            eval_str,
            material_str,
            endgame_color,
            self.is_endgame()
        );
    }

    pub fn worker_search(&mut self, stop: &AtomicBool, delta_offset: i32) -> (Move, i32, u8, u64) {
        let hash = self.board.position_hash();

        let mut alpha = -MATE_SCORE;
        let mut beta = MATE_SCORE;

        let delta = 50 + delta_offset;

        let mut best_move = Move::none();
        let mut best_score = 0i32;

        let mut depth: u8 = 1;
        while !stop.load(Ordering::Relaxed)
            && depth <= self.search.max_depth
            && best_score < MATE_SCORE - 100
        {
            best_score = self.alpha_beta_search(depth, 0, alpha, beta);
            if best_score <= alpha || best_score >= beta {
                best_score = self.alpha_beta_search(depth, 0, -MATE_SCORE, MATE_SCORE);
            }

            alpha = best_score - delta;
            beta = best_score + delta;

            self.search.max_depth_reached = depth;
            best_move = self.search.transposition_table.get_best_move(hash);

            depth += 1;
        }

        (
            best_move,
            best_score,
            self.search.max_depth_reached,
            self.search.nodes,
        )
    }

    pub fn vote_best_move(&mut self, search_results: Vec<SearchResult>) -> Move {
        let min_score = search_results.iter().map(|(_, s, _, _)| *s).min().unwrap();
        let max_score = search_results.iter().map(|(_, s, _, _)| *s).max().unwrap();

        // skip voting if checkmate is found, in which case the best move is likely shallow, so voting is not necessary
        if max_score > MATE_SCORE - 100 || min_score < -MATE_SCORE + 100 {
            let (best_move, best_score, max_depth, total_nodes) =
                *search_results.iter().max_by_key(|(_, s, _, _)| *s).unwrap();
            self.print_parallel_search_info(best_move, best_score, max_depth, total_nodes);
            return best_move;
        }

        // voting for best move based on score and depth
        let mut max_depth = 0;
        let mut total_nodes = 0u64;

        let mut votes: HashMap<Move, i64> = HashMap::new();
        for (best_move, score, depth, nodes) in &search_results {
            let vote = (*score - min_score + 14) as i64 * (*depth as i64);
            *votes.entry(*best_move).or_insert(0) += vote;

            max_depth = max(max_depth, *depth);
            total_nodes += nodes;
        }

        let best_move = votes.into_iter().max_by_key(|(_, vote)| *vote).unwrap().0;
        let best_score = search_results
            .iter()
            .find(|(r#move, _, _, _)| *r#move == best_move)
            .unwrap()
            .1;

        self.print_parallel_search_info(best_move, best_score, max_depth, total_nodes);

        best_move
    }
}
