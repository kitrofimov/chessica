use crate::constants::{
    SEE_EXCHANGE_MAX_DEPTH,
    attacks::{PAWN_ATTACKS_WHITE, PAWN_ATTACKS_BLACK}
};
use crate::utility::lsb;
use crate::engine::{
    board::{
        position::Position,
        movegen::*,
    },
    base::{
        _move::Move,
        piece::Piece,
        player::Player,
        bitboard::*,
    },
};

impl Position {
    pub fn static_exchange_eval(&self, m: Move) -> i32 {
        assert!(m.is_capture());

        let to = m.to as usize;  // reduce "as usize" spam
        let mut gain = [0i32; SEE_EXCHANGE_MAX_DEPTH];
        let mut d = 0;

        gain[d] = if let Some((_, captured)) = self.piece_at(m.to) {
            captured.value()
        } else {  // En passant
            Piece::Pawn.value()
        };

        let mut occ = self.all();  // occupancy mask

        // If en passant, take off the captured piece (interferes with sliding pieces' attacks)
        if m.en_passant {
            let capture_sq = m.en_passant_sq_to_captured_sq(self.player_to_move);
            occ.unset_bit(capture_sq);
        }

        // Take off attacking piece
        occ = occ.unset_bit(m.from);

        // Prepare for loop
        let mut attackers = self.attackers_to_with_occ(to, occ);
        let mut side = self.player_to_move.opposite();

        // Exchange loop
        while attackers != 0 {
            d += 1;

            // Find least valuable attacker
            let (attack_sq, attack_piece) = 
                if let Some(pair) = self.least_valuable_attacker(attackers, side) {
                    pair
                } else {
                    break;
                };

            gain[d] = attack_piece.value() - gain[d-1].max(0);

            // Take off the attacker
            occ = occ.unset_bit(attack_sq as u8);

            attackers = self.attackers_to_with_occ(to, occ);
            side = side.opposite();
        }

        // Choose the best option for the side starting the exchange
        for i in (1..=d).rev() {
            gain[i - 1] = -gain[i - 1].max(-gain[i]);
        }

        gain[0]
    }

    /// Return a mask of all pieces attacking the square, assuming every
    /// piece not present in occupancy mask is nonexistent for sliding pieces
    fn attackers_to_with_occ(&self, sq: usize, occ: Bitboard) -> Bitboard {
        let mut attackers = 0u64;
        let p = self;

        // Not a bug: I really mean this black-white inversion :)
        attackers |= PAWN_ATTACKS_BLACK[sq]       &  p.w.pawns & occ;
        attackers |= PAWN_ATTACKS_WHITE[sq]       &  p.b.pawns & occ;
        attackers |= knight_attacks    (p, sq, 0) & (p.w.knights | p.b.knights) & occ;
        attackers |= king_attacks      (p, sq, 0) & (p.w.king    | p.b.king)    & occ;
        attackers |= bishop_attacks_occ(occ, sq)  & (p.w.bishops | p.b.bishops | p.w.queens | p.b.queens) & occ;
        attackers |= rook_attacks_occ  (occ, sq)  & (p.w.rooks   | p.b.rooks   | p.w.queens | p.b.queens) & occ;

        attackers
    }

    /// Finds (square, piece) of the least valuable attacker from `side`
    fn least_valuable_attacker(
        &self,
        attackers: Bitboard,
        side: Player,
    ) -> Option<(usize, Piece)> {
        let (bb, _) = self.perspective(side);

        // Check pieces in order of increasing value
        let piece_types = [
            (bb.pawns,   Piece::Pawn),
            (bb.knights, Piece::Knight),
            (bb.bishops, Piece::Bishop),
            (bb.rooks,   Piece::Rook),
            (bb.queens,  Piece::Queen),
            (bb.king,    Piece::King),
        ];

        for (piece_bb, piece) in &piece_types {
            // Attackers of this piece type
            let subset = piece_bb & attackers;
            if subset != 0 {
                // Take some attacker, it does not matter
                let sq = lsb(subset) as usize;
                return Some((sq, *piece));
            }
        }

        None
    }
}
