use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::{constants::{AUTHOR, NAME}, core::{chess_move::Move, position::FenParseError}};
use crate::core::{
    game::Game,
    player::Player,
    perft::*,
};

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

pub fn stop_search(
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    stop_flag.store(true, Ordering::Relaxed);
    if let Some(handle) = search_thread.take() {
        let _ = handle.join();
    }
    stop_flag.store(false, Ordering::Relaxed);
}

pub fn position(game: &mut Game, tokens: &[&str]) {
    if tokens.len() < 2 {
        return;
    }

    let i;
    match tokens[1] {
        "fen" => {
            if tokens.len() < 8 {
                eprintln!("info string Bad FEN! {:?}", FenParseError::BadFieldCount);
                return;
            }
            let fen = tokens[2..=7].join(" ");
            match Game::from_fen(&fen) {
                Ok(parsed) => {
                    *game = parsed;
                    i = 8;
                }
                Err(e) => {
                    eprintln!("info string Bad FEN! {:?}", e);
                    return;
                }
            }
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

#[derive(Debug)]
struct GoParams {
    perft:    Option<usize>,
    movetime: Option<usize>,
    depth:    Option<usize>,
    infinite: bool,
    wtime:    Option<usize>,
    btime:    Option<usize>,
    winc:     Option<usize>,
    binc:     Option<usize>,
}

fn parse_go_params(tokens: &[&str]) -> GoParams {
    let mut params = GoParams {
        perft:    None,
        movetime: None,
        depth:    None,
        infinite: false,
        wtime:    None,
        btime:    None,
        winc:     None,
        binc:     None,
    };

    // Treat `go` as `go infinite`
    if tokens.len() == 1 {
        params.infinite = true;
        return params;
    }

    let mut i = 1;  // skip the "go"
    let parse = |target: &mut Option<usize>, i: &mut usize| {
        if let Some(value) = tokens.get(*i + 1) {
            *target = value.parse().ok();
            *i += 1;
        }
    };

    while i < tokens.len() {
        match tokens[i] {
            "perft"    => parse(&mut params.perft,    &mut i),
            "movetime" => parse(&mut params.movetime, &mut i),
            "depth"    => parse(&mut params.depth,    &mut i),
            "wtime"    => parse(&mut params.wtime,    &mut i),
            "btime"    => parse(&mut params.btime,    &mut i),
            "winc"     => parse(&mut params.winc,     &mut i),
            "binc"     => parse(&mut params.binc,     &mut i),
            "infinite" => params.infinite = true,
            _ => {}
        }
        i += 1;
    }
    params
}

fn compute_movetime(game: &mut Game, wtime: usize, btime: usize, winc: usize, binc: usize) -> usize {
    let (time, inc) = if game.position().player_to_move == Player::White {
        (wtime, winc)
    } else {
        (btime, binc)
    };

    let moves_remaining = 30;  // TODO: make this adaptive!
    let base_time = time / moves_remaining;
    let inc_bonus = inc * 8 / 10;  // 80% of the increment

    base_time + inc_bonus
}

pub fn go(
    game: &mut Game,
    tokens: &[&str],
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    let params = parse_go_params(tokens);

    if let Some(perft_depth) = params.perft {
        go_perft(game, perft_depth, stop_flag, search_thread);
    } else if let Some(movetime) = params.movetime {
        go_movetime(game, Duration::from_millis(movetime.try_into().unwrap()), stop_flag, search_thread);
    } else if let Some(depth) = params.depth {
        go_depth(game, depth, stop_flag, search_thread);
    } else if params.infinite {
        go_infinite(game, stop_flag, search_thread);
    } else if params.wtime.is_some() && params.btime.is_some() {
        let wtime = params.wtime.unwrap();
        let btime = params.btime.unwrap();
        let winc = params.winc.unwrap_or(0);
        let binc = params.binc.unwrap_or(0);
        let ms = compute_movetime(game, wtime, btime, winc, binc);
        go_movetime(game, Duration::from_millis(ms.try_into().unwrap()), stop_flag, search_thread);
    }
}

fn go_perft(game: &mut Game, depth: usize, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    // Stop current search (if there is some)
    stop_search(stop_flag, search_thread);

    // TODO: this cloning shouldn't be a huge performance penalty right?
    // this is also the case for other `go_*` functions
    let mut game_clone = game.clone();
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

fn print_uci_info(depth: usize, eval: i32, nodes: u64, elapsed: Duration) {
    println!(
        "info depth {} score cp {} time {} nodes {} nps {}",
        depth,
        eval,
        elapsed.as_millis(),
        nodes,
        (nodes as f64 / elapsed.as_secs_f64()).round()
    );
}

fn print_best_move(best_move: Option<Move>) {
    if let Some(m) = best_move {
        println!("bestmove {}", m.to_string());
    } else {
        println!("bestmove 0000");
    }
}

fn iterative_deepening(
    game: &mut Game,
    stop_flag: Arc<AtomicBool>,
    max_depth: Option<usize>,
    time_limit: Option<Duration>,
) -> Option<Move>
{
    let mut last_move = None;
    let start = Instant::now();

    for depth in 1.. {
        if let Some(d) = max_depth {
            if depth > d {
                break;
            }
        }

        let depth_start = Instant::now();
        let (m, eval, nodes) = game.find_best_move(
            depth,
            &stop_flag,
            start,
            time_limit
        );
        let elapsed = depth_start.elapsed();

        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        last_move = Some(m);
        print_uci_info(depth, eval, nodes, elapsed);

        if let Some(limit) = time_limit {
            if start.elapsed() >= limit {
                break;
            }
        }
    }

    last_move
}

fn go_movetime(
    game: &mut Game,
    movetime: Duration,
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    stop_search(stop_flag, search_thread);
    let mut game_clone = game.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = iterative_deepening(&mut game_clone, stop_flag_clone, None, Some(movetime));
        print_best_move(best_move);
    }));
}

fn go_depth(game: &mut Game, depth: usize, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    stop_search(stop_flag, search_thread);
    let mut game_clone = game.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = iterative_deepening(&mut game_clone, stop_flag_clone, Some(depth), None);
        print_best_move(best_move);
    }));
}

fn go_infinite(game: &mut Game, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    stop_search(stop_flag, search_thread);
    let mut game_clone = game.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = iterative_deepening(&mut game_clone, stop_flag_clone, None, None);
        print_best_move(best_move);
    }));
}
