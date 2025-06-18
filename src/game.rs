use crate::{movegen::pseudo_moves, position::{Move, Position}};

pub struct Game {
    positions: Vec<Position>,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            positions: vec![Position::default()]
        }
    }
}

impl Game {
    pub fn new(pos: Position) -> Game {
        Game {
            positions: vec![pos]
        }
    }

    pub fn generate_pseudo_moves(&self) -> Vec<Move> {
        let pos = self.positions.last().unwrap();
        pseudo_moves(*pos)
    }

    pub fn make_move(&mut self, m: &Move) {
        let mut pos = *self.positions.last().unwrap();
        pos.make_move(m);
        self.positions.push(pos);
    }

    pub fn unmake_move(&mut self) {
        self.positions.pop();
    }
}
