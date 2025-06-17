use crate::position::*;
use crate::utility::*;
use crate::constants::*;

pub fn pseudo_moves(pos: Position) -> Vec<Move> {
    let mut moves = Vec::new();
    generate_pseudo_pawn_moves  (&pos, &mut moves);
    generate_pseudo_moves_for_piece(&pos, Piece::Knight, &mut moves);
    generate_pseudo_moves_for_piece(&pos, Piece::King, &mut moves);
    generate_pseudo_moves_for_piece(&pos, Piece::Rook, &mut moves);
    generate_pseudo_moves_for_piece(&pos, Piece::Queen, &mut moves);
    generate_pseudo_moves_for_piece(&pos, Piece::King, &mut moves);
    moves
}

fn add_pawn_moves(moves: &mut Vec<Move>, to_mask: u64, offset: i8, promotion: bool, en_passant: bool) {
    let mut bb = to_mask;
    while bb != 0 {
        let to = pop_lsb(&mut bb);
        let from = (to as i8 - offset) as u8;
        // TODO: if inside a loop? too slow?
        if promotion {
            for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                moves.push(Move::pawn(from, to, Some(promo), en_passant));
            }
        } else if en_passant {
            moves.push(Move::pawn(from, to, None, true));
        } else {
            moves.push(Move::pawn(from, to, None, false));
        }
    }
}

fn generate_pseudo_pawn_moves(pos: &Position, moves: &mut Vec<Move>) {
    let empty = !pos.occupied;
    let en_passant_bb = pos.en_passant_square.map(|sq| 1u64 << sq).unwrap_or(0);
    let (pawns, enemy, left_offset, forward_offset, right_offset,
            start_rank, promo_rank, mask_right, mask_left) =
        if pos.whites_turn {
            (
                pos.w.pawns,
                pos.b.all,
                7,
                8,
                9,
                RANK[2],
                RANK[8],
                !FILE_H,
                !FILE_A,
            )
        } else {
            (
                pos.b.pawns,
                pos.w.all,
                -7,
                -8,
                -9,
                RANK[7],
                RANK[1],
                !FILE_A,
                !FILE_H,
            )
        };

    let single = signed_shift(pawns, forward_offset) & empty;
    let double = signed_shift(signed_shift(pawns & start_rank, forward_offset) & empty, forward_offset) & empty;
    let left   = signed_shift(pawns & mask_left, left_offset) & enemy;
    let right  = signed_shift(pawns & mask_right, right_offset) & enemy;

    // Handle promotions separately
    let promo_push  = single & promo_rank;
    let promo_left  = left & promo_rank;
    let promo_right = right & promo_rank;

    let non_promo_push  = single & !promo_rank;
    let non_promo_left  = left & !promo_rank;
    let non_promo_right = right & !promo_rank;

    // En passant
    let ep_left = signed_shift(pawns & mask_left, left_offset) & en_passant_bb;
    let ep_right = signed_shift(pawns & mask_right, right_offset) & en_passant_bb;

    add_pawn_moves(moves, non_promo_push,  forward_offset,     false, false);
    add_pawn_moves(moves, double,          2 * forward_offset, false, false);
    add_pawn_moves(moves, non_promo_left,  left_offset,        false, false);
    add_pawn_moves(moves, non_promo_right, right_offset,       false, false);

    add_pawn_moves(moves, promo_push,  forward_offset, true, false);
    add_pawn_moves(moves, promo_left,  left_offset,    true, false);
    add_pawn_moves(moves, promo_right, right_offset,   true, false);

    add_pawn_moves(moves, ep_left,  left_offset,  false, true);
    add_pawn_moves(moves, ep_right, right_offset, false, true);
}

fn knight_attacks(_pos: &Position, sq: usize, friendly: u64) -> u64 {
    KNIGHT_ATTACKS[sq] & !friendly
}

fn bishop_attacks(pos: &Position, sq: usize, friendly: u64) -> u64 {
    let mask = BISHOP_MASKS[sq];
    let magic = BISHOP_MAGICS[sq];
    let shift = BISHOP_MAGICS_SHIFT[sq];
    let blockers = pos.occupied & mask;
    let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
    BISHOP_ATTACK_TABLES[sq][hash] & !friendly
}

fn rook_attacks(pos: &Position, sq: usize, friendly: u64) -> u64 {
    let mask = ROOK_MASKS[sq];
    let magic = ROOK_MAGICS[sq];
    let shift = ROOK_MAGICS_SHIFT[sq];
    let blockers = pos.occupied & mask;
    let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
    ROOK_ATTACK_TABLES[sq][hash] & !friendly
}

fn queen_attacks(pos: &Position, sq: usize, friendly: u64) -> u64 {
    let rook_attacks = rook_attacks(pos, sq, friendly);
    let bishop_attacks = bishop_attacks(pos, sq, friendly);
    rook_attacks | bishop_attacks
}

fn king_attacks(_pos: &Position, sq: usize, friendly: u64) -> u64 {
    KING_ATTACKS[sq] & !friendly
}

