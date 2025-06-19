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
    generate_pseudo_castling_moves(&pos, &mut moves);
    moves
}

fn generate_pseudo_castling_moves(pos: &Position, moves: &mut Vec<Move>) {
    let (mut kingside_mask, mut queenside_mask, king_bb) =
        match pos.player_to_move {
            Player::White => (
                bit(4) | bit(5) | bit(6),
                bit(2) | bit(3) | bit(4),
                bit(4)
            ),
            Player::Black => (
                bit(60) | bit(61) | bit(62),
                bit(58) | bit(59) | bit(60),
                bit(60)
            ),
        };

    let white_kingside = pos.castling.white_kingside && pos.player_to_move == Player::White;
    let white_queenside = pos.castling.white_queenside && pos.player_to_move == Player::White;
    let black_kingside = pos.castling.black_kingside && pos.player_to_move == Player::Black;
    let black_queenside = pos.castling.black_queenside && pos.player_to_move == Player::Black;

    let kingside_empty = pos.occupied & kingside_mask == king_bb;
    let queenside_empty = pos.occupied & queenside_mask == king_bb;

    let enemy = pos.player_to_move.opposite();

    let mut kingside_not_attacked = true;
    while kingside_mask != 0 {
        let sq = pop_lsb(&mut kingside_mask);
        kingside_not_attacked = kingside_not_attacked && !pos.is_square_attacked(sq.into(), enemy);
    };

    let mut queenside_not_attacked = true;
    while queenside_mask != 0 {
        let sq = pop_lsb(&mut queenside_mask);
        queenside_not_attacked = queenside_not_attacked && !pos.is_square_attacked(sq.into(), enemy);
    };

    if (white_kingside || black_kingside) && kingside_empty && kingside_not_attacked {
        moves.push(Move::castling(pos.player_to_move, CastlingSide::KingSide));
    }

    if (white_queenside || black_queenside) && queenside_empty && queenside_not_attacked {
        moves.push(Move::castling(pos.player_to_move, CastlingSide::QueenSide));
    }
}

fn add_pawn_moves(moves: &mut Vec<Move>, to_mask: u64, offset: i8, capture: bool, promotion: bool, en_passant: bool) {
    let mut bb = to_mask;
    while bb != 0 {
        let to = pop_lsb(&mut bb);
        let from = (to as i8 - offset) as u8;
        // TODO: if inside a loop? too slow?
        if promotion {
            for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                moves.push(Move::pawn(from, to, capture, Some(promo), en_passant));
            }
        } else if en_passant {
            moves.push(Move::pawn(from, to, capture, None, true));
        } else {
            moves.push(Move::pawn(from, to, capture, None, false));
        }
    }
}

