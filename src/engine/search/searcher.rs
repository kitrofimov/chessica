use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant},
    thread::JoinHandle,
};
use crate::constants::{move_ordering::*, evaluation::*, *};
use crate::engine::{
    base::_move::Move,
    board::game::Game,
    search::{
        evaluate::*,
        transposition_table::*,
    },
};

/// Structure orchestrating the search process
#[derive(Clone)]
pub struct Searcher {
    // Move ordering heuristics
    pub transposition_table: TranspositionTable,
    pub history: [[i32; 64]; 64],  // [from][to]
    pub killer_moves: [[Option<Move>; 2]; SEARCH_MAX_PLY_DEPTH],
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            transposition_table: TranspositionTable::default(),
            history: [[0; 64]; 64],
            killer_moves: [[None; 2]; SEARCH_MAX_PLY_DEPTH],
        }
    }
}

pub struct SearchContext {
    pub stop_flag: Arc<AtomicBool>,
    pub start_time: Instant,
    pub time_limit: Option<Duration>,
    pub last_pv_move: Option<Move>,
}

struct SearchResultInternal {
    best_move: Option<Move>,
    eval: Evaluation,
    pv: Vec<Move>,
    was_unwinded: bool,
}

impl SearchResultInternal {
    fn unwinded() -> Self {
        SearchResultInternal {
            best_move: None,
            eval: 0,  // unwind makes whole search irrelevant, so 0 here
            pv: vec![],
            was_unwinded: true,
        }
    }
}

pub struct SearchResult {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub pv: Vec<Move>,
    pub was_unwinded: bool,
    pub nodes: u64,
}

impl Searcher {
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

    fn negamax(
        &mut self,
        game: &mut Game,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        nodes: &mut u64,
        ctx: &SearchContext,
    ) -> SearchResultInternal {
        *nodes += 1;
        let orig_alpha = alpha;
        let orig_beta  = beta;

        if self.should_stop(nodes, ctx) {
            return SearchResultInternal::unwinded();
        }

        if let Some(tt_result) = self.lookup_tt(game, depth, &mut alpha, &mut beta) {
            return tt_result;
        }

        if depth == 0 {
            return self.quiescence(game, alpha, beta, nodes, ctx);
        }

        if game.is_draw() {
            return SearchResultInternal {
                best_move: None,
                eval: DRAW_EVAL,
                pv: vec![],
                was_unwinded: false,
            };
        }

        let mut pseudo_moves = game.pseudo_moves();
        if pseudo_moves.is_empty() {
            return self.handle_no_legal_moves(game, depth)
        }

        self.order_moves(game, &mut pseudo_moves, Some(depth));

        let mut best_eval = -EVAL_INF;
        let mut best_move = None;
        let mut best_pv = Vec::new();
        let mut found_legal = false;

        for mv in pseudo_moves {
            if game.try_to_make_move(&mv) == false {
                continue;
            }

            found_legal = true;
            let subtree = self.negamax(game, depth - 1, -beta, -alpha, nodes, ctx);
            game.unmake_move();

            if subtree.was_unwinded {
                return SearchResultInternal::unwinded();
            }

            let eval = -subtree.eval;
            if eval > best_eval {
                best_eval = eval;
                best_move = Some(mv);
                best_pv.clear();
                best_pv.push(mv);
                best_pv.extend(subtree.pv);

                // Update history heuristic if doesn't cause beta cutoff
                if eval < beta && !mv.is_capture() && !mv.is_promotion() {
                    let hist = &mut self.history[mv.from as usize][mv.to as usize];
                    *hist = hist.saturating_add((depth * depth) as i32)
                        .clamp(0, MOVE_ORDERING_HISTORY_CAP);
                }
            }

            if best_eval > alpha {
                alpha = best_eval;
            }

            if alpha >= beta {  // beta cutoff
                // Update killer heuristic
                if !mv.is_capture() && !mv.is_promotion() {
                    let killers = &mut self.killer_moves[depth];
                    if Some(mv) != killers[0] {
                        killers[1] = killers[0];
                        killers[0] = Some(mv);
                    }
                }
                break;
            }
        }

        if !found_legal {
            return self.handle_no_legal_moves(game, depth)
        }

        let flag = if best_eval <= orig_alpha {
            NodeType::UpperBound
        } else if best_eval >= orig_beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        self.transposition_table.insert(TTEntry {
            zobrist: game.position.zobrist_hash,
            depth: depth as u8,
            eval: best_eval,
            flag,
            best_move
        });

        SearchResultInternal {
            best_move,
            eval: best_eval,
            pv: best_pv,
            was_unwinded: false,
        }
    }

