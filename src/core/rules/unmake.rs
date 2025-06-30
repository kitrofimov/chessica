use crate::constants::board;
use crate::core::{
    position::*,
    bitboard::*,
    chess_move::*,
    player::Player,
    piece::Piece,
};

#[derive(Clone)]
pub struct UndoData {
    pub move_to_undo: Move,
    pub captured_piece: Option<Piece>,
    pub castling: CastlingRights,
    pub en_passant_square: Option<u8>,
    pub halfmove_clock: usize,
    pub zobrist_hash: u64,
}

pub fn unmake_move(pos: &mut Position, undo: UndoData, halfmove_clock: &mut usize) {
    let who_moved = pos.player_to_move.opposite();
    let m = undo.move_to_undo;

    pos.castling = undo.castling;
    pos.en_passant_square = undo.en_passant_square;
    pos.zobrist_hash = undo.zobrist_hash;
    *halfmove_clock = undo.halfmove_clock;
    pos.player_to_move = who_moved;

    let (friendly, hostile) = pos.perspective_mut(who_moved);

    if m.is_castling() {
        undo_castling(pos, &m, who_moved);
    } else {
        if m.promotion.is_some() {
            undo_promotion(friendly, &m);
        } else {
            undo_non_promotion_move(friendly, &m);
        }

        if m.en_passant {
            undo_en_passant(hostile, &m, who_moved);
        } else if m.capture {
            undo_capture(hostile, &m, undo.captured_piece.unwrap());
        }
    }

    pos.update();
}

fn undo_castling(pos: &mut Position, m: &Move, who: Player) {
    let (friendly, _) = pos.perspective_mut(who);

    let (rook_from, rook_to) = match (who, m.kingside_castling, m.queenside_castling) {
        (Player::White, true, _) => (board::H1, board::F1),
        (Player::White, _, true) => (board::A1, board::D1),
        (Player::Black, true, _) => (board::H8, board::F8),
        (Player::Black, _, true) => (board::A8, board::D8),
        _ => unreachable!(),
    };

    friendly.unset_bit(m.to);
    friendly.set_bit(m.from, Piece::King);

    friendly.unset_bit(rook_to);
    friendly.set_bit(rook_from, Piece::Rook);
}

fn undo_promotion(friendly: &mut BitboardSet, m: &Move) {
    friendly.unset_bit(m.to);
    friendly.set_bit(m.from, Piece::Pawn);
}

fn undo_non_promotion_move(friendly: &mut BitboardSet, m: &Move) {
    friendly.unset_bit(m.to);
    friendly.set_bit(m.from, m.piece);
}

fn undo_capture(hostile: &mut BitboardSet, m: &Move, captured: Piece) {
    hostile.set_bit(m.to, captured);
}

fn undo_en_passant(hostile: &mut BitboardSet, m: &Move, who: Player) {
    let sq = match who {
        Player::White => m.to - 8,
        Player::Black => m.to + 8,
    };
    hostile.set_bit(sq, Piece::Pawn);
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rules::make::make_move;

    #[test]
    fn unmake_move_normal_move() {
        let (mut pos, mut clock) = Position::from_fen("8/3r4/2k5/8/5R2/2K5/8/8 w - - 0 1").unwrap();
        let save = pos;
        let m = Move::new(board::F4, board::F8, Piece::Rook, false);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_capture() {
        let (mut pos, mut clock) = Position::from_fen("2b5/5k2/8/4n3/8/6B1/1K6/8 w - - 0 1").unwrap();
        let save = pos;
        let m = Move::new(board::G3, board::E5, Piece::Bishop, true);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_promotion() {
        let (mut pos, mut clock) = Position::from_fen("8/2P5/5k2/1K6/8/8/8/8 w - - 0 1").unwrap();
        let save = pos;
        let m = Move::pawn(board::C7, board::C8, false, Some(Piece::Queen), false);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_en_passant() {
        let (mut pos, mut clock) = Position::from_fen("8/8/5k2/1KPp4/8/8/8/8 w - d6 0 1").unwrap();
        let save = pos;
        let m = Move::pawn(board::C5, board::D6, true, None, true);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_castling() {
        let (mut pos, mut clock) = Position::from_fen("5b2/1q1pp2p/5k2/8/6Q1/8/P4PPP/2B1K2R w K - 0 1").unwrap();
        let save = pos;
        let m = Move::castling(Player::White, CastlingSide::KingSide);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_castling_rights_rook_move() {
        let (mut pos, mut clock) = Position::from_fen("5b2/1q1pp2p/5k2/8/6Q1/8/P4PPP/2B1K2R w K - 0 1").unwrap();
        let save = pos;
        let m = Move::new(board::H1, board::F1, Piece::Rook, false);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_castling_rights_king_move() {
        let (mut pos, mut clock) = Position::from_fen("5b2/1q1pp2p/5k2/8/6Q1/8/P4PPP/2B1K2R w K - 0 1").unwrap();
        let save = pos;
        let m = Move::new(board::E1, board::D2, Piece::King, false);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_castling_rights_rook_capture() {
        let (mut pos, mut clock) = Position::from_fen("5b2/1q1pp2p/5k2/8/6Q1/6n1/P4PPP/2B1K2R b K - 0 1").unwrap();
        let save = pos;
        let m = Move::new(board::G3, board::H1, Piece::Knight, true);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
    }

    #[test]
    fn unmake_move_castling_rights_clock() {
        let (mut pos, mut clock) = Position::from_fen("8/8/1k1p4/2p5/6P1/7P/5K2/8 b - - 39 100").unwrap();
        let save = pos;
        let save_clock = clock;
        let m = Move::new(board::B6, board::B5, Piece::King, false);
        let undo = make_move(&mut pos, &m, &mut clock);
        unmake_move(&mut pos, undo, &mut clock);
        assert_eq!(pos, save);
        assert_eq!(clock, save_clock);
    }
}
