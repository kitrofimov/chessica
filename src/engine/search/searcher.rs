use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::JoinHandle,
};
use crate::constants::move_ordering::KILLER_MOVES_PLY_DEPTH;
use crate::engine::{
    base::_move::Move,
    search::transposition_table::*,
};

#[derive(Clone)]
pub struct Searcher {
    pub transposition_table: TranspositionTable,
    pub history: [[i32; 64]; 64],  // [from][to]
    pub killer_moves: [[Option<Move>; 2]; KILLER_MOVES_PLY_DEPTH],
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            transposition_table: TranspositionTable::default(),
            history: [[0; 64]; 64],
            killer_moves: [[None; 2]; KILLER_MOVES_PLY_DEPTH],
        }
    }
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
}
