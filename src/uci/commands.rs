use crate::uci::constants::*;
use crate::constants::*;
use crate::engine::{
    engine::Engine,
    board::game::Game,
    board::position::FenParseError,
    search::{evaluate::*, transposition_table::*},
};

pub fn uci() {
    println!("id name {}", NAME);
    println!("id author {}", AUTHOR);
    // Make sure they are parsed in `setoption`
    println!("option name Hash type spin default {} min 1 max 1024", DEFAULT_HASH_MB_SIZE);
    println!("option name EvalParamsJSON type string");
    println!("uciok");
}

pub fn isready() {
    println!("readyok");
}

pub fn ucinewgame(engine: &mut Engine) {
    *engine = Engine::default();
}

pub fn position(engine: &mut Engine, tokens: &[&str]) {
    if tokens.len() < 2 {
        return;
    }

    let i;
    match tokens[1] {
        "fen" => {
            if tokens.len() < 8 {
                eprintln!("{} {} {:?}", UCI_LOG, UCI_BAD_FEN, FenParseError::BadFieldCount);
                return;
            }
            let fen = tokens[2..=7].join(" ");
            match Game::from_fen(&fen) {
                Ok(parsed) => {
                    engine.game = parsed;
                    i = 8;
                }
                Err(e) => {
                    eprintln!("{} {} {:?}", UCI_LOG, UCI_BAD_FEN, e);
                    return;
                }
            }
        }
        "startpos" => {
            *engine = Engine::default();
            i = 2;
        }
        _ => return,
    }

    if tokens.get(i) == Some(&"moves") {
        for mv in &tokens[i + 1..] {
            let ok = engine.game.try_to_make_uci_move(mv);
            if !ok {
                println!("{} Failed to execute move {}!", UCI_LOG, mv);
            }
        }
    }
}

// Limitation: only one-word option names
pub fn setoption(engine: &mut Engine, tokens: &[&str]) {
    if tokens.len() < 5 || tokens[1] != "name" || tokens[3] != "value" {
        eprintln!("{} Bad \"setoption\" command!", UCI_LOG);
        return;
    }

    let name = tokens[2];
    let value = tokens[4..].join(" ");

    // Make sure they are listed in `uci`
    match name {
        "Hash" => {
            if let Ok(size_mb) = value.parse::<usize>() {
                engine.searcher.transposition_table = TranspositionTable::new(size_mb);
            }
        },
        "EvalParamsJSON" => {
            engine.searcher.eval_params = match EvalParams::load(&value) {
                Ok(params) => params,
                Err(e) => {
                    eprintln!("{} Failed to parse the provided JSON file!: {}", UCI_LOG, e);
                    return;
                }
            };
        },
        _ => {
            eprintln!("{} Unknown option name: {}", UCI_LOG, name);
        }
    }
}
