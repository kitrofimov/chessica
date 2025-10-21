use std::{sync::{atomic::AtomicBool, Arc}, time::Duration};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::uci::constants::*;
use crate::{
    engine::{
        base::player::Player,
        search::{searcher::Searcher, perft::*},
        engine::Engine,
        board::game::Game,
    },
    uci::output::*, 
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

fn compute_movetime(game: &mut Game, wtime: usize, btime: usize, winc: usize, binc: usize) -> usize {
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
    engine: &mut Engine,
    tokens: &[&str],
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    let params = parse_go_params(tokens);
    Searcher::stop_search(stop_flag, search_thread);

    if let Some(perft_depth) = params.perft {  // non-UCI compliant
        go_perft(engine, perft_depth, stop_flag, search_thread);
    } else if let Some(movetime) = params.movetime {
        go_movetime(engine, Duration::from_millis(movetime.try_into().unwrap()), stop_flag, search_thread);
    } else if let Some(depth) = params.depth {
        go_depth(engine, depth, stop_flag, search_thread);
    } else if params.infinite {
        go_infinite(engine, stop_flag, search_thread);
    } else if params.wtime.is_some() && params.btime.is_some() {
        let wtime = params.wtime.unwrap();
        let btime = params.btime.unwrap();
        let winc = params.winc.unwrap_or(0);
        let binc = params.binc.unwrap_or(0);
        let ms = compute_movetime(&mut engine.game, wtime, btime, winc, binc);
        println!("{} will search for {} ms", UCI_LOG, ms);
        go_movetime(engine, Duration::from_millis(ms.try_into().unwrap()), stop_flag, search_thread);
    }
}

fn go_perft(engine: &mut Engine, depth: usize, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    let mut game_clone = engine.game.clone();
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
    engine: &mut Engine,
    movetime: Duration,
    stop_flag: &mut Arc<AtomicBool>,
    search_thread: &mut Option<JoinHandle<()>>,
) {
    let mut engine_clone = engine.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = engine_clone.searcher
            .search(&mut engine_clone.game, stop_flag_clone, None, Some(movetime));
        print_best_move(best_move);
    }));
}

fn go_depth(engine: &mut Engine, depth: usize, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    let mut engine_clone = engine.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = engine_clone.searcher
            .search(&mut engine_clone.game, stop_flag_clone, Some(depth), None);
        print_best_move(best_move);
    }));
}

fn go_infinite(engine: &mut Engine, stop_flag: &mut Arc<AtomicBool>, search_thread: &mut Option<JoinHandle<()>>) {
    let mut engine_clone = engine.clone();
    let stop_flag_clone = Arc::clone(stop_flag);

    *search_thread = Some(thread::spawn(move || {
        let best_move = engine_clone.searcher
            .search(&mut engine_clone.game, stop_flag_clone, None, None);
        print_best_move(best_move);
    }));
}
