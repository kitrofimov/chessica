use crate::constants::GAME_HISTORY_CAPACITY;
use crate::engine::{
    board::position::{Position, FenParseError},
    board::movegen::pseudo_moves,
    board::rules::{
        make_move::*,
        unmake_move::*,
        draw::*,
        checks::*,
    },
    base::_move::Move,
};

/// Represents a chess game fully, including move history and halfmove clock
#[derive(Clone)]
pub struct Game {
    pub position: Position,
    pub undos: Vec<UndoData>,
    pub halfmove_clock: usize,
}

impl Default for Game {
    fn default() -> Self {
        let undos = Vec::with_capacity(GAME_HISTORY_CAPACITY);
        let position = Position::default();
        Game {
            position,
            undos,
            halfmove_clock: 0,
        }
    }
}

impl Game {
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let parsed = Position::from_fen(fen)?;
        Ok(Game {
            position: parsed.position,
            undos: Vec::with_capacity(GAME_HISTORY_CAPACITY),
            halfmove_clock: parsed.halfmove_clock,
        })
    }

    pub fn try_to_make_move(&mut self, m: &Move) -> bool {
        let mut clock = self.halfmove_clock;
        let undo = make_move(&mut self.position, m, &mut clock);

        // Check legality of a move (is player that made the move still in check?)
        // Using `.opposite()` because the flag was already flipped in `make_move`
        if is_king_in_check(&self.position, self.position.player_to_move.opposite()) {
            unmake_move(&mut self.position, undo, &mut clock);
            return false;
        }

        self.undos.push(undo);
        self.halfmove_clock = clock;

        true
    }

    pub fn unmake_move(&mut self) {
        let mut clock = self.halfmove_clock;  // double mutable borrow workaround
        unmake_move(&mut self.position, self.undos.pop().unwrap(), &mut clock);
        self.halfmove_clock = clock;
    }

    // UTTERLY INSANE IMPLEMENTATION that works and seems to be fast enough
    pub fn try_to_make_uci_move(&mut self, uci: &str) -> bool {
        let moves = self.pseudo_moves();
        for m in &moves {
            if m.to_string() == uci {
                return self.try_to_make_move(m);
            }
        }
        false
    }

    fn is_threefold_repetition(&self) -> bool {
        let current_hash = self.position.zobrist_hash;
        let mut count = 1;
        for undo in self.undos.iter().rev() {
            if undo.zobrist_hash == current_hash {
                count += 1;
                if count == 3 {
                    return true;
                }
            }
        }
        false
    }

    fn is_fifty_move_rule(&self) -> bool {
        self.halfmove_clock >= 100
    }

    fn is_insufficient_material(&self) -> bool {
        is_insufficient_material(&self.position)
    }

    pub fn pseudo_moves(&self) -> Vec<Move> {
        pseudo_moves(&self.position)
    }

    pub fn is_draw(&self) -> bool {
        self.is_threefold_repetition()
            || self.is_fifty_move_rule()
            || self.is_insufficient_material()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::board;
    use crate::engine::base::piece::Piece;

    #[test]
    fn threefold_repetition() -> Result<(), FenParseError> {
        let mut game = Game::from_fen("8/2r5/8/4k3/8/6R1/3K4/8 w - - 0 1")?;

        let m1 = Move::new(board::G3, board::F3, Piece::Rook, false);
        let m2 = Move::new(board::C7, board::C6, Piece::Rook, false);
        let m3 = Move::new(board::F3, board::G3, Piece::Rook, false);
        let m4 = Move::new(board::C6, board::C7, Piece::Rook, false);

        for _ in 0..2 {
            game.try_to_make_move(&m1);
            game.try_to_make_move(&m2);
            game.try_to_make_move(&m3);
            game.try_to_make_move(&m4);
        }

        assert_eq!(game.is_threefold_repetition(), true);
        Ok(())
    }

    #[test]
    fn fifty_move_rule() -> Result<(), FenParseError> {
        let mut game = Game::from_fen("8/3k4/1n6/8/8/5N2/3K4/8 w - - 99 1")?;
        let m = Move::new(board::F3, board::G5, Piece::Knight, false);
        game.try_to_make_move(&m);
        assert_eq!(game.halfmove_clock, 100);
        Ok(())
    }
}
