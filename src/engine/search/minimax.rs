use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant},
};
use crate::constants::{
    move_ordering::MOVE_ORDERING_HISTORY_CAP,
    evaluation::EVAL_INF,
    *
};
use crate::engine::{
    base::_move::Move,
    board::game::Game,
    search::{
        evaluate::*,
        transposition_table::*,
        searcher::Searcher,
    },
};

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

    fn negamax(
        &mut self,
        game: &mut Game,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        nodes: &mut u64,
        ctx: &SearchContext,
    ) -> SearchResultInternal {
        *nodes += 1;

        if self.should_stop(nodes, ctx) {
            return SearchResultInternal::unwinded();
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
                if eval < beta {
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

        SearchResultInternal {
            best_move,
            eval: best_eval,
            pv: best_pv,
            was_unwinded: false,
        }
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
        let result = self.negamax(game, depth, -EVAL_INF, EVAL_INF, &mut nodes, ctx);

        SearchResult {
            best_move: result.best_move,
            eval: result.eval,
            pv: result.pv,
            was_unwinded: result.was_unwinded,
            nodes,
        }
    }
}
