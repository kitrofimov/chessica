use crate::constants::attacks;
use crate::utility::lsb;
use crate::core::{
    position::*,
    movegen::*,
    player::Player,
};

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
    let king_bb = match player {
        Player::White => pos.w.king,
        Player::Black => pos.b.king,
    };
    is_square_attacked(pos, lsb(king_bb).into(), player.opposite())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_square_attacked_endgame() -> Result<(), FenParseError> {
        let (pos, _) = Position::from_fen("8/3r1k2/8/4N3/1Q5q/8/2K5/8 b - - 0 1")?;
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
        let (pos, _) = Position::from_fen("r1bqkb1r/ppp2ppp/5n2/1B4Q1/1n1P2N1/2N5/PPP2PPP/R1B1K2R b KQkq - 0 1")?;
        assert_eq!(is_king_in_check(&pos, Player::White), false);
        assert_eq!(is_king_in_check(&pos, Player::Black), true);
        Ok(())
    }

    #[test]
    fn is_king_in_check_midgame_2() -> Result<(), FenParseError> {
        let (pos, _) = Position::from_fen("r1bqk1nr/pppp2pp/2n5/1B2pp2/1b1PP3/5N2/PPP2PPP/RNBQK2R w KQkq - 0 1")?;
        assert_eq!(is_king_in_check(&pos, Player::White), true);
        assert_eq!(is_king_in_check(&pos, Player::Black), false);
        Ok(())
    }

    #[test]
    fn is_king_in_check_endgame() -> Result<(), FenParseError> {
        let (pos, _) = Position::from_fen("R6k/8/7K/8/8/1b6/8/8 b - - 0 1")?;
        assert_eq!(is_king_in_check(&pos, Player::White), false);
        assert_eq!(is_king_in_check(&pos, Player::Black), true);
        Ok(())
    }
}
