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

    // Returns (best_move, best_eval, pv, unwind)
    // PV is REVERSED (leaf -> root), reverse it when printing to get normal root -> leaf
    fn minimax_alphabeta(
        &mut self,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        maximize: bool,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>,
        nodes: &mut u64,
    ) -> (Option<Move>, i32, Vec<Move>, bool) {
        *nodes += 1;

        if self.is_threefold_repetition() ||
            self.is_fifty_move_rule() ||
            self.is_insufficient_material() {
            return (None, DRAW_EVAL, Vec::new(), false);
        }

        if depth == 0 {
            return (None, evaluate(&self.position), Vec::new(), false);
        }

        // Unwind the search if `stop_flag` was set or time is over
        // Check every 1024 nodes, because it is time-expensive
        if *nodes % 1024 == 0 {
            if stop_flag.load(Ordering::Relaxed) {
                return (None, evaluate(&self.position), Vec::new(), true);
            }

            if let Some(tl) = time_limit {
                if start_time.elapsed() >= tl {
                    return (None, evaluate(&self.position), Vec::new(), true);
                }
            }
        }

        let moves = self.pseudo_moves();
        let mut best_eval = if maximize { i32::MIN } else { i32::MAX };
        let mut best_move = None;
        let mut best_pv = None;
        let mut found_legal_move = false;

        for m in &moves {
            let legal = self.try_to_make_move(m);
            if !legal {
                continue;
            }

            found_legal_move = true;
            let (_best_response, eval, mut child_pv, unwind) = self.minimax_alphabeta(
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
                return (None, best_eval, Vec::new(), true);
            }

            let is_better = if maximize {
                eval > best_eval
            } else {
                eval < best_eval
            };

            if is_better {
                best_eval = eval;
                best_move = Some(m);
                child_pv.push(*m);
                best_pv = Some(child_pv);
            }

            if maximize {
                alpha = max(alpha, eval);
            } else {
                beta = min(beta, eval);
            }

            if beta <= alpha {
                break;
            }
        }

        if !found_legal_move {
            // Checkmate
            if is_king_in_check(&self.position, self.position.player_to_move) {
                // losing sooner is worse
                let eval = match self.position.player_to_move {
                    Player::White => -CHECKMATE_EVAL + depth as i32,
                    Player::Black =>  CHECKMATE_EVAL - depth as i32,
                };
                return (None, eval, Vec::new(), false);
            } else {  // Draw
                return (None, DRAW_EVAL, Vec::new(), false);
            }
        }

        (best_move.copied(), best_eval, best_pv.unwrap(), false)
    }

    // Returns (best_move, best_score, nodes, pv, unwind)
    pub fn find_best_move(
        &mut self,
        depth: usize,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>
    ) -> (Option<Move>, i32, u64, Vec<Move>, bool) {
        let maximize = match self.position.player_to_move {
            Player::White => true,
            Player::Black => false,
        };
        let mut nodes = 0;

        let (best_move, best_eval, pv, unwind) = self.minimax_alphabeta(
            depth,  // NOT depth-1 here! compare the outputs of `go depth 1`
            i32::MIN,
            i32::MAX,
            maximize,
            stop_flag,
            start_time,
            time_limit,
            &mut nodes
        );

        (best_move, best_eval, nodes, pv, unwind)
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
