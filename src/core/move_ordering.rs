use crate::core::chess_move::Move;
use crate::constants::move_ordering::*;
use crate::core::position::Position;

pub fn order_moves(mut moves: Vec<Move>, position: &Position, killers: &[Option<Move>; 2]) -> Vec<Move> {
    moves.sort_by_key(|m| {
        // TODO: PV & TT/Hash move
        if m.is_capture() {  // MVV-LVA captures
            let mvv_lva = m.mvv_lva_score(position);
            if mvv_lva > 0 {
                return MOVE_ORDERING_WINNING_CAPTURE + mvv_lva
            } else {
                return MOVE_ORDERING_LOSING_CAPTURE + mvv_lva
            }
        } else if m.is_promotion() {  // Promotions
            MOVE_ORDERING_PROMOTION
        } else if Some(*m) == killers[0] {
            MOVE_ORDERING_KILLER_1
        } else if Some(*m) == killers[1] {
            MOVE_ORDERING_KILLER_2
        // TODO: History moves
        } else {
            0
        }
    });
    moves.reverse(); // highest score first
    return moves;
}
