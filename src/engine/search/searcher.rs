use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::JoinHandle,
};
use crate::engine::{
    search::{
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
    pub fn new() -> Self {
        Self::default()
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
