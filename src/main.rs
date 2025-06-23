use std::{i32, time::Instant};
use chess_engine::{game::*, perft::*};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let depth = args.get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .expect("Please provide a valid integer as the first argument.");

    let mut game = Game::default();
    println!("{}", game.find_best_move(depth).to_string());

    // let start = Instant::now();
    // let nodes = perft(&mut game, depth, 0);
    // let duration = start.elapsed();
    // let seconds = duration.as_secs_f64();

    // println!("\nNodes searched: {}", nodes);
    // println!("Time: {:.3} sec", seconds);
    // println!("Nodes per second: {:.2}", nodes as f64 / seconds);
}

