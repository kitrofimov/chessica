use crate::{core::piece::Piece, utility::bit};

pub type Bitboard = u64;

pub trait BitOps {
    fn set_bit(self, n: u8) -> Self;
    fn unset_bit(self, n: u8) -> Self;
}

impl BitOps for Bitboard {
    fn set_bit(self, n: u8) -> Self {
        self | (1 << n)
    }

    fn unset_bit(self, n: u8) -> Self {
        self & !(1 << n)
    }
}


#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitboardSet {
    pub all:     Bitboard,
    pub pawns:   Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks:   Bitboard,
    pub queens:  Bitboard,
    pub king:    Bitboard,  // only one king per side, so not plural :)
}

impl BitboardSet {
    pub fn update(&mut self) {
        self.all = self.pawns | self.knights | self.bishops |
                   self.rooks | self.queens  | self.king;
    }

    pub fn piece_to_bb_mut(&mut self, piece: Piece) -> &mut Bitboard {
        match piece {
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Rook   => &mut self.rooks,
            Piece::Queen  => &mut self.queens,
            Piece::King   => &mut self.king,
            Piece::Pawn   => &mut self.pawns,
        }
    }

    fn piece_to_bb(&self, piece: Piece) -> &Bitboard {
        match piece {
            Piece::Knight => &self.knights,
            Piece::Bishop => &self.bishops,
            Piece::Rook   => &self.rooks,
            Piece::Queen  => &self.queens,
            Piece::King   => &self.king,
            Piece::Pawn   => &self.pawns,
        }
    }

    pub fn unset_bit(&mut self, n: u8) {
        self.all     = self.all.unset_bit(n);
        self.pawns   = self.pawns.unset_bit(n);
        self.knights = self.knights.unset_bit(n);
        self.bishops = self.bishops.unset_bit(n);
        self.rooks   = self.rooks.unset_bit(n);
        self.queens  = self.queens.unset_bit(n);
        self.king    = self.king.unset_bit(n);
    }

    pub fn set_bit(&mut self, n: u8, piece: Piece) {
        self.all     = self.all.set_bit(n);
        let bb = self.piece_to_bb_mut(piece);
        *bb = bb.set_bit(n);
    }

    pub fn count(&self, piece: Piece) -> u32 {
        self.piece_to_bb(piece).count_ones()
    }

    pub fn count_all(&self) -> u32 {
        let mut result = 0;
        for p in Piece::all_variants() {
            result += self.piece_to_bb(p).count_ones();
        }
        result
    }

    pub fn what(&self, sq_idx: u8) -> Option<Piece> {
        let bb = bit(sq_idx);
        if self.pawns & bb != 0 {
            Some(Piece::Pawn)
        } else if self.knights & bb != 0 {
            Some(Piece::Knight)
        } else if self.bishops & bb != 0 {
            Some(Piece::Bishop)
        } else if self.rooks & bb != 0 {
            Some(Piece::Rook)
        } else if self.queens & bb != 0 {
            Some(Piece::Queen)
        } else if self.king & bb != 0 {
            Some(Piece::King)
        } else {
            None
        }
    }
}
