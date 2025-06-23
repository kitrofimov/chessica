use std::{cmp::{max, min}, i32, time::Instant};
use chess_engine::{game::*, perft::*};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let depth = args.get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .expect("Please provide a valid integer as the first argument.");

    let mut game = Game::default();
    let score = minimax_alphabeta(&mut game, depth, i32::MIN, i32::MAX, true);
    println!("{}", score);

    // let start = Instant::now();
    // let nodes = perft(&mut game, depth, 0);
    // let duration = start.elapsed();
    // let seconds = duration.as_secs_f64();

    // println!("\nNodes searched: {}", nodes);
    // println!("Time: {:.3} sec", seconds);
    // println!("Nodes per second: {:.2}", nodes as f64 / seconds);
}

fn minimax_alphabeta(game: &mut Game, depth: usize, mut alpha: i32, mut beta: i32, maximize: bool) -> i32 {
    if depth == 0 {
        return game.position().evaluate();
    }

    let moves = game.generate_pseudo_moves();

    // TODO: checkmate/stalemate? Should I handle this specifically?
    if moves.is_empty() {
        return game.position().evaluate();
    }

    let mut best = if maximize { i32::MIN } else { i32::MAX };
    for m in &moves {
        let legal = game.try_to_make_move(m);
        if !legal {
            continue;
        }
        let score = minimax_alphabeta(game, depth - 1, alpha, beta, !maximize);
        game.unmake_move();
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
