use crate::constants::move_ordering::*;
use crate::engine::{
    base::_move::Move,
    board::game::Game,
    search::searcher::Searcher,
};

impl Searcher {
    pub fn order_moves(
        &self, game: &Game, moves: &mut Vec<Move>,
        depth: usize, last_pv_move: Option<Move>
    ) {
        let hash_move = self.transposition_table
            .probe(game.position.zobrist_hash)
            .and_then(|e| e.best_move);
        let killers = &self.search_state.killer_moves[depth];
        let history = &self.search_state.history;

        moves.sort_by_key(|m| {
            if Some(*m) == hash_move {
                MOVE_ORDERING_HASH_MOVE
            } else if Some(*m) == last_pv_move {
                MOVE_ORDERING_LAST_PV_MOVE
            } else if m.is_capture() {  // MVV-LVA captures
                let mvv_lva = m.mvv_lva_score(&game.position);
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
            } else {
                history[m.from as usize][m.to as usize]
            }
        });
        moves.reverse(); // Highest score first
    }
}
