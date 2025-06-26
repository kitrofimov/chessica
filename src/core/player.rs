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
}
