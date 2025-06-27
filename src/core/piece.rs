#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
}