fn generate_pseudo_moves_for_piece(pos: &Position, piece_type: Piece, moves: &mut Vec<Move>) {
    let bb = if pos.whites_turn { &pos.w } else { &pos.b };
    let friendly = bb.all;

    type AttackFn = fn(&Position, usize, u64) -> u64;
    let (mut pieces, attack_fn): (u64, AttackFn) = match piece_type {
        Piece::Knight => (bb.knights, knight_attacks),
        Piece::Bishop => (bb.bishops, bishop_attacks),
        Piece::Rook   => (bb.rooks, rook_attacks),
        Piece::Queen  => (bb.queens, queen_attacks),
        Piece::King   => (bb.king, king_attacks),
        Piece::Pawn   => panic!("aaaghhh")
    };

    while pieces != 0 {
        let from = pop_lsb(&mut pieces) as usize;
        let mut attacks = attack_fn(pos, from, friendly);
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            moves.push(Move::new(from as u8, to, piece_type));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pseudo_pawn_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        generate_pseudo_pawn_moves(&pos, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 8,  to: 16, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 9,  to: 17, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 10, to: 18, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 11, to: 19, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 12, to: 20, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 13, to: 21, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 14, to: 22, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 15, to: 23, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 8,  to: 24, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 9,  to: 25, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 10, to: 26, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 11, to: 27, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 12, to: 28, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 13, to: 29, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 14, to: 30, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 15, to: 31, piece: Piece::Pawn, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_pawn_moves_endgame() {
        let pos = Position::from_fen("8/p1pp3p/BN6/3R4/1k2K3/8/1p3pp1/2Q1B3 b - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_pawn_moves(&pos, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 48, to: 41, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 50, to: 41, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 50, to: 42, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 50, to: 34, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 51, to: 43, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 55, to: 47, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 55, to: 39, piece: Piece::Pawn, promotion: None, en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false }
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_pawn_moves_en_passant() {
        let pos = Position::from_fen("8/6k1/8/1pP1Pp2/8/8/5K2/8 w - b6 0 1");
        let mut moves = Vec::new();
        generate_pseudo_pawn_moves(&pos, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 34, to: 42, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 34, to: 41, piece: Piece::Pawn, promotion: None, en_passant: true },
            Move { from: 36, to: 44, piece: Piece::Pawn, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_knight_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Knight, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 1, to: 16, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 1, to: 18, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 6, to: 21, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 6, to: 23, piece: Piece::Knight, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_knight_moves_endgame() {
        let pos = Position::from_fen("8/3nk3/1N3R2/3n2n1/3N4/8/3K1N2/1r6 w - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Knight, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 13, to:  3, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to:  7, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 19, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 23, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 28, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 30, piece: Piece::Knight, promotion: None, en_passant: false },

            Move { from: 27, to: 10, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 12, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 17, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 21, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 33, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 37, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 42, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 44, piece: Piece::Knight, promotion: None, en_passant: false },

            Move { from: 41, to: 24, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 26, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 35, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 51, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 56, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 58, piece: Piece::Knight, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_king_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::King, &mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_king_moves_endgame() {
        let pos = Position::from_fen("8/8/8/8/7P/6K1/1r6/k7 w - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::King, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 22, to: 13, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 14, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 15, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 21, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 23, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 29, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 30, piece: Piece::King, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_rook_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Rook, &mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_rook_moves_endgame() {
        let pos = Position::from_fen("8/3k4/8/R3p3/6P1/1P6/3K2R1/8 w - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Rook, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 32, to: 40, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 48, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 56, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 24, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 16, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 8,  piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 0,  piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 33, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 34, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 35, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 36, piece: Piece::Rook, promotion: None, en_passant: false },

            Move { from: 14, to: 6,  piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 22, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 15, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 13, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 12, piece: Piece::Rook, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_bishop_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Bishop, &mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_bishop_moves_endgame() {
        let pos = Position::from_fen("8/8/8/3b4/5P1b/1k6/3b3K/b7 b - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Bishop, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 0,  to: 9,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 27, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 36, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 45, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 54, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 63, piece: Piece::Bishop, promotion: None, en_passant: false },

            Move { from: 11, to: 2,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 4,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 25, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 32, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 20, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 29, piece: Piece::Bishop, promotion: None, en_passant: false },

            Move { from: 31, to: 22, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 13, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 4,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 38, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 45, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 52, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 59, piece: Piece::Bishop, promotion: None, en_passant: false },

            Move { from: 35, to: 26, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 44, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 53, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 62, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 42, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 49, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 56, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 28, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 21, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 14, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 7,  piece: Piece::Bishop, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_bishop_moves_blocking_friendly() {
        let pos = Position::from_fen("1k3K2/8/1P3P2/8/3B4/8/1P3P2/8 w - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Bishop, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 27, to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 20, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 34, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 36, piece: Piece::Bishop, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_bishop_moves_blocking_hostile() {
        let pos = Position::from_fen("1k3K2/8/1p3p2/8/3B4/8/1p3p2/8 w - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Bishop, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 27, to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 9,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 20, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 13, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 34, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 41, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 36, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 45, piece: Piece::Bishop, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_queen_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Queen, &mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_queen_moves_endgame() {
        let pos = Position::from_fen("8/k3b3/2r5/8/4Q1N1/8/2K5/8 w - - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_moves_for_piece(&pos, Piece::Queen, &mut moves);

        let expected: HashSet<Move> = [
            Move { from: 28, to: 4,  piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 7,  piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 12, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 14, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 19, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 20, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 21, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 24, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 25, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 26, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 27, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 29, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 35, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 36, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 37, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 42, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 44, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 46, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 52, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 55, piece: Piece::Queen, promotion: None, en_passant: false },
        ].into();

        println!("gen");
        for m in &moves {
            println!("{:?}", m);
        }
        println!("exp");
        for m in &expected {
            println!("{:?}", m);
        }

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }
}
