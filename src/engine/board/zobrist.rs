use crate::constants::zobrist::*;
use crate::engine::{board::position::*, base::player::Player};
use crate::utility::sq_to_coord;

pub type ZobristHash = u64;

// Compute the Zobrist hash from scratch
// Is only used when initializing a Position
pub fn zobrist_hash(pos: &Position) -> u64 {
    let mut hash: u64 = 0;
    for sq_idx in 0..64 {
        let what = pos.piece_at(sq_idx);

        if let Some((player, piece)) = what {
            let piece = piece.index();
            let color = player.index();
            hash ^= ZOBRIST_PIECE[piece][color][sq_idx as usize];
        }
    }

    hash ^= ZOBRIST_CASTLING[pos.castling.encode() as usize];

    if let Some(ep_sq_idx) = pos.en_passant_square {
        let (file, _) = sq_to_coord(ep_sq_idx);
        hash ^= ZOBRIST_EN_PASSANT_FILE[file as usize];
    }

    if pos.player_to_move == Player::Black {
        hash ^= ZOBRIST_SIDE_BLACK;
    }

    hash
}
