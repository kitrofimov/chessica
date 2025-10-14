use crate::core::chess_move::Move;
use crate::constants::move_ordering::*;
use crate::core::position::Position;

pub fn order_moves(mut moves: Vec<Move>, position: &Position) -> Vec<Move> {
    moves.sort_by_key(|m| {
        // TODO: PV & TT/Hash move
        if m.is_capture() {  // MVV-LVA captures
            MOVE_ORDERING_CAPTURE + m.mvv_lva_score(position)
        } else if m.is_promotion() {  // Promotions
            MOVE_ORDERING_PROMOTION
        }
        // TODO: Killer moves & History moves
        else {
            0
        }
    });
    moves.reverse(); // highest score first
    return moves;
}
