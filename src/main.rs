use std::io::{self, BufRead, Write};
use std::time::Instant;
use chess_engine::{game::*, perft::*};

const NAME: &str = "chess-engine";
const AUTHOR: &str = "Kirill Trofimov";

fn main() {
    let stdin = io::stdin();
    let mut game = Game::default();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name {}", NAME);
                println!("id author {}", AUTHOR);
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                game = Game::default();
            }
            "position" => {
                if tokens.len() == 1 {
                    continue;
                }

                let mut i = 0;
                let mut moves_loop = false;

                match tokens[1] {
                    "fen" => {
                        let fen = tokens[2..=7].join(" ");
                        game = Game::from_fen(&fen);
                        i = 8;
                        moves_loop = tokens.len() > i;
                    }
                    "startpos" => {
                        game = Game::default();
                        i = 2;
                        moves_loop = tokens.len() > i;
                    }
                    _ => {}
                }

                if moves_loop {
                    while i < tokens.len() {
                        game.try_to_make_uci_move(tokens[i]);
                        i += 1;
                    }
                }
            }
            "go" => {
                match tokens[1] {
                    "perft" => {
                        let depth = tokens[2].parse::<usize>();
                        if depth.is_err() {
                            continue;
                        }

                        let start = Instant::now();
                        let nodes = perft(&mut game, depth.unwrap(), 0);
                        let duration = start.elapsed();
                        let seconds = duration.as_secs_f64();

                        println!("Nodes searched: {}", nodes);
                        println!("Time: {:.3} sec", seconds);
                        println!("Nodes per second: {:.2}", nodes as f64 / seconds);
                    }
                    _ => {}
                }
            }
            "quit" => return,
            "d" => {  // Stockfish-style, non-UCI-compliant board printing
                println!("{:?}", game.position());
            }
            _ => {}
        }

        io::stdout().flush().unwrap();
    }
}
