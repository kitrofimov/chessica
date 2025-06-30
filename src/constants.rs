pub mod magics;
pub mod board;
pub mod attacks;
pub mod masks;
pub mod zobrist;

pub const NAME: &str = "chess-engine";
pub const AUTHOR: &str = "Kirill Trofimov";

// Preallocation constants
pub const GAME_HISTORY_CAPACITY: usize = 256;
pub const MOVE_LIST_CAPACITY: usize = 256;

pub const CHECKMATE_EVAL: i32 = 2_000_000_000;
pub const DRAW_EVAL: i32 = 0;
