use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::constants::{NAME, AUTHOR};
use crate::game::Game;
use crate::perft::*;

pub fn uci() {
    println!("id name {}", NAME);
    println!("id author {}", AUTHOR);
    println!("uciok");
}

pub fn isready() {
    println!("readyok");
}

pub fn ucinewgame(game: &mut Game) {
    *game = Game::default();
}

pub fn stop(
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    stop_flag.store(true, Ordering::Relaxed);
    if let Some(handle) = search_thread.take() {
        let _ = handle.join();
    }
}

pub fn position(game: &mut Game, tokens: &[&str]) {
    if tokens.len() < 2 {
        return;
    }

    let i;
    match tokens[1] {
        "fen" if tokens.len() >= 8 => {
            let fen = tokens[2..=7].join(" ");
            *game = Game::from_fen(&fen);
            i = 8;
        }
        "startpos" => {
            *game = Game::default();
            i = 2;
        }
        _ => return,
    }

    if tokens.get(i) == Some(&"moves") {
        for mv in &tokens[i + 1..] {
            game.try_to_make_uci_move(mv);
        }
    }
}

pub fn go(
    game: &mut Game,
    tokens: &[&str],
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    if tokens.len() >= 3 && tokens[1] == "perft" {
        stop(stop_flag, search_thread);

        if let Ok(depth) = tokens[2].parse::<usize>() {
            // TODO: this cloning shouldn't be a huge performance penalty right?
            let mut game_clone = game.clone();
            *stop_flag = Arc::new(AtomicBool::new(false));
            let stop_flag_clone = Arc::clone(stop_flag);

            *search_thread = Some(thread::spawn(move || {
                let start = Instant::now();
                let nodes = perft(&mut game_clone, depth, 0, &stop_flag_clone);
                let duration = start.elapsed();
                let seconds = duration.as_secs_f64();

                if nodes == PERFT_INTERRUPTED {
                    println!("perft interrupted");
                } else {
                    println!("Nodes searched: {}", nodes);
                    println!("Time: {:.3} sec", seconds);
                    println!("Nodes per second: {:.2}", nodes as f64 / seconds);
                }
            }));
        }
    }
}
