use std::{
    cmp::{max, min},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant}
};
use crate::core::{
    chess_move::*,
    evaluate::evaluate,
    movegen::pseudo_moves,
    player::Player,
    position::*,
    rules::{is_king_in_check, make_move}
};

#[derive(Clone)]
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

    pub fn from_fen(fen: &str) -> Result<Game, FenParseError> {
        Ok(Game {
            positions: vec![Position::from_fen(fen)?]
        })
    }

    pub fn position(&self) -> &Position {
        self.positions.last().unwrap()
    }

    pub fn pseudo_moves(&self) -> Vec<Move> {
        pseudo_moves(self.position())
    }

    pub fn try_to_make_move(&mut self, m: &Move) -> bool {
        let pos = self.position();
        let new = make_move(pos, m);

        // Check legality of a move (is player that made the move still in check?)
        // Using `pos.player_to_move` because the flag was already flipped in `new`
        if is_king_in_check(&new, pos.player_to_move) {
            return false;
        }

        self.positions.push(new);
        true
    }

    pub fn unmake_move(&mut self) {
        self.positions.pop();
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

        // Unwind the search if `stop_flag` was set or time is over
        if stop_flag.load(Ordering::Relaxed)
            || time_limit.map(|tl| start_time.elapsed() >= tl).unwrap_or(false) {
            // TODO: is it correct to evaluate the position here?
            return (evaluate(self.position()), true);
        }

        if depth == 0 {
            return (evaluate(self.position()), false);
        }

        let moves = self.pseudo_moves();
        let mut best = if maximize { i32::MIN } else { i32::MAX };
        for m in &moves {
            let legal = self.try_to_make_move(m);
            if !legal {
                continue;
            }
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
                if beta <= alpha {
                    break;
                }
            } else {
                best = min(best, eval);
                beta = min(beta, eval);
                if beta <= alpha {
                    break;
                }
            }
        }
        (best, false)
    }

    pub fn find_best_move(
        &mut self,
        depth: usize,
        stop_flag: &Arc<AtomicBool>,
        start_time: Instant,
        time_limit: Option<Duration>
    ) -> (Move, i32, u64) {
        let mut best_move = None;
        let (mut best_score, maximize) = match self.position().player_to_move {
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
                // TODO: this panicks! see #8
                return (best_move.unwrap(), best_score, nodes);
            }
        }

        (best_move.unwrap(), best_score, nodes)  // TODO: panicking is bad. What if there are no moves?
    }
}
