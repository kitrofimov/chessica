use std::time::Instant;
use chess_engine::{game::*, perft::*};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let depth = args.get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .expect("Please provide a valid integer as the first argument.");

    let mut game = Game::default();
    let score = minimax(&mut game, depth, true);
    println!("{}", score);

    // let start = Instant::now();
    // let nodes = perft(&mut game, depth, 0);
    // let duration = start.elapsed();
    // let seconds = duration.as_secs_f64();

    // println!("\nNodes searched: {}", nodes);
    // println!("Time: {:.3} sec", seconds);
    // println!("Nodes per second: {:.2}", nodes as f64 / seconds);
}

fn minimax(game: &mut Game, depth: usize, maximize: bool) -> i32 {
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
        let score = minimax(game, depth - 1, !maximize);
        game.unmake_move();
        best = if maximize { best.max(score) } else { best.min(score) };
    }
    best
}
