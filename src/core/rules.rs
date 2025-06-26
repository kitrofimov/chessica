use crate::constants::{attacks, board};
use crate::utility::pop_lsb;
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
    let (friendly, hostile, kingside, queenside) = match who_made_move {
        Player::White => (
            &mut new.w, &mut new.b,
            &mut new.castling.white_kingside, &mut new.castling.white_queenside
        ),
        Player::Black => (
            &mut new.b, &mut new.w,
            &mut new.castling.black_kingside, &mut new.castling.black_queenside
        ),
    };

    new.en_passant_square = calculate_en_passant_square(m);

    if m.kingside_castling || m.queenside_castling {
        handle_castling(m, friendly, who_made_move, kingside, queenside);
    } else {
        update_castling_rights(m, kingside, queenside);

        if let Some(promotion_piece) = m.promotion {
            handle_promotion(m, friendly, promotion_piece);
        } else {
            handle_non_promotion_move(m, friendly);
        }

        if m.en_passant {
            handle_en_passant(m, hostile, who_made_move);
        } else if m.capture {
            handle_capture(m, hostile, &mut new.castling);
        }
    }

    new.update();
    new.player_to_move = who_made_move.opposite();
    new
}

fn calculate_en_passant_square(m: &Move) -> Option<u8> {
    if m.double_push {
        Some((m.from + m.to) / 2)
    } else {
        None
    }
}

fn handle_castling(
    m: &Move,
    friendly: &mut BitboardSet,
    who_made_move: Player,
    kingside: &mut bool,
    queenside: &mut bool
) {
    let (rook_from, rook_to) = match (who_made_move, m.kingside_castling, m.queenside_castling) {
        (Player::White, true, _) => (board::H1, board::F1),
        (Player::White, _, true) => (board::A1, board::D1),
        (Player::Black, true, _) => (board::H8, board::F8),
        (Player::Black, _, true) => (board::A8, board::D8),
        _ => unreachable!(),
    };
    friendly.king = friendly.king.unset_bit(m.from).set_bit(m.to);
    friendly.rooks = friendly.rooks.unset_bit(rook_from).set_bit(rook_to);

    *kingside = false;  // can't castle twice :)
    *queenside = false;
}

// There is also special code updating castling rights in `handle_capture`
fn update_castling_rights(m: &Move, kingside: &mut bool, queenside: &mut bool) {
    if m.piece == Piece::King {
        *kingside = false;
        *queenside = false;
    } else if m.piece == Piece::Rook && !(*kingside == false && *queenside == false) {
        if m.from == board::A1 || m.from == board::A8 {
            *queenside = false;
        } else if m.from == board::H1 || m.from == board::H8 {
            *kingside = false
        }
    }
}

fn handle_promotion(m: &Move, friendly: &mut BitboardSet, promotion_piece: Piece) {
    friendly.pawns = friendly.pawns.unset_bit(m.from);
    let bb = friendly.piece_to_bb_mut(promotion_piece);
    *bb = bb.set_bit(m.to);
}

fn handle_non_promotion_move(m: &Move, friendly: &mut BitboardSet) {
    let bb = friendly.piece_to_bb_mut(m.piece);
    *bb = bb.unset_bit(m.from).set_bit(m.to);
}

fn handle_en_passant(m: &Move, hostile: &mut BitboardSet, who_made_move: Player) {
    match who_made_move {
        Player::White => hostile.unset_bit(m.to - 8),
        Player::Black => hostile.unset_bit(m.to + 8),
    }
}

fn handle_capture(m: &Move, hostile: &mut BitboardSet, castling: &mut CastlingRights) {
    hostile.unset_bit(m.to);

    // Capturing rook square disables castling - harmless if no rook was there
    match m.to {
        board::A1 => castling.white_queenside = false,
        board::H1 => castling.white_kingside = false,
        board::A8 => castling.black_queenside = false,
        board::H8 => castling.black_kingside = false,
        _ => {}
    }
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
}
