use crate::constants::{AUTHOR, NAME};
use crate::engine::{
    engine::Engine,
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

pub fn ucinewgame(game: &mut Engine) {
    *game = Engine::default();
}

pub fn position(game: &mut Engine, tokens: &[&str]) {
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
            match Engine::from_fen(&fen) {
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
            *game = Engine::default();
            i = 2;
        }
        _ => return,
    }

    if tokens.get(i) == Some(&"moves") {
        for mv in &tokens[i + 1..] {
            let ok = game.try_to_make_uci_move(mv);
            if !ok {
                println!("info string Failed to execute move {}!", mv);
            }
        }
    }
}
