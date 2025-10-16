#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    White, Black
}

impl Player {
    pub fn opposite(&self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Player::White => 0,
            Player::Black => 1
        }
    }
}
