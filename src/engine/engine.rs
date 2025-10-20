use crate::{engine::{
    board::game::Game,
    search::searcher::Searcher,
}};

#[derive(Clone, Default)]
pub struct Engine {
    pub game: Game,
    pub searcher: Searcher,
}
