use std::{
    cmp::{max, min},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant},
    thread::JoinHandle,
};
use crate::{constants::{move_ordering::MOVE_ORDERING_HISTORY_CAP, *}};
use crate::uci;
use crate::engine::{
    base::{
        _move::Move,
        player::Player,
    },
    board::{
        game::Game,
        rules::checks::*,
    },
    search::{
        evaluate::evaluate,
        move_ordering::order_moves,
        transposition_table::*,
        search_state::SearchState,
    },
};

#[derive(Clone, Default)]
pub struct Searcher {
    pub transposition_table: TranspositionTable,
    pub search_state: SearchState,
}

impl Searcher {
    pub fn order_moves(
        &self, game: &Game, moves: Vec<Move>,
        depth: usize, last_pv_move: Option<Move>
    ) -> Vec<Move> {
        let hash_move = self.transposition_table
            .probe(game.position.zobrist_hash)
            .and_then(|e| e.best_move);
        return order_moves(
            moves, &game.position,
            &self.search_state.killer_moves[depth],
            &self.search_state.history,
            hash_move,
            last_pv_move
        );
    }

    // Returns (best_move, best_eval, pv, unwind)
    // PV is REVERSED (leaf -> root), reverse it when printing to get normal root -> leaf
    fn minimax(
        &mut self,
        game: &mut Game,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        maximize: bool,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        nodes: &mut u64,
        last_pv_move: Option<Move>
    ) -> (Option<Move>, i32, Vec<Move>, bool) {
        *nodes += 1;

        // TODO: what to do with PV when looking up in TT?
        if let Some(tt_entry) = self.transposition_table.probe(game.position.zobrist_hash) {
            if tt_entry.depth >= depth as u8 {  // Use TT entry only if it is deep enough
                let return_value = (tt_entry.best_move, tt_entry.eval, vec![], false);
                match tt_entry.flag {
                    NodeType::Exact =>
                        return return_value,
                    NodeType::LowerBound => {
                        if tt_entry.eval >= beta {
                            return return_value;
                        }
                        alpha = alpha.max(tt_entry.eval);
                    }
                    NodeType::UpperBound => {
                        if tt_entry.eval <= alpha {
                            return return_value;
                        }
                        beta = beta.min(tt_entry.eval);
                    }
                }
            }
        }

        if game.is_threefold_repetition() ||
            game.is_fifty_move_rule() ||
            game.is_insufficient_material() {
            return (None, DRAW_EVAL, Vec::new(), false);
        }

        if depth == 0 {
            return (None, evaluate(&game.position), Vec::new(), false);
        }

        // Unwind the search if `stop_flag` was set or time is over
        // Check every 1024 nodes, because it is time-expensive
        if *nodes % 1024 == 0 {
            if stop_flag.load(Ordering::Relaxed) {
                return (None, evaluate(&game.position), Vec::new(), true);
            }

            if let Some(tl) = time_limit {
                if start_time.elapsed() >= tl {
                    return (None, evaluate(&game.position), Vec::new(), true);
                }
            }
        }

        let pseudo_moves = game.pseudo_moves();
        let sorted_pseudo_moves = self.order_moves(game, pseudo_moves, depth, last_pv_move);
        let mut best_eval = if maximize { i32::MIN } else { i32::MAX };
        let mut best_move = None;
        let mut best_pv = None;
        let mut found_legal_move = false;

        let alpha_orig = alpha;
        let beta_orig = beta;

        for m in &sorted_pseudo_moves {
            let legal = game.try_to_make_move(m);
            if !legal {
                continue;
            }

            found_legal_move = true;
            let (_best_response, eval, mut child_pv, unwind) = self.minimax(
                game, depth - 1, alpha, beta, !maximize, stop_flag,
                start_time, time_limit, nodes, last_pv_move
            );
            game.unmake_move();
            if unwind {
                return (None, best_eval, Vec::new(), true);
            }

            let is_better = if maximize {
                eval > best_eval
            } else {
                eval < best_eval
            };

            if is_better {
                best_eval = eval;
                best_move = Some(m);
                child_pv.push(*m);
                best_pv = Some(child_pv);
            }

            if maximize {
                alpha = max(alpha, eval);
            } else {
                beta = min(beta, eval);
            }

            if beta <= alpha {
                // TODO: count number of cutoffs globally
                // to check if move ordering is useful
                if !m.is_capture() && !m.is_promotion() {  // Filling killer moves
                    let killers = &mut self.search_state.killer_moves[depth];
                    if Some(*m) != killers[0] {
                        killers[1] = killers[0];
                        killers[0] = Some(*m);
                    }
                }

                // Update history heuristic
                let history_entry = &mut self.search_state.history[m.from as usize][m.to as usize];
                *history_entry = history_entry.saturating_add((depth * depth) as i32).clamp(0, MOVE_ORDERING_HISTORY_CAP) as i32;

                break;
            }
        }

        let flag = if best_eval <= alpha_orig {
            NodeType::UpperBound
        } else if best_eval >= beta_orig {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        self.transposition_table.insert(TTEntry {
            zobrist: game.position.zobrist_hash,
            depth: depth as u8,
            eval: best_eval,
            flag,
            best_move: best_move.copied(),
        });

        if !found_legal_move {
            // Checkmate
            if is_king_in_check(&game.position, game.position.player_to_move) {
                // losing sooner is worse
                let eval = match game.position.player_to_move {
                    Player::White => -CHECKMATE_EVAL + depth as i32,
                    Player::Black =>  CHECKMATE_EVAL - depth as i32,
                };
                return (None, eval, Vec::new(), false);
            } else {  // Draw
                return (None, DRAW_EVAL, Vec::new(), false);
            }
        }

        (best_move.copied(), best_eval, best_pv.unwrap(), false)
    }

    // Returns (best_move, best_score, nodes, pv, unwind)
    pub fn find_best_move(
        &mut self,
        game: &mut Game,
        depth: usize,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        last_pv_move: Option<Move>
    ) -> (Option<Move>, i32, u64, Vec<Move>, bool) {
        let maximize = match game.position.player_to_move {
            Player::White => true,
            Player::Black => false,
        };
        let mut nodes = 0;

        let (best_move, best_eval, pv, unwind) = self.minimax(
            game, depth, i32::MIN, i32::MAX, maximize,
            stop_flag, start_time, time_limit,
            &mut nodes, last_pv_move
        );

        (best_move, best_eval, nodes, pv, unwind)
    }

    pub fn iterative_deepening(
        &mut self,
        game: &mut Game,
        stop_flag: Arc<AtomicBool>,
        max_depth: Option<usize>,
        time_limit: Option<Duration>,
    ) -> Option<Move>
    {
        let mut last_move = None;
        let start = Instant::now();
        let mut last_pv_move = None;

        for depth in 1.. {
            if let Some(d) = max_depth {
                if depth > d {
                    break;
                }
            }

            let depth_start = Instant::now();
            let (m, eval, nodes, pv, unwind) = self.find_best_move(
                game,
                depth,
                &stop_flag,
                start,
                time_limit,
                last_pv_move
            );
            let elapsed = depth_start.elapsed();

            // Update the best move only if there was NO unwind (the depth was searched fully)
            if unwind {
                break;
            }

            last_move = m;
            last_pv_move = Some(pv[0]);
            uci::print_uci_info(depth, eval, nodes, pv, elapsed);

            if let Some(limit) = time_limit {
                if start.elapsed() >= limit {
                    break;
                }
            }
        }

        last_move
    }

    pub fn stop_search(
        stop_flag: &mut Arc<AtomicBool>,
        search_thread: &mut Option<JoinHandle<()>>,
    ) {
        stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = search_thread.take() {
            let _ = handle.join();
        }
        stop_flag.store(false, Ordering::Relaxed);
    }
}
