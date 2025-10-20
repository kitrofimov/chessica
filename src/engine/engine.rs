use crate::{engine::{
    game::Game,
    searcher::Searcher,
}};

#[derive(Clone, Default)]
pub struct Engine {
    pub game: Game,
    pub searcher: Searcher,
}
