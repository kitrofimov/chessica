use crate::constants::zobrist::*;
use crate::core::{position::*, player::Player};
use crate::utility::square_idx_to_coordinates;

pub fn zobrist_hash(pos: &Position) -> u64 {
    let mut hash: u64 = 0;
    for sq_idx in 0..64 {
        let what = pos.what(sq_idx);

        if let Some((player, piece)) = what {
            let piece = piece.index();
            let color = player.index();
            hash ^= ZOBRIST_PIECE[piece][color][sq_idx as usize];
        }
    }

    if pos.castling.white_kingside {
        hash ^= ZOBRIST_CASTLING[0];
    }
    if pos.castling.white_queenside {
        hash ^= ZOBRIST_CASTLING[1];
    }
    if pos.castling.black_kingside {
        hash ^= ZOBRIST_CASTLING[2];
    }
    if pos.castling.black_queenside {
        hash ^= ZOBRIST_CASTLING[3];
    }

    if let Some(ep_sq_idx) = pos.en_passant_square {
        let (file, _) = square_idx_to_coordinates(ep_sq_idx);
        hash ^= ZOBRIST_EN_PASSANT[file as usize];
    }

    if pos.player_to_move == Player::Black {
        hash ^= ZOBRIST_SIDE_BLACK;
    }

    hash
}
