use std::io::{self, BufRead, Write};
use std::sync::{Arc, atomic::AtomicBool};
use std::thread::JoinHandle;

use chess_engine::{core::game::Game, uci};

fn main() {
    let stdin = io::stdin();
    let mut game = Game::default();

    let mut stop_flag = Arc::new(AtomicBool::new(false));
    let mut search_thread: Option<JoinHandle<()>> = None;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci"        => uci::uci(),
            "isready"    => uci::isready(),
            "ucinewgame" => uci::ucinewgame(&mut game),
            "position"   => uci::position(&mut game, &tokens),
            "go"         => uci::go(&mut game, &tokens, &mut stop_flag, &mut search_thread),
            "stop"       => uci::stop_search(&mut stop_flag, &mut search_thread),
            "quit" => {
                uci::stop_search(&mut stop_flag, &mut search_thread);
                break;
            }
            "d" => println!("{}", game.position),
            _   => println!("info string Unknown command!")
        }

        io::stdout().flush().unwrap();
    }
}
