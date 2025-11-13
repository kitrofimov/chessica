/// Constants related to move ordering heuristics

pub const MOVE_ORDERING_HASH_MOVE:       i32 = 22_000;
pub const MOVE_ORDERING_WINNING_CAPTURE: i32 = 16_000;
pub const MOVE_ORDERING_PROMOTION:       i32 = 11_000;
pub const MOVE_ORDERING_KILLER_1:        i32 = 10_500;
pub const MOVE_ORDERING_KILLER_2:        i32 = 10_000;
pub const MOVE_ORDERING_LOSING_CAPTURE:  i32 = 5_000;
pub const MOVE_ORDERING_HISTORY_CAP:     i32 = 1_000;

pub const MVV_LVA_PROMOTION_BONUS: i32   = 100;
pub const KILLER_MOVES_PLY_DEPTH:  usize = 128;