    fn lookup_tt(
        &mut self,
        game: &Game,
        depth: usize,
        alpha: &mut i32,
        beta: &mut i32,
    ) -> Option<SearchResultInternal> {
        if let Some(tt_entry) = self.transposition_table.probe(game.position.zobrist_hash) {
            if tt_entry.depth < depth as u8 {  // Not deep enough
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

    fn handle_no_legal_moves(&self, game: &Game, depth: usize) -> SearchResultInternal {
        SearchResultInternal {
            best_move: None,
            eval: if game.position.is_king_in_check(game.position.player_to_move) {
                // losing sooner is worse (depth is lower at the leafs & eval is relative to the current player)
                -CHECKMATE_EVAL - depth as i32
            } else {
                DRAW_EVAL
            },
            pv: vec![],  // no future moves available
            was_unwinded: false,
        }
    }

    fn quiescence(
        &mut self,
        game: &mut Game,
        mut alpha: i32,
        beta: i32,
        nodes: &mut u64,
        ctx: &SearchContext,
    ) -> SearchResultInternal {
        *nodes += 1;

        if self.should_stop(nodes, ctx) {
            return SearchResultInternal::unwinded();
        }

        let stand_pat = game.position.evaluate();
        if stand_pat >= beta {
            return SearchResultInternal {
                best_move: None,
                eval: stand_pat,
                pv: vec![],
                was_unwinded: false,
            };
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // Generating only captures and promotions
        let mut pseudo_captures = game.pseudo_moves()
            .into_iter()
            .filter(|m| match () {
                _ if m.is_promotion() => true,
                _ if m.is_capture()   => 
                    game.position.static_exchange_eval(*m) >= SEE_QUIESCENCE_SEARCH_LOWER_BOUND,
                _ => false,
            })
            .collect::<Vec<_>>();
        self.order_moves(game, &mut pseudo_captures, None);

        for mv in pseudo_captures {
            if game.try_to_make_move(&mv) == false {
                continue;
            }

            let subtree = self.quiescence(game, -beta, -alpha, nodes, ctx);
            game.unmake_move();

            if subtree.was_unwinded {
                return SearchResultInternal::unwinded();
            }

            let eval = -subtree.eval;
            if eval >= beta {
                return SearchResultInternal {
                    best_move: Some(mv),
                    eval,
                    pv: vec![mv],
                    was_unwinded: false,
                };
            }
            if eval > alpha {
                alpha = eval;
            }
        }

        SearchResultInternal {
            best_move: None,
            eval: alpha,
            pv: vec![],
            was_unwinded: false,
        }
    }

    // Check if stop_flag was set or time is over
    fn should_stop(&self, nodes: &mut u64, ctx: &SearchContext) -> bool {
        // Check every 1024 nodes, because it is time-expensive
        if *nodes % 1024 == 0 {
            if ctx.stop_flag.load(Ordering::Relaxed) {
                return true;
            }
            if let Some(limit) = ctx.time_limit {
                if ctx.start_time.elapsed() >= limit {
                    return true;
                }
            }
        }
        false
    }

    /// Wrapper that helps set initial parameters for minimax recursion
    pub fn negamax_wrapper(
        &mut self,
        game: &mut Game,
        depth: usize,
        ctx: &SearchContext,
    ) -> SearchResult {
        let mut nodes = 0;
        let mut result = self.negamax(game, depth, -EVAL_INF, EVAL_INF, &mut nodes, ctx);
        result.pv.reverse();

        SearchResult {
            best_move: result.best_move,
            eval: result.eval,
            pv: result.pv,
            was_unwinded: result.was_unwinded,
            nodes,
        }
    }
}