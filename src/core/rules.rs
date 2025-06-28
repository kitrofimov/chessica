use crate::constants::{attacks, board, zobrist::*};
use crate::utility::{pop_lsb, square_idx_to_coordinates};
use crate::core::{
    position::*,
    bitboard::*,
    chess_move::*,
    movegen::*,
    player::Player,
    piece::Piece,
};

pub fn make_move(pos: &Position, m: &Move) -> Position {
    let mut new = pos.clone();

    let who_made_move = pos.player_to_move;
    let old_castling = new.castling;

    update_en_passant_square(&mut new, m);

    if m.is_castling() {
        handle_castling(&mut new, m, who_made_move);
    } else {
        handle_standard_move(&mut new, m, who_made_move);
    }

    update_castling_hash(&mut new, old_castling);
    finalize_move(&mut new);
    new
}

fn update_en_passant_square(new: &mut Position, m: &Move) {
    if let Some(prev_ep_sq) = new.en_passant_square {
        en_passant_hash(&mut new.zobrist_hash, prev_ep_sq);
    }

    new.en_passant_square = if m.double_push {
        let new_ep_sq = (m.from + m.to) / 2;
        en_passant_hash(&mut new.zobrist_hash, new_ep_sq);
        Some(new_ep_sq)
    } else {
        None
    };
}

fn handle_castling(new: &mut Position, m: &Move, who_made_move: Player) {
    let (rook_from, rook_to) = match (who_made_move, m.kingside_castling, m.queenside_castling) {
        (Player::White, true, _) => (board::H1, board::F1),
        (Player::White, _, true) => (board::A1, board::D1),
        (Player::Black, true, _) => (board::H8, board::F8),
        (Player::Black, _, true) => (board::A8, board::D8),
        _ => unreachable!(),
    };
    let friendly = match who_made_move {
        Player::White => &mut new.w,
        Player::Black => &mut new.b,
    };

    friendly.king = friendly.king.unset_bit(m.from).set_bit(m.to);
    friendly.rooks = friendly.rooks.unset_bit(rook_from).set_bit(rook_to);

    let rook_move = Move::new(rook_from, rook_to, Piece::Rook, false);

    apply_move_hash(&mut new.zobrist_hash, m, who_made_move);
    apply_move_hash(&mut new.zobrist_hash, &rook_move, who_made_move);

    new.castling.reset(who_made_move);
}

fn toggle_piece_hash(hash: &mut u64, piece: Piece, player: Player, sq: u8) {
    *hash ^= ZOBRIST_PIECE[piece.index()][player.index()][sq as usize];
}

fn apply_move_hash(hash: &mut u64, m: &Move, player: Player) {
    toggle_piece_hash(hash, m.piece, player, m.from);
    toggle_piece_hash(hash, m.piece, player, m.to);
}

fn en_passant_hash(hash: &mut u64, ep_sq: u8) {
    let (file, _) = square_idx_to_coordinates(ep_sq);
    *hash ^= ZOBRIST_EN_PASSANT_FILE[file as usize];
}

fn handle_standard_move(new: &mut Position, m: &Move, who_made_move: Player) {
    update_castling_rights(&mut new.castling, m, who_made_move);

    if m.piece == Piece::Pawn {
        new.halfmove_clock = 0;
    }

    // Borrow checker workaround
    let mut castling = new.castling;
    let mut hash = new.zobrist_hash;

    let (friendly, hostile) = new.perspective_mut(who_made_move);

    if let Some(promotion_piece) = m.promotion {
        handle_promotion(friendly, m, &mut hash, who_made_move, promotion_piece);
    } else {
        handle_normal_move(friendly, m, &mut hash, who_made_move);
    }

    if m.en_passant {
        handle_en_passant(hostile, m, &mut hash, who_made_move);
    } else if m.capture {
        handle_capture(hostile, m, &mut hash, &mut castling, who_made_move);
    }

    new.castling = castling;
    new.zobrist_hash = hash;
}

fn handle_promotion(
    friendly: &mut BitboardSet,
    m: &Move,
    hash: &mut u64,
    who_made_move: Player,
    promotion_piece: Piece
) {
    friendly.pawns = friendly.pawns.unset_bit(m.from);
    let bb = friendly.piece_to_bb_mut(promotion_piece);
    *bb = bb.set_bit(m.to);
    toggle_piece_hash(hash, Piece::Pawn, who_made_move, m.from);
    toggle_piece_hash(hash, promotion_piece, who_made_move, m.to);
}

fn handle_normal_move(
    friendly: &mut BitboardSet,
    m: &Move,
    hash: &mut u64,
    who_made_move: Player
) {
    let bb = friendly.piece_to_bb_mut(m.piece);
    *bb = bb.unset_bit(m.from).set_bit(m.to);
    toggle_piece_hash(hash, m.piece, who_made_move, m.from);
    toggle_piece_hash(hash, m.piece, who_made_move, m.to);
}

fn handle_en_passant(
    hostile: &mut BitboardSet,
    m: &Move,
    hash: &mut u64,
    who_made_move: Player
) {
    let captured_pawn_sq = match who_made_move {
        Player::White => m.to - 8,
        Player::Black => m.to + 8,
    };
    hostile.pawns = hostile.pawns.unset_bit(captured_pawn_sq);
    toggle_piece_hash(hash, Piece::Pawn, who_made_move.opposite(), captured_pawn_sq);
}

fn handle_capture(
    hostile: &mut BitboardSet,
    m: &Move,
    hash: &mut u64,
    castling: &mut CastlingRights,
    who_made_move: Player
) {
    let captured_piece = hostile.what(m.to)
        .expect("tried to capture an empty square");

    hostile.unset_bit(m.to);
    toggle_piece_hash(hash, captured_piece, who_made_move.opposite(), m.to);

    // Update castling rights
    match m.to {
        board::A1 => castling.reset_side(Player::White, CastlingSide::QueenSide),
        board::H1 => castling.reset_side(Player::White, CastlingSide::KingSide),
        board::A8 => castling.reset_side(Player::Black, CastlingSide::QueenSide),
        board::H8 => castling.reset_side(Player::Black, CastlingSide::KingSide),
        _ => {}
    }
}

fn update_castling_rights(castling: &mut CastlingRights, m: &Move, who_made_move: Player) {
    match m.piece {
        Piece::King => castling.reset(who_made_move),
        Piece::Rook if castling.any(who_made_move) => {
            match m.from {
                board::A1 | board::A8 => castling.reset_side(who_made_move, CastlingSide::QueenSide),
                board::H1 | board::H8 => castling.reset_side(who_made_move, CastlingSide::KingSide),
                _ => {}
            }
        }
        _ => {}
    }
}

fn update_castling_hash(new: &mut Position, old_castling: CastlingRights) {
    new.zobrist_hash ^= ZOBRIST_CASTLING[old_castling.encode() as usize];
    new.zobrist_hash ^= ZOBRIST_CASTLING[new.castling.encode() as usize];
}

fn finalize_move(new: &mut Position) {
    new.update();
    new.player_to_move = new.player_to_move.opposite();
    new.zobrist_hash ^= ZOBRIST_SIDE_BLACK;
    new.halfmove_clock += 1;
}

pub fn is_square_attacked(pos: &Position, sq: usize, by_player: Player) -> bool {
    let friend = match by_player {
        Player::White => &pos.w,
        Player::Black => &pos.b,
    };

    // All the possible pieces' positions, which could attack this square
    // reversing intentionally, questioning: "what could have attacked this square?"
    let pawn = match by_player {
        Player::White => attacks::PAWN_ATTACKS_BLACK[sq],
        Player::Black => attacks::PAWN_ATTACKS_WHITE[sq],
    };
    let knight = knight_attacks(pos, sq, 0x0);
    let bishop = bishop_attacks(pos, sq, 0x0);
    let rook   = rook_attacks  (pos, sq, 0x0);
    let queen  = queen_attacks (pos, sq, 0x0);
    let king   = king_attacks  (pos, sq, 0x0);

    pawn   & friend.pawns   > 0 || knight & friend.knights > 0 ||
    bishop & friend.bishops > 0 || rook   & friend.rooks   > 0 ||
    queen  & friend.queens  > 0 || king   & friend.king    > 0
}

