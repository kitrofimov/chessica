use crate::{constants::board, core::{piece::Piece, player::Player}, utility::square_idx_to_string};

const FLAG_CAPTURE: u8 = 1 << 0;
const FLAG_EN_PASSANT: u8 = 1 << 1;
const FLAG_DOUBLE_PUSH: u8 = 1 << 2;
const FLAG_KINGSIDE_CASTLING: u8 = 1 << 3;
const FLAG_QUEENSIDE_CASTLING: u8 = 1 << 4;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub pieces_field: u8,
    pub flags: u8,
}

impl ToString for Move {
    // Long algebraic notation, UCI-compliant
    fn to_string(&self) -> String {
        let mut s = String::new();
        s += &square_idx_to_string(self.from);
        s += &square_idx_to_string(self.to);
        if let Some(promotion_piece) = self.promotion() {
            s += &promotion_piece.to_char().to_string();
        }
        s
    }
}

impl Move {
    fn encode_pieces(piece: Piece, promotion: Option<Piece>) -> u8 {
        let base = piece.encode() & 0x0F;
        let prom = promotion.map(|p| p.encode()).unwrap_or(Piece::empty());
        prom << 4 | base
    }

    pub fn new(from: u8, to: u8, piece: Piece, capture: bool) -> Self {
        Move {
            from,
            to,
            pieces_field: Self::encode_pieces(piece, None),
            flags: if capture { FLAG_CAPTURE } else { 0 },
        }
    }

    pub fn pawn(from: u8, to: u8, capture: bool, promotion: Option<Piece>, en_passant: bool) -> Self {
        let mut flags = 0;
        if capture { flags |= FLAG_CAPTURE; }
        if en_passant { flags |= FLAG_EN_PASSANT; }
        if to.wrapping_sub(from) == 16 || from.wrapping_sub(to) == 16 {
            flags |= FLAG_DOUBLE_PUSH;
        }

        Move {
            from,
            to,
            pieces_field: Self::encode_pieces(Piece::Pawn, promotion),
            flags,
        }
    }

    pub fn castling(player: Player, side: CastlingSide) -> Self {
        let (from, to, flag) = match (player, side) {
            (Player::White, CastlingSide::KingSide)  => (board::E1, board::G1, FLAG_KINGSIDE_CASTLING),
            (Player::White, CastlingSide::QueenSide) => (board::E1, board::C1, FLAG_QUEENSIDE_CASTLING),
            (Player::Black, CastlingSide::KingSide)  => (board::E8, board::G8, FLAG_KINGSIDE_CASTLING),
            (Player::Black, CastlingSide::QueenSide) => (board::E8, board::C8, FLAG_QUEENSIDE_CASTLING),
        };

        Move {
            from,
            to,
            pieces_field: Self::encode_pieces(Piece::King, None),
            flags: flag,
        }
    }

    pub fn piece(&self) -> Piece {
        Piece::decode(self.pieces_field & 0x0F).unwrap()
    }

    pub fn promotion(&self) -> Option<Piece> {
        Piece::decode(self.pieces_field >> 4)
    }

    pub fn is_capture(&self) -> bool {
        self.flags & FLAG_CAPTURE != 0
    }

    pub fn is_en_passant(&self) -> bool {
        self.flags & FLAG_EN_PASSANT != 0
    }

    pub fn is_double_push(&self) -> bool {
        self.flags & FLAG_DOUBLE_PUSH != 0
    }

    pub fn is_kingside_castling(&self) -> bool {
        self.flags & FLAG_KINGSIDE_CASTLING != 0
    }

    pub fn is_queenside_castling(&self) -> bool {
        self.flags & FLAG_QUEENSIDE_CASTLING != 0
    }

    pub fn is_castling(&self) -> bool {
        self.flags & (FLAG_KINGSIDE_CASTLING | FLAG_QUEENSIDE_CASTLING) != 0
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CastlingSide {
    KingSide,
    QueenSide
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true
        }
    }
}

impl ToString for CastlingRights {
    // FEN-like castling rights string
    fn to_string(&self) -> String {
        let mut s = String::new();
        if self.white_kingside {
            s += "K";
        }
        if self.white_queenside {
            s += "Q";
        }
        if self.black_kingside {
            s += "k";
        }
        if self.black_queenside {
            s += "q";
        }
        if s == "" {
            return "-".into();
        }
        s
    }
}

impl CastlingRights {
    // Parse castling rights from a FEN-like string (KQkq)
    pub fn from_string(s: &str) -> Self {
        let mut rights = CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        };
        if s.contains("K") {
            rights.white_kingside = true;
        }
        if s.contains("Q") {
            rights.white_queenside = true;
        }
        if s.contains("k") {
            rights.black_kingside = true;
        }
        if s.contains("q") {
            rights.black_queenside = true;
        }
        rights
    }

    pub fn encode(&self) -> u8 {
        (self.white_kingside  as u8) << 0 |
        (self.white_queenside as u8) << 1 |
        (self.black_kingside  as u8) << 2 |
        (self.black_queenside as u8) << 3
    }

    pub fn reset(&mut self, player: Player) {
        match player {
            Player::White => {
                self.white_kingside = false;
                self.white_queenside = false;
            }
            Player::Black => {
                self.black_kingside = false;
                self.black_queenside = false;
            }
        }
    }

    pub fn reset_side(&mut self, player: Player, side: CastlingSide) {
        match (player, side) {
            (Player::White, CastlingSide::QueenSide) => self.white_queenside = false,
            (Player::White, CastlingSide::KingSide)  => self.white_kingside  = false,
            (Player::Black, CastlingSide::QueenSide) => self.black_queenside = false,
            (Player::Black, CastlingSide::KingSide)  => self.black_kingside  = false,
        }
    }

    pub fn any(&self, player: Player) -> bool {
        match player {
            Player::White => self.white_kingside | self.white_queenside,
            Player::Black => self.black_kingside | self.black_queenside,
        }
    }
}
