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

    pub fn position(&self) -> &Position {
        self.positions.last().unwrap()
    }

    pub fn generate_pseudo_moves(&self) -> Vec<Move> {
        let pos = self.positions.last().unwrap();
        pseudo_moves(pos)
    }

    pub fn try_to_make_move(&mut self, m: &Move) -> bool {
        let mut pos = *self.positions.last().unwrap();
        pos.make_move(m);

        // Check legality of a move (is player that made the move still in check?)
        // pos.make_move already flipped the flag, so we flip it the second time
        // terrible workaround, but works
        if pos.is_king_in_check(pos.player_to_move.opposite()) {
            return false;
        }

        self.positions.push(pos);
        true
    }

    pub fn unmake_move(&mut self) {
        self.positions.pop();
    }
}
