#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn to_char(&self) -> char {
        match self {
            Piece::Pawn   => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook   => 'r',
            Piece::Queen  => 'q',
            Piece::King   => 'k',
        }
    }

    pub fn all_variants() -> [Piece; 6] {
        [Piece::Pawn, Piece::Knight, Piece::Bishop,
         Piece::Rook, Piece::Queen,  Piece::King]
    }

    // Used in evaluation function
    pub fn value(&self) -> i32 {
        match self {
            Piece::Pawn   => 100,
            Piece::Knight => 300,
            Piece::Bishop => 330,
            Piece::Rook   => 500,
            Piece::Queen  => 900,
            Piece::King   => 100_000,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Piece::Pawn   => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook   => 3,
            Piece::Queen  => 4,
            Piece::King   => 5,
        }
    }

    pub fn encode(&self) -> u8 {
        *self as u8
    }

    pub fn decode(x: u8) -> Option<Piece> {
        if x == Piece::empty() {
            return None;
        }
        match x {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => unreachable!(),
        }
    }

    pub const fn empty() -> u8 {
        0b111
    }
}
