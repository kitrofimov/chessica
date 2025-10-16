use std::{sync::{atomic::AtomicBool, Arc}, time::Duration};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::{
    engine::{
        base::player::Player,
        search::perft::*,
        engine::Engine,
    },
    uci::{output::*, search_control::*},
};

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

fn compute_movetime(game: &mut Engine, wtime: usize, btime: usize, winc: usize, binc: usize) -> usize {
    let (time, inc) = if game.position.player_to_move == Player::White {
        (wtime, winc)
    } else {
        (btime, binc)
    };

    let moves_remaining = 30;
    let base_time = time / moves_remaining;
    let inc_bonus = inc * 8 / 10;  // use 80% of the increment

    base_time + inc_bonus
}

pub fn go(
    game: &mut Engine,
    tokens: &[&str],
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    let params = parse_go_params(tokens);
    stop_search(stop_flag, search_thread);

    if let Some(perft_depth) = params.perft {  // non-UCI compliant
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
        println!("info string will search for {} ms", ms);
        go_movetime(game, Duration::from_millis(ms.try_into().unwrap()), stop_flag, search_thread);
    }
}

fn go_perft(game: &mut Engine, depth: usize, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
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

fn go_movetime(
    game: &mut Engine,
    movetime: Duration,
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    let mut game_clone = game.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = iterative_deepening(&mut game_clone, stop_flag_clone, None, Some(movetime));
        print_best_move(best_move);
    }));
}

fn go_depth(game: &mut Engine, depth: usize, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    let mut game_clone = game.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = iterative_deepening(&mut game_clone, stop_flag_clone, Some(depth), None);
        print_best_move(best_move);
    }));
}

fn go_infinite(game: &mut Engine, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    let mut game_clone = game.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = iterative_deepening(&mut game_clone, stop_flag_clone, None, None);
        print_best_move(best_move);
    }));
}
