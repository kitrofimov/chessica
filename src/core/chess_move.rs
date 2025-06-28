use crate::{constants::board, core::{piece::Piece, player::Player}, utility::square_idx_to_string};

// TODO: will tightly-packing this improve performance?
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: Piece,
    pub capture: bool,
    pub promotion: Option<Piece>,
    pub en_passant: bool,
    pub double_push: bool,
    pub kingside_castling: bool,
    pub queenside_castling: bool,
}

impl ToString for Move {
    // Long algebraic notation, UCI-compliant
    fn to_string(&self) -> String {
        let mut s = String::new();
        s += &square_idx_to_string(self.from);
        s += &square_idx_to_string(self.to);
        if let Some(promotion_piece) = self.promotion {
            s += &promotion_piece.to_char().to_string();
        }
        s
    }
}

impl Move {
    pub fn new(from: u8, to: u8, piece: Piece, capture: bool) -> Self {
        Move {
            from,
            to,
            piece,
            capture,
            promotion: None,
            en_passant: false,
            double_push: false,
            kingside_castling: false,
            queenside_castling: false,
        }
    }

    pub fn pawn(from: u8, to: u8, capture: bool, promotion: Option<Piece>, en_passant: bool) -> Self {
        Move {
            from,
            to,
            piece: Piece::Pawn,
            capture,
            promotion,
            en_passant,
            double_push: to.wrapping_sub(from) == 16 || from.wrapping_sub(to) == 16,
            kingside_castling: false,
            queenside_castling: false,
        }
    }

    pub fn castling(player: Player, side: CastlingSide) -> Self {
        Move {
            from: match player {
                Player::White => board::E1,
                Player::Black => board::E8,
            },
            to: match (player, side) {
                (Player::White, CastlingSide::KingSide)  => board::G1,
                (Player::White, CastlingSide::QueenSide) => board::C1,
                (Player::Black, CastlingSide::KingSide)  => board::G8,
                (Player::Black, CastlingSide::QueenSide) => board::C8,
            },
            piece: Piece::King,
            capture: false,
            promotion: None,
            en_passant: false,
            double_push: false,
            kingside_castling: side == CastlingSide::KingSide,
            queenside_castling: side == CastlingSide::QueenSide,
        }
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CastlingSide {
    KingSide,
    QueenSide
}


#[derive(Clone, Copy, PartialEq, Eq)]
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
}
