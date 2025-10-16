use crate::constants::move_ordering::KILLER_MOVES_PLY_DEPTH;
use crate::engine::base::_move::Move;

#[derive(Clone, Debug)]
pub struct SearchState {
    pub history: [[i32; 64]; 64],  // [from][to]
    pub killer_moves: [[Option<Move>; 2]; KILLER_MOVES_PLY_DEPTH],
}

impl Default for SearchState {
    fn default() -> Self {
        SearchState {
            history: [[0; 64]; 64],
            killer_moves: [[None; 2]; KILLER_MOVES_PLY_DEPTH],
        }
    }
}
