use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};
use crate::engine::{
    base::_move::Move,
    board::game::Game,
    search::searcher::Searcher,
    search::minimax::SearchContext,
};
use crate::uci;

impl Searcher {
    pub fn search(
        &mut self,
        game: &mut Game,
        stop_flag: Arc<AtomicBool>,
        max_depth: Option<usize>,
        time_limit: Option<Duration>,
    ) -> Option<Move> {
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
            let result = self.negamax_wrapper(
                game, depth, &SearchContext {
                    stop_flag: stop_flag.clone(),
                    start_time: start,
                    time_limit,
                    last_pv_move,
                }
            );
            let elapsed = depth_start.elapsed();

            // Update the best move only if there was NO unwind (the depth was searched fully)
            if result.was_unwinded {
                break;
            }

            last_move = result.best_move;
            last_pv_move = Some(result.pv[0]);
            uci::print_uci_info(depth, result.eval, result.nodes, result.pv, elapsed);

            if let Some(limit) = time_limit {
                if start.elapsed() >= limit {
                    break;
                }
            }
        }

        last_move
    }
}
