use crate::{engine::{
    board::game::Game,
    search::searcher::Searcher,
}};

/// Main structure containing the game state and searcher
#[derive(Clone, Default)]
pub struct Engine {
    pub game: Game,
    pub searcher: Searcher,
}
