use std::{
    cmp::{max, min},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant}
};
use crate::constants::*;
use crate::core::{
    chess_move::*,
    evaluate::evaluate,
    movegen::pseudo_moves,
    player::Player,
    position::*,
    rules::{
        make::*,
        unmake::*,
        draw::*,
        checks::*
    }
};

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
        Game { position, undos, halfmove_clock: 0 }
    }
}

impl Game {
    pub fn new(pos: Position) -> Game {
        let undos = Vec::with_capacity(GAME_HISTORY_CAPACITY);
        Game { position: pos, undos, halfmove_clock: 0 }
    }

    pub fn from_fen(fen: &str) -> Result<Game, FenParseError> {
        let (position, clock) = Position::from_fen(fen)?;
        let undos = Vec::with_capacity(GAME_HISTORY_CAPACITY);
        Ok(Game { position, undos, halfmove_clock: clock })
    }

    pub fn pseudo_moves(&self) -> Vec<Move> {
        pseudo_moves(&self.position)
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
        let mut clock = self.halfmove_clock;
        unmake_move(&mut self.position, self.undos.pop().unwrap(), &mut clock);
        self.halfmove_clock = clock;
    }

    // UTTERLY INSANE IMPLEMENTATION that works
    // this does not need to be *really* fast (called rarely), it's fast *enough*
    // TODO: ^^^ is this really true? `position startpos moves ...` goes after
    // every move of a human...
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

    // Returns (eval, unwind)
    fn minimax_alphabeta(
        &mut self,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        maximize: bool,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        nodes: &mut u64
    ) -> (i32, bool) {
        *nodes += 1;

        if self.is_threefold_repetition() ||
            self.is_fifty_move_rule() ||
            self.is_insufficient_material() {
            return (0, false);  // draw
        }

        // Unwind the search if `stop_flag` was set or time is over
        if stop_flag.load(Ordering::Relaxed)
            || time_limit.map(|tl| start_time.elapsed() >= tl).unwrap_or(false) {
            // TODO: is it correct to evaluate the position here?
            return (evaluate(&self.position), true);
        }

        if depth == 0 {
            return (evaluate(&self.position), false);
        }

        let moves = self.pseudo_moves();
        let mut best = if maximize { i32::MIN } else { i32::MAX };
        let mut found_legal_move = false;

        for m in &moves {
            let legal = self.try_to_make_move(m);
            if !legal {
                continue;
            }

            found_legal_move = true;
            let (eval, unwind) = self.minimax_alphabeta(
                depth - 1,
                alpha,
                beta,
                !maximize,
                stop_flag,
                start_time,
                time_limit,
                nodes
            );
            self.unmake_move();
            if unwind {
                return (best, true);
            }

            if maximize {
                best = max(best, eval);
                alpha = max(alpha, eval);
            } else {
                best = min(best, eval);
                beta = min(beta, eval);
            }

            if beta <= alpha {
                break;
            }
        }

        if !found_legal_move {
            // Checkmate
            if is_king_in_check(&self.position, self.position.player_to_move) {
                return (match self.position.player_to_move {  // losing sooner is worse
                    Player::White => -10_000 + depth as i32,
                    Player::Black =>  10_000 - depth as i32,
                }, false);
            } else {  // Draw
                return (0, false);
            }
        }

        (best, false)
    }

    // Returns (best_move, best_score, nodes, unwind)
    pub fn find_best_move(
        &mut self,
        depth: usize,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>
    ) -> (Option<Move>, i32, u64, bool) {
        let mut best_move = None;
        let (mut best_score, maximize) = match self.position.player_to_move {
            Player::White => (i32::MIN, true),
            Player::Black => (i32::MAX, false),
        };

        let mut nodes = 0;

        for m in self.pseudo_moves() {
            let legal = self.try_to_make_move(&m);
            if !legal {
                continue;
            }
            let (eval, unwind) = self.minimax_alphabeta(
                depth - 1,
                i32::MIN,
                i32::MAX,
                maximize,
                stop_flag,
                start_time,
                time_limit,
                &mut nodes
            );
            self.unmake_move();

            if (maximize && eval > best_score) || (!maximize && eval < best_score) {
                best_score = eval;
                best_move = Some(m);
            }

            if unwind {
                return (best_move, best_score, nodes, true);
            }
        }

        (best_move, best_score, nodes, false)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::board;
    use crate::core::piece::Piece;

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
