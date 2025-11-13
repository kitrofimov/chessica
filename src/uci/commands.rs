use crate::constants::PATH_TO_EVAL_PARAMS;
use crate::uci::constants::*;
use crate::engine::{
    engine::Engine,
    board::game::Game,
    board::position::FenParseError
};

pub fn uci() {
    println!("id name {}", NAME);
    println!("id author {}", AUTHOR);
    println!("uciok");
}

pub fn isready() {
    println!("readyok");
}

pub fn ucinewgame(engine: &mut Engine) {
    *engine = Engine::new(PATH_TO_EVAL_PARAMS);
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
            *engine = Engine::new(PATH_TO_EVAL_PARAMS);
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
