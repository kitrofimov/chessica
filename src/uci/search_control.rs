use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use std::thread::JoinHandle;
use std::time::Instant;

use crate::{
    engine::{
        game::Game,
        base::_move::Move,
    },
    uci::output::*,
};

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

pub fn iterative_deepening(
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
        let (m, eval, nodes, pv, unwind) = game.find_best_move(
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
        print_uci_info(depth, eval, nodes, pv, elapsed);

        if let Some(limit) = time_limit {
            if start.elapsed() >= limit {
                break;
            }
        }
    }

    last_move
}
