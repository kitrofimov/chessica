use chess_engine::{game::*, perft::*};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let depth = args.get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .expect("Please provide a valid integer as the first argument.");

    let mut game = Game::default();
    let nodes = perft(&mut game, depth, 0);
    println!("{} nodes (depth = {} half-moves)", nodes, depth);
}
