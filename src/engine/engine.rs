use crate::{engine::{
    board::game::Game,
    search::searcher::Searcher,
}};

/// Main structure containing the game state and searcher
#[derive(Clone)]
pub struct Engine {
    pub game: Game,
    pub searcher: Searcher,
}

impl Engine {
    pub fn new(path_to_eval_params: &str) -> Self {
        Engine {
            game: Game::default(),
            searcher: Searcher::new(path_to_eval_params),
        }
    }
}
