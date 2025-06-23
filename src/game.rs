use std::cmp::{max, min};
use crate::{movegen::pseudo_moves, position::{Move, Player, Position}};

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

    // UTTERLY INSANE IMPLEMENTATION that works
    // this does not need to be *really* fast (called rarely), it's fast *enough*
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

    fn minimax_alphabeta(&mut self, depth: usize, mut alpha: i32, mut beta: i32, maximize: bool) -> i32 {
        if depth == 0 {
            return self.position().evaluate();
        }

        let moves = self.generate_pseudo_moves();

        // TODO: checkmate/stalemate? Should I handle this specifically?
        if moves.is_empty() {
            return self.position().evaluate();
        }

        let mut best = if maximize { i32::MIN } else { i32::MAX };
        for m in &moves {
            let legal = self.try_to_make_move(m);
            if !legal {
                continue;
            }
            let score = self.minimax_alphabeta(depth - 1, alpha, beta, !maximize);
            self.unmake_move();
            if maximize {
                best = max(best, score);
                alpha = max(alpha, score);
                if beta <= alpha {
                    break;
                }
            } else {
                best = min(best, score);
                beta = min(beta, score);
                if beta <= alpha {
                    break;
                }
            }
        }
        best
    }

    pub fn find_best_move(&mut self, depth: usize) -> Move {
        let mut best_score = i32::MIN;
        let mut best_move = None;
        let maximize = match self.position().player_to_move {
            Player::White => true,
            Player::Black => false,
        };
        for m in self.generate_pseudo_moves() {
            let legal = self.try_to_make_move(&m);
            if !legal {
                continue;
            }
            let score = self.minimax_alphabeta(depth - 1, i32::MIN, i32::MAX, maximize);
            self.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
        }
        best_move.unwrap()  // TODO: panicking is bad. What if there are no moves?
    }
}