fn generate_pseudo_pawn_moves(pos: &Position, moves: &mut Vec<Move>) {
    let empty = !pos.occupied;
    let en_passant_bb = pos.en_passant_square.map(|sq| 1u64 << sq).unwrap_or(0);
    let (pawns, enemy, left_offset, forward_offset, right_offset,
         start_rank, promo_rank, mask_right, mask_left) =
        match pos.player_to_move {
            Player::White => (
                pos.w.pawns, pos.b.all, 7, 8, 9,
                RANK[2], RANK[8], !FILE_H, !FILE_A,
            ),
            Player::Black => (
                pos.b.pawns, pos.w.all, -7, -8, -9,
                RANK[7], RANK[1], !FILE_A, !FILE_H,
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

    add_pawn_moves(moves, non_promo_push,  forward_offset,     false, false, false);
    add_pawn_moves(moves, double,          2 * forward_offset, false, false, false);
    add_pawn_moves(moves, non_promo_left,  left_offset,        true,  false, false);
    add_pawn_moves(moves, non_promo_right, right_offset,       true,  false, false);

    add_pawn_moves(moves, promo_push,  forward_offset, false, true, false);
    add_pawn_moves(moves, promo_left,  left_offset,    true,  true, false);
    add_pawn_moves(moves, promo_right, right_offset,   true,  true, false);

    add_pawn_moves(moves, ep_left,  left_offset,  true, false, true);
    add_pawn_moves(moves, ep_right, right_offset, true, false, true);
}

pub fn knight_attacks(_pos: &Position, sq: usize, friendly: u64) -> u64 {
    KNIGHT_ATTACKS[sq] & !friendly
}

pub fn bishop_attacks(pos: &Position, sq: usize, friendly: u64) -> u64 {
    let mask = BISHOP_MASKS[sq];
    let magic = BISHOP_MAGICS[sq];
    let shift = BISHOP_MAGICS_SHIFT[sq];
    let blockers = pos.occupied & mask;
    let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
    BISHOP_ATTACK_TABLES[sq][hash] & !friendly
}

pub fn rook_attacks(pos: &Position, sq: usize, friendly: u64) -> u64 {
    let mask = ROOK_MASKS[sq];
    let magic = ROOK_MAGICS[sq];
    let shift = ROOK_MAGICS_SHIFT[sq];
    let blockers = pos.occupied & mask;
    let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
    ROOK_ATTACK_TABLES[sq][hash] & !friendly
}

pub fn queen_attacks(pos: &Position, sq: usize, friendly: u64) -> u64 {
    let rook_attacks = rook_attacks(pos, sq, friendly);
    let bishop_attacks = bishop_attacks(pos, sq, friendly);
    rook_attacks | bishop_attacks
}

pub fn king_attacks(_pos: &Position, sq: usize, friendly: u64) -> u64 {
    KING_ATTACKS[sq] & !friendly
}

fn generate_pseudo_moves_for_piece(pos: &Position, piece_type: Piece, moves: &mut Vec<Move>) {
    let (my_set, enemy_set) = match pos.player_to_move {
        Player::White => (&pos.w, &pos.b),
        Player::Black => (&pos.b, &pos.w),
    };
    let friendly = my_set.all;
    let hostile = enemy_set.all;

    type AttackFn = fn(&Position, usize, u64) -> u64;
    let (mut pieces, attack_fn): (u64, AttackFn) = match piece_type {
        Piece::Knight => (my_set.knights, knight_attacks),
        Piece::Bishop => (my_set.bishops, bishop_attacks),
        Piece::Rook   => (my_set.rooks, rook_attacks),
        Piece::Queen  => (my_set.queens, queen_attacks),
        Piece::King   => (my_set.king, king_attacks),
        Piece::Pawn   => panic!("aaaghhh")
    };

    while pieces != 0 {
        let from = pop_lsb(&mut pieces) as usize;
        let mut attacks = attack_fn(pos, from, friendly);
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            let capture = (bit(to.into()) & hostile) > 0;
            moves.push(Move::new(from as u8, to, piece_type, capture));
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
            Move::pawn(8, 16, false, None, false),
            Move::pawn(9, 17, false, None, false),
            Move::pawn(10, 18, false, None, false),
            Move::pawn(11, 19, false, None, false),
            Move::pawn(12, 20, false, None, false),
            Move::pawn(13, 21, false, None, false),
            Move::pawn(14, 22, false, None, false),
            Move::pawn(15, 23, false, None, false),
            Move::pawn(8, 24, false, None, false),
            Move::pawn(9, 25, false, None, false),
            Move::pawn(10, 26, false, None, false),
            Move::pawn(11, 27, false, None, false),
            Move::pawn(12, 28, false, None, false),
            Move::pawn(13, 29, false, None, false),
            Move::pawn(14, 30, false, None, false),
            Move::pawn(15, 31, false, None, false),
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
            Move::pawn(48, 41, true, None, false),
            Move::pawn(50, 41, true, None, false),
            Move::pawn(50, 42, false, None, false),
            Move::pawn(50, 34, false, None, false),
            Move::pawn(51, 43, false, None, false),
            Move::pawn(55, 47, false, None, false),
            Move::pawn(55, 39, false, None, false),

            Move::pawn(9, 1, false, Some(Piece::Knight), false),
            Move::pawn(9, 2, true, Some(Piece::Knight), false),
            Move::pawn(13, 5, false, Some(Piece::Knight), false),
            Move::pawn(13, 4, true, Some(Piece::Knight), false),
            Move::pawn(14, 6, false, Some(Piece::Knight), false),

            Move::pawn(9, 1, false, Some(Piece::Bishop), false),
            Move::pawn(9, 2, true, Some(Piece::Bishop), false),
            Move::pawn(13, 5, false, Some(Piece::Bishop), false),
            Move::pawn(13, 4, true, Some(Piece::Bishop), false),
            Move::pawn(14, 6, false, Some(Piece::Bishop), false),

            Move::pawn(9, 1, false, Some(Piece::Rook), false),
            Move::pawn(9, 2, true, Some(Piece::Rook), false),
            Move::pawn(13, 5, false, Some(Piece::Rook), false),
            Move::pawn(13, 4, true, Some(Piece::Rook), false),
            Move::pawn(14, 6, false, Some(Piece::Rook), false),

            Move::pawn(9, 1, false, Some(Piece::Queen), false),
            Move::pawn(9, 2, true, Some(Piece::Queen), false),
            Move::pawn(13, 5, false, Some(Piece::Queen), false),
            Move::pawn(13, 4, true, Some(Piece::Queen), false),
            Move::pawn(14, 6, false, Some(Piece::Queen), false),
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
            Move::pawn(34, 42, false, None, false),
            Move::pawn(34, 41, true, None, true),
            Move::pawn(36, 44, false, None, false),
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
            Move::new(1, 16, Piece::Knight, false),
            Move::new(1, 18, Piece::Knight, false),
            Move::new(6, 21, Piece::Knight, false),
            Move::new(6, 23, Piece::Knight, false),
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
            Move::new(13, 3, Piece::Knight, false),
            Move::new(13, 7, Piece::Knight, false),
            Move::new(13, 19, Piece::Knight, false),
            Move::new(13, 23, Piece::Knight, false),
            Move::new(13, 28, Piece::Knight, false),
            Move::new(13, 30, Piece::Knight, false),
            Move::new(27, 10, Piece::Knight, false),
            Move::new(27, 12, Piece::Knight, false),
            Move::new(27, 17, Piece::Knight, false),
            Move::new(27, 21, Piece::Knight, false),
            Move::new(27, 33, Piece::Knight, false),
            Move::new(27, 37, Piece::Knight, false),
            Move::new(27, 42, Piece::Knight, false),
            Move::new(27, 44, Piece::Knight, false),
            Move::new(41, 24, Piece::Knight, false),
            Move::new(41, 26, Piece::Knight, false),
            Move::new(41, 35, Piece::Knight, true),
            Move::new(41, 51, Piece::Knight, true),
            Move::new(41, 56, Piece::Knight, false),
            Move::new(41, 58, Piece::Knight, false),
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
            Move::new(22, 13, Piece::King, false),
            Move::new(22, 14, Piece::King, false),
            Move::new(22, 15, Piece::King, false),
            Move::new(22, 21, Piece::King, false),
            Move::new(22, 23, Piece::King, false),
            Move::new(22, 29, Piece::King, false),
            Move::new(22, 30, Piece::King, false),
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
            Move::new(32, 40, Piece::Rook, false),
            Move::new(32, 48, Piece::Rook, false),
            Move::new(32, 56, Piece::Rook, false),
            Move::new(32, 24, Piece::Rook, false),
            Move::new(32, 16, Piece::Rook, false),
            Move::new(32, 8, Piece::Rook, false),
            Move::new(32, 0, Piece::Rook, false),
            Move::new(32, 33, Piece::Rook, false),
            Move::new(32, 34, Piece::Rook, false),
            Move::new(32, 35, Piece::Rook, false),
            Move::new(32, 36, Piece::Rook, true),
            Move::new(14, 6, Piece::Rook, false),
            Move::new(14, 22, Piece::Rook, false),
            Move::new(14, 15, Piece::Rook, false),
            Move::new(14, 13, Piece::Rook, false),
            Move::new(14, 12, Piece::Rook, false),
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
            Move::new(0, 9, Piece::Bishop, false),
            Move::new(0, 18, Piece::Bishop, false),
            Move::new(0, 27, Piece::Bishop, false),
            Move::new(0, 36, Piece::Bishop, false),
            Move::new(0, 45, Piece::Bishop, false),
            Move::new(0, 54, Piece::Bishop, false),
            Move::new(0, 63, Piece::Bishop, false),
            Move::new(11, 2, Piece::Bishop, false),
            Move::new(11, 4, Piece::Bishop, false),
            Move::new(11, 18, Piece::Bishop, false),
            Move::new(11, 25, Piece::Bishop, false),
            Move::new(11, 32, Piece::Bishop, false),
            Move::new(11, 20, Piece::Bishop, false),
            Move::new(11, 29, Piece::Bishop, true),
            Move::new(31, 22, Piece::Bishop, false),
            Move::new(31, 13, Piece::Bishop, false),
            Move::new(31, 4, Piece::Bishop, false),
            Move::new(31, 38, Piece::Bishop, false),
            Move::new(31, 45, Piece::Bishop, false),
            Move::new(31, 52, Piece::Bishop, false),
            Move::new(31, 59, Piece::Bishop, false),
            Move::new(35, 26, Piece::Bishop, false),
            Move::new(35, 44, Piece::Bishop, false),
            Move::new(35, 53, Piece::Bishop, false),
            Move::new(35, 62, Piece::Bishop, false),
            Move::new(35, 42, Piece::Bishop, false),
            Move::new(35, 49, Piece::Bishop, false),
            Move::new(35, 56, Piece::Bishop, false),
            Move::new(35, 28, Piece::Bishop, false),
            Move::new(35, 21, Piece::Bishop, false),
            Move::new(35, 14, Piece::Bishop, false),
            Move::new(35, 7, Piece::Bishop, false),
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
            Move::new(27, 18, Piece::Bishop, false),
            Move::new(27, 20, Piece::Bishop, false),
            Move::new(27, 34, Piece::Bishop, false),
            Move::new(27, 36, Piece::Bishop, false),
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
            Move::new(27, 18, Piece::Bishop, false),
            Move::new(27, 9, Piece::Bishop, true),
            Move::new(27, 20, Piece::Bishop, false),
            Move::new(27, 13, Piece::Bishop, true),
            Move::new(27, 34, Piece::Bishop, false),
            Move::new(27, 41, Piece::Bishop, true),
            Move::new(27, 36, Piece::Bishop, false),
            Move::new(27, 45, Piece::Bishop, true),
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
            Move::new(28, 4, Piece::Queen, false),
            Move::new(28, 7, Piece::Queen, false),
            Move::new(28, 12, Piece::Queen, false),
            Move::new(28, 14, Piece::Queen, false),
            Move::new(28, 19, Piece::Queen, false),
            Move::new(28, 20, Piece::Queen, false),
            Move::new(28, 21, Piece::Queen, false),
            Move::new(28, 24, Piece::Queen, false),
            Move::new(28, 25, Piece::Queen, false),
            Move::new(28, 26, Piece::Queen, false),
            Move::new(28, 27, Piece::Queen, false),
            Move::new(28, 29, Piece::Queen, false),
            Move::new(28, 35, Piece::Queen, false),
            Move::new(28, 36, Piece::Queen, false),
            Move::new(28, 37, Piece::Queen, false),
            Move::new(28, 42, Piece::Queen, true),
            Move::new(28, 44, Piece::Queen, false),
            Move::new(28, 46, Piece::Queen, false),
            Move::new(28, 52, Piece::Queen, true),
            Move::new(28, 55, Piece::Queen, false),
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_castling_moves_midgame1() {
        let pos = Position::from_fen("rnb1k1nr/pppp1ppp/3b1q2/4p3/2BPP3/2P2N2/PP3PPP/RNBQK2R w KQkq - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_castling_moves(&pos, &mut moves);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], Move::castling(Player::White, CastlingSide::KingSide));
    }

    #[test]
    fn pseudo_castling_moves_midgame2() {
        let pos = Position::from_fen("r3kbnr/ppp2ppp/2np2b1/4p2q/4P3/5PP1/PPPP3P/RNBQKBNR b KQkq - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_castling_moves(&pos, &mut moves);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], Move::castling(Player::Black, CastlingSide::QueenSide));
    }

    #[test]
    fn pseudo_castling_moves_should_generate_nothing() {
        let pos = Position::from_fen("r3kbnr/ppp2ppp/2np2b1/4p2q/4P3/3P1PPB/PPP4P/RNBQK1NR b KQkq - 0 1");
        let mut moves = Vec::new();
        generate_pseudo_castling_moves(&pos, &mut moves);
        assert_eq!(moves.len(), 0);
    }
}
