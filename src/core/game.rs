use std::{cmp::{max, min}, sync::{atomic::{AtomicBool, Ordering}, Arc}};
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

    pub fn from_fen(fen: &str) -> Game {
        Game {
            positions: vec![Position::from_fen(fen)]
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
        let pos = *self.positions.last().unwrap();
        let new = make_move(&pos, m);

        // Check legality of a move (is player that made the move still in check?)
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
        let moves = self.generate_pseudo_moves();
        for m in &moves {
            if m.to_string() == uci {
                self.try_to_make_move(m);
                return true;
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
        nodes: &mut u64
    ) -> (i32, bool) {
        *nodes += 1;

        if stop_flag.load(Ordering::Relaxed) {
            return (evaluate(self.position()), true);
        }

        if depth == 0 {
            return (evaluate(self.position()), false);
        }

        let moves = self.generate_pseudo_moves();

        // TODO: checkmate/stalemate? Should I handle this specifically?
        if moves.is_empty() {
            return (evaluate(self.position()), false);
        }

        let mut best = if maximize { i32::MIN } else { i32::MAX };
        for m in &moves {
            let legal = self.try_to_make_move(m);
            if !legal {
                continue;
            }
            let (eval, unwind) = self.minimax_alphabeta(depth - 1, alpha, beta, !maximize, stop_flag, nodes);
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

    pub fn find_best_move(&mut self, depth: usize, stop_flag: &Arc<AtomicBool>) -> (Move, i32, u64) {
        let mut best_move = None;
        let (mut best_score, maximize) = match self.position().player_to_move {
            Player::White => (i32::MIN, true),
            Player::Black => (i32::MAX, false),
        };

        let mut nodes = 0;

        for m in self.generate_pseudo_moves() {
            let legal = self.try_to_make_move(&m);
            if !legal {
                continue;
            }
            let (eval, _) = self.minimax_alphabeta(depth - 1, i32::MIN, i32::MAX, maximize, stop_flag, &mut nodes);
            self.unmake_move();

            if (maximize && eval > best_score) || (!maximize && eval < best_score) {
                best_score = eval;
                best_move = Some(m);
            }
        }

        (best_move.unwrap(), best_score, nodes)  // TODO: panicking is bad. What if there are no moves?
    }
}