pub fn is_king_in_check(pos: &Position, player: Player) -> bool {
    let mut king_bb = match player {
        Player::White => pos.w.king,
        Player::Black => pos.b.king,
    };
    let sq = pop_lsb(&mut king_bb) as usize;
    is_square_attacked(pos, sq, player.opposite())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::bit;
    use crate::core::piece::Piece;

    #[test]
    fn make_move_knight() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/1k6/3r4/8/4N3/8/1K6/8 w - - 0 1")?;
        let m = Move::new(28, 43, Piece::Knight, true);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.king, bit(9));
        assert_eq!(new.w.knights, bit(43));
        assert_eq!(new.w.all, bit(9) | bit(43));

        assert_eq!(new.b.king, bit(49));
        assert_eq!(new.b.rooks, 0x0);
        assert_eq!(new.b.all, bit(49));

        assert_eq!(new.occupied, bit(9) | bit(43) | bit(49));
        Ok(())
    }

    #[test]
    fn make_move_rook() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/8/8/5r2/8/1k6/5Q2/1K6 b - - 0 1")?;
        let m = Move::new(37, 13, Piece::Rook, true);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.king, bit(1));
        assert_eq!(new.w.queens, 0x0);
        assert_eq!(new.w.all, bit(1));

        assert_eq!(new.b.king, bit(17));
        assert_eq!(new.b.rooks, bit(13));
        assert_eq!(new.b.all, bit(13) | bit(17));
        Ok(())
    }

    #[test]
    fn make_move_king() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/5kq1/1R6/8/3K4/8/8/8 w - - 0 1")?;
        let m = Move::new(27, 35, Piece::King, false);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.rooks, bit(41));
        assert_eq!(new.w.king, bit(35));
        assert_eq!(new.w.all, bit(35) | bit(41));

        assert_eq!(new.b.king, bit(53));
        assert_eq!(new.b.queens, bit(54));
        assert_eq!(new.b.all, bit(53) | bit(54));
        Ok(())
    }

    #[test]
    fn make_move_bishop() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/2k5/8/4K3/1r6/8/3B4/8 w - - 0 1")?;
        let m = Move::new(11, 25, Piece::Bishop, true);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.king, bit(36));
        assert_eq!(new.w.bishops, bit(25));
        assert_eq!(new.w.all, bit(25) | bit(36));

        assert_eq!(new.b.king, bit(50));
        assert_eq!(new.b.rooks, 0x0);
        assert_eq!(new.b.all, bit(50));
        Ok(())
    }

    #[test]
    fn make_move_queen() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/8/1kq5/8/5K2/2R5/8/8 b - - 0 1")?;
        let m = Move::new(42, 18, Piece::Queen, true);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.king, bit(29));
        assert_eq!(new.w.rooks, 0x0);
        assert_eq!(new.w.all, bit(29));

        assert_eq!(new.b.king, bit(41));
        assert_eq!(new.b.queens, bit(18));
        assert_eq!(new.b.all, bit(18) | bit(41));
        Ok(())
    }

    #[test]
    fn make_move_white_kingside_castling() -> Result<(), FenParseError> {
        let pos = Position::from_fen("rn1qkbnr/ppp2ppp/3p4/4p3/2B1P1b1/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4")?;
        let m = Move::castling(Player::White, CastlingSide::KingSide);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.all, pos.w.all & !(bit(4) | bit(7)) | bit(5) | bit(6));
        assert_eq!(new.occupied, pos.occupied & !(bit(4) | bit(7)) | bit(5) | bit(6));
        assert_eq!(new.b, pos.b);
        assert_eq!(new.w.king, bit(6));
        assert_eq!(new.w.rooks, bit(0) | bit(5));
        Ok(())
    }

    #[test]
    fn make_move_black_kingside_castling() -> Result<(), FenParseError> {
        let pos = Position::from_fen("rnbqk2r/pppp1ppp/5n2/2b1p3/4P3/3PBN2/PPP2PPP/RN1QKB1R b KQkq - 4 4")?;
        let m = Move::castling(Player::Black, CastlingSide::KingSide);
        let new = make_move(&pos, &m);
        assert_eq!(new.b.all, pos.b.all & !(bit(60) | bit(63)) | bit(61) | bit(62));
        assert_eq!(new.occupied, pos.occupied & !(bit(60) | bit(63)) | bit(61) | bit(62));
        assert_eq!(new.w, pos.w);
        assert_eq!(new.b.king, bit(62));
        assert_eq!(new.b.rooks, bit(56) | bit(61));
        Ok(())
    }

    #[test]
    fn make_move_white_queenside_castling() -> Result<(), FenParseError> {
        let pos = Position::from_fen("rn2k1nr/ppp2ppp/3pbq2/2b1p2Q/4P3/2NPB3/PPP2PPP/R3KBNR w KQkq - 4 6")?;
        let m = Move::castling(Player::White, CastlingSide::QueenSide);
        let new = make_move(&pos, &m);
        assert_eq!(new.w.all, pos.w.all & !(bit(0) | bit(4)) | bit(2) | bit(3));
        assert_eq!(new.occupied, pos.occupied & !(bit(0) | bit(4)) | bit(2) | bit(3));
        assert_eq!(new.b, pos.b);
        assert_eq!(new.w.king, bit(2));
        assert_eq!(new.w.rooks, bit(3) | bit(7));
        Ok(())
    }

    #[test]
    fn make_move_black_queenside_castling() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r3kbnr/ppp2ppp/2npbq2/4p1N1/4P3/2NPB3/PPP2PPP/R2QKB1R b KQkq - 7 6")?;
        let m = Move::castling(Player::Black, CastlingSide::QueenSide);
        let new = make_move(&pos, &m);
        assert_eq!(new.b.all, pos.b.all & !(bit(56) | bit(60)) | bit(58) | bit(59));
        assert_eq!(new.occupied, pos.occupied & !(bit(56) | bit(60)) | bit(58) | bit(59));
        assert_eq!(new.w, pos.w);
        assert_eq!(new.b.king, bit(58));
        assert_eq!(new.b.rooks, bit(59) | bit(63));
        Ok(())
    }

    #[test]
    fn is_square_attacked_endgame() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/3r1k2/8/4N3/1Q5q/8/2K5/8 b - - 0 1")?;
        assert_eq!(is_square_attacked(&pos, 53, Player::White), true);
        assert_eq!(is_square_attacked(&pos, 51, Player::White), true);
        assert_eq!(is_square_attacked(&pos, 20, Player::White), false);
        assert_eq!(is_square_attacked(&pos, 25, Player::Black), true);
        assert_eq!(is_square_attacked(&pos, 52, Player::Black), true);
        assert_eq!(is_square_attacked(&pos, 10, Player::Black), false);
        Ok(())
    }

    #[test]
    fn is_king_in_check_midgame_1() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r1bqkb1r/ppp2ppp/5n2/1B4Q1/1n1P2N1/2N5/PPP2PPP/R1B1K2R b KQkq - 0 1")?;
        assert_eq!(is_king_in_check(&pos, Player::White), false);
        assert_eq!(is_king_in_check(&pos, Player::Black), true);
        Ok(())
    }

    #[test]
    fn is_king_in_check_midgame_2() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r1bqk1nr/pppp2pp/2n5/1B2pp2/1b1PP3/5N2/PPP2PPP/RNBQK2R w KQkq - 0 1")?;
        assert_eq!(is_king_in_check(&pos, Player::White), true);
        assert_eq!(is_king_in_check(&pos, Player::Black), false);
        Ok(())
    }

    #[test]
    fn is_king_in_check_endgame() -> Result<(), FenParseError> {
        let pos = Position::from_fen("R6k/8/7K/8/8/1b6/8/8 b - - 0 1")?;
        assert_eq!(is_king_in_check(&pos, Player::White), false);
        assert_eq!(is_king_in_check(&pos, Player::Black), true);
        Ok(())
    }

    #[test]
    fn zobrist_hash_piece_movement() -> Result<(), FenParseError> {
        let pos = Position::start();
        let new = make_move(&pos, &Move::pawn(board::E2, board::E3, false, None, false));
        let after = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_piece_movement_en_passant() -> Result<(), FenParseError> {
        let pos = Position::start();
        let new = make_move(&pos, &Move::pawn(board::E2, board::E4, false, None, false));
        let after = Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_piece_movement_en_passant_update() -> Result<(), FenParseError> {
        let pos = Position::start();
        let e4 = make_move(&pos, &Move::pawn(board::E2, board::E4, false, None, false));
        let d5 = make_move(&e4,  &Move::pawn(board::D7, board::D5, false, None, false));
        let x = e4.zobrist_hash ^ ZOBRIST_PIECE[Piece::Pawn.index()][Player::White.index()][board::E4 as usize]
                                ^ ZOBRIST_PIECE[Piece::Pawn.index()][Player::Black.index()][board::D7 as usize]
                                ^ ZOBRIST_EN_PASSANT_FILE[4];
        let y = d5.zobrist_hash ^ ZOBRIST_PIECE[Piece::Pawn.index()][Player::White.index()][board::E4 as usize]
                                ^ ZOBRIST_PIECE[Piece::Pawn.index()][Player::Black.index()][board::D5 as usize]
                                ^ ZOBRIST_SIDE_BLACK
                                ^ ZOBRIST_EN_PASSANT_FILE[3];
        assert_eq!(x, y);
        Ok(())
    }

    #[test]
    fn zobrist_hash_piece_capture() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/1k6/4r3/1K1P4/8/8/8/8 w - - 0 1")?;
        let new = make_move(&pos, &Move::pawn(board::D5, board::E6, true, None, false));
        let after = Position::from_fen("8/1k6/4P3/1K6/8/8/8/8 b - - 0 1")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_piece_capture_en_passant() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/6k1/1p6/2pP4/8/8/2P3K1/8 w - c6 0 1")?;
        let new = make_move(&pos, &Move::pawn(board::D5, board::C6, true, None, true));
        let after = Position::from_fen("8/6k1/1pP5/8/8/8/2P3K1/8 b - - 0 1")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_pawn_promotion() -> Result<(), FenParseError> {
        let pos = Position::from_fen("8/2P5/8/8/8/1r6/4k1K1/8 w - - 0 1")?;
        let new = make_move(&pos, &Move::pawn(board::C7, board::C8, false, Some(Piece::Queen), false));
        let after = Position::from_fen("2Q5/8/8/8/8/1r6/4k1K1/8 b - - 0 1")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_castling() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r1b1kbnr/pppp1ppp/2n2q2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")?;
        let new = make_move(&pos, &Move::castling(Player::White, CastlingSide::KingSide));
        let after = Position::from_fen("r1b1kbnr/pppp1ppp/2n2q2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 5 4")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_castling_revoked_rook_move() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r1b1kbnr/pppp1ppp/2n2q2/4p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 0 1")?;
        let new = make_move(&pos, &Move::new(board::A8, board::B8, Piece::Rook, false));
        let after = Position::from_fen("1rb1kbnr/pppp1ppp/2n2q2/4p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQk - 1 2")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_castling_revoked_king_move() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r1b1kbnr/pppp1ppp/2n2q2/4p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 0 1")?;
        let new = make_move(&pos, &Move::new(board::E8, board::E7, Piece::King, false));
        let after = Position::from_fen("r1b2bnr/ppppkppp/2n2q2/4p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQ - 1 2")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }

    #[test]
    fn zobrist_hash_castling_revoked_rook_capture() -> Result<(), FenParseError> {
        let pos = Position::from_fen("r1b1kbnr/ppp2ppp/1Nn2q2/4p3/2BpP3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 4")?;
        let new = make_move(&pos, &Move::new(board::B6, board::A8, Piece::Knight, true));
        let after = Position::from_fen("N1b1kbnr/ppp2ppp/2n2q2/4p3/2BpP3/5N2/PPPP1PPP/R1BQK2R b KQk - 0 4")?;
        assert_eq!(new.zobrist_hash, after.zobrist_hash);
        Ok(())
    }
}
