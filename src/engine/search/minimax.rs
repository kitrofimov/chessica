use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant},
};
use crate::{constants::{move_ordering::MOVE_ORDERING_HISTORY_CAP, *}};
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
        transposition_table::*,
        searcher::Searcher,
    },
};

pub struct SearchContext {
    pub stop_flag: Arc<AtomicBool>,
    pub start_time: Instant,
    pub time_limit: Option<Duration>,
}

struct SearchResultInternal {
    best_move: Option<Move>,
    eval: i32,
    pv: Vec<Move>,  // REVERSED (leaf -> root)
    was_unwinded: bool,
}

pub struct SearchResult {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub pv: Vec<Move>,
    pub was_unwinded: bool,
    pub nodes: u64,
}

impl Searcher {
    fn minimax(
        &mut self,
        game: &mut Game,
        last_pv_move: Option<Move>,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        maximize: bool,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        nodes: &mut u64,
    ) -> SearchResultInternal {
        *nodes += 1;

        if self.should_stop(stop_flag, start_time, time_limit, *nodes) {
            return SearchResultInternal {
                best_move: None,
                eval: evaluate(&game.position),
                pv: vec![],
                was_unwinded: true,
            };
        }

        if let Some(result) = self.lookup_tt(game, depth, &mut alpha, &mut beta) {
            return result;
        }

        if game.is_draw() {
            return SearchResultInternal {
                best_move: None,
                eval: DRAW_EVAL,
                pv: vec![],
                was_unwinded: false,
            };
        }

        if depth == 0 {
            return SearchResultInternal {
                best_move: None,
                eval: evaluate(&game.position),
                pv: vec![],
                was_unwinded: false,
            };
        }

        self.search_moves(game, last_pv_move, depth, alpha, beta, maximize,
                          stop_flag, start_time, time_limit, nodes)
    }

    // Check if stop_flag was set or time is over
    fn should_stop(
        &self,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        nodes: u64
    ) -> bool {
        // Check every 1024 nodes, because it is time-expensive
        if nodes % 1024 == 0 {
            if stop_flag.load(Ordering::Relaxed) {
                return true;
            }
            if let Some(tl) = time_limit {
                if start_time.elapsed() >= tl {
                    return true;
                }
            }
        }
        false
    }

    fn lookup_tt(
        &mut self,
        game: &Game,
        depth: usize,
        alpha: &mut i32,
        beta: &mut i32,
    ) -> Option<SearchResultInternal> {
        if let Some(tt_entry) = self.transposition_table.probe(game.position.zobrist_hash) {
            if tt_entry.depth < depth as u8 {
                return None;
            }

            let possible_result = SearchResultInternal {
                best_move: tt_entry.best_move,
                eval: tt_entry.eval,
                pv: vec![],
                was_unwinded: false,
            };

            match tt_entry.flag {
                NodeType::Exact => return Some(possible_result),
                NodeType::LowerBound => {
                    if tt_entry.eval >= *beta {
                        return Some(possible_result);
                    }
                    *alpha = (*alpha).max(tt_entry.eval);
                }
                NodeType::UpperBound => {
                    if tt_entry.eval <= *alpha {
                        return Some(possible_result);
                    }
                    *beta = (*beta).min(tt_entry.eval);
                }
            }
        }
        None
    }

    fn search_moves(
        &mut self,
        game: &mut Game,
        last_pv_move: Option<Move>,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        maximize: bool,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        nodes: &mut u64,
    ) -> SearchResultInternal {
        let alpha_orig = alpha;
        let beta_orig = beta;
        let mut best_eval = if maximize { i32::MIN } else { i32::MAX };
        let mut best_move = None;
        let mut best_pv = None;
        let mut found_legal = false;

        let mut pseudo_moves = game.pseudo_moves();
        self.order_moves(game, &mut pseudo_moves, depth, last_pv_move);

        for m in pseudo_moves {
            if !game.try_to_make_move(&m) {
                continue;
            }

            found_legal = true;

            let result = self.minimax(
                game, Some(m), depth - 1, alpha, beta, !maximize,
                stop_flag, start_time, time_limit, nodes
            );
            let eval = result.eval;
            let mut child_pv = result.pv;
            let was_unwinded = result.was_unwinded;

            game.unmake_move();

            if was_unwinded {
                return SearchResultInternal {
                    best_move: None,
                    eval: best_eval,
                    pv: vec![],
                    was_unwinded: true,
                };
            }

            let better = (maximize && eval > best_eval) || (!maximize && eval < best_eval);
            if better {
                best_eval = eval;
                best_move = Some(m);
                child_pv.push(m);
                best_pv = Some(child_pv);
            }

            if maximize { alpha = alpha.max(eval); } else { beta = beta.min(eval); }

            if beta <= alpha {
                self.update_killers_and_history(&m, depth);
                break;
            }
        }

        if !found_legal {
            return self.handle_no_legal_moves(game, depth);
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
            best_move,
        });

        SearchResultInternal {
            best_move,
            eval: best_eval,
            pv: best_pv.unwrap_or_default(),
            was_unwinded: false,
        }
    }

    fn handle_no_legal_moves(&self, game: &Game, depth: usize) -> SearchResultInternal {
        let eval = if is_king_in_check(&game.position, game.position.player_to_move) {
            match game.position.player_to_move {
                Player::White => -CHECKMATE_EVAL + depth as i32,
                Player::Black => CHECKMATE_EVAL - depth as i32,
            }
        } else {
            DRAW_EVAL
        };

        SearchResultInternal {
            best_move: None,
            eval,
            pv: vec![],
            was_unwinded: false,
        }
    }

    fn update_killers_and_history(&mut self, m: &Move, depth: usize) {
        if !m.is_capture() && !m.is_promotion() {
            let killers = &mut self.killer_moves[depth];
            if Some(*m) != killers[0] {
                killers[1] = killers[0];
                killers[0] = Some(*m);
            }
        }

        let hist = &mut self.history[m.from as usize][m.to as usize];
        *hist = hist.saturating_add((depth * depth) as i32)
            .clamp(0, MOVE_ORDERING_HISTORY_CAP);
    }

    // Wrapper that helps set initial parameters for minimax recursion
    pub fn minimax_wrapper(
        &mut self,
        game: &mut Game,
        depth: usize,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        last_pv_move: Option<Move>
    ) -> SearchResult {
        let maximize = match game.position.player_to_move {
            Player::White => true,
            Player::Black => false,
        };
        let mut nodes = 0;

        let result = self.minimax(
            game, last_pv_move, depth, i32::MIN, i32::MAX, maximize,
            stop_flag, start_time, time_limit, &mut nodes
        );

        SearchResult {
            best_move: result.best_move,
            eval: result.eval,
            pv: result.pv.into_iter().rev().collect(),  // Reverse PV to be root -> leaf
            was_unwinded: result.was_unwinded,
            nodes,
        }
    }
}
