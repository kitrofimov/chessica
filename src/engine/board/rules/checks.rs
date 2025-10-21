use crate::constants::attacks;
use crate::utility::lsb;
use crate::engine::{
    base::player::Player,
    board::{position::*, movegen::*},
};

impl Position {
    pub fn is_square_attacked(&self, sq: usize, by_player: Player) -> bool {
        let friend = match by_player {
            Player::White => &self.w,
            Player::Black => &self.b,
        };

        // All the possible pieces' positions, which could attack this square
        // reversing intentionally, questioning: "what could have attacked this square?"
        let pawn = match by_player {
            Player::White => attacks::PAWN_ATTACKS_BLACK[sq],
            Player::Black => attacks::PAWN_ATTACKS_WHITE[sq],
        };
        let knight = knight_attacks(self, sq, 0x0);
        let bishop = bishop_attacks(self, sq, 0x0);
        let rook   = rook_attacks  (self, sq, 0x0);
        let queen  = queen_attacks (self, sq, 0x0);
        let king   = king_attacks  (self, sq, 0x0);

        pawn   & friend.pawns   > 0 || knight & friend.knights > 0 ||
        bishop & friend.bishops > 0 || rook   & friend.rooks   > 0 ||
        queen  & friend.queens  > 0 || king   & friend.king    > 0
    }

    pub fn is_king_in_check(&self, player: Player) -> bool {
        let king_bb = match player {
            Player::White => self.w.king,
            Player::Black => self.b.king,
        };
        self.is_square_attacked(lsb(king_bb).into(), player.opposite())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_square_attacked_endgame() -> Result<(), FenParseError> {
        let parsed = Position::from_fen("8/3r1k2/8/4N3/1Q5q/8/2K5/8 b - - 0 1")?;
        let pos = parsed.position;
        assert_eq!(pos.is_square_attacked(53, Player::White), true);
        assert_eq!(pos.is_square_attacked(51, Player::White), true);
        assert_eq!(pos.is_square_attacked(20, Player::White), false);
        assert_eq!(pos.is_square_attacked(25, Player::Black), true);
        assert_eq!(pos.is_square_attacked(52, Player::Black), true);
        assert_eq!(pos.is_square_attacked(10, Player::Black), false);
        Ok(())
    }

    #[test]
    fn is_king_in_check_midgame_1() -> Result<(), FenParseError> {
        let parsed = Position::from_fen("r1bqkb1r/ppp2ppp/5n2/1B4Q1/1n1P2N1/2N5/PPP2PPP/R1B1K2R b KQkq - 0 1")?;
        let pos = parsed.position;
        assert_eq!(pos.is_king_in_check(Player::White), false);
        assert_eq!(pos.is_king_in_check(Player::Black), true);
        Ok(())
    }

    #[test]
    fn is_king_in_check_midgame_2() -> Result<(), FenParseError> {
        let parsed = Position::from_fen("r1bqk1nr/pppp2pp/2n5/1B2pp2/1b1PP3/5N2/PPP2PPP/RNBQK2R w KQkq - 0 1")?;
        let pos = parsed.position;
        assert_eq!(pos.is_king_in_check(Player::White), true);
        assert_eq!(pos.is_king_in_check(Player::Black), false);
        Ok(())
    }

    #[test]
    fn is_king_in_check_endgame() -> Result<(), FenParseError> {
        let parsed = Position::from_fen("R6k/8/7K/8/8/1b6/8/8 b - - 0 1")?;
        let pos = parsed.position;
        assert_eq!(pos.is_king_in_check(Player::White), false);
        assert_eq!(pos.is_king_in_check(Player::Black), true);
        Ok(())
    }
}
