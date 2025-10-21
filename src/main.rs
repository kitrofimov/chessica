use std::io::{self, BufRead, Write};
use std::sync::{Arc, atomic::AtomicBool};
use std::thread::JoinHandle;

use chessica::uci::{self, constants::*};
use chessica::engine::{
    engine::Engine,
    search::searcher::Searcher,
};

fn main() {
    let stdin = io::stdin();
    let mut engine = Engine::default();

    let mut stop_flag = Arc::new(AtomicBool::new(false));
    let mut search_thread: Option<JoinHandle<()>> = None;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci"        => uci::uci(),
            "isready"    => uci::isready(),
            "ucinewgame" => uci::ucinewgame(&mut engine),
            "position"   => uci::position(&mut engine, &tokens),
            "go"         => uci::go(&mut engine, &tokens, &mut stop_flag, &mut search_thread),
            "stop"       => Searcher::stop_search(&mut stop_flag, &mut search_thread),
            "quit" => {
                Searcher::stop_search(&mut stop_flag, &mut search_thread);
                break;
            }
            "d" => println!("{}", engine.game.position),
            _   => println!("{} {}", UCI_LOG, UCI_UNKNOWN_COMMAND)
        }

        io::stdout().flush().unwrap();
    }
}
