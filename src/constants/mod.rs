pub mod magics;
pub mod board;
pub mod attacks;
pub mod blockers;
pub mod zobrist;
pub mod move_ordering;
pub mod evaluation;

// Preallocation constants
pub const GAME_HISTORY_CAPACITY: usize = 256;
pub const MOVE_LIST_CAPACITY: usize = 256;
pub const TRANSPOSITION_TABLE_MB_SIZE: usize = 64;
pub const PATH_TO_EVAL_PARAMS: &str = "params.json";

// Search and SEE constants
pub const SEARCH_MAX_PLY_DEPTH: usize = 128;
pub const SEE_EXCHANGE_MAX_DEPTH: usize = 32;
pub const SEE_QUIESCENCE_SEARCH_LOWER_BOUND: i32 = 0;
