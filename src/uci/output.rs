use std::time::Duration;
use crate::constants::evaluation::*;
use crate::engine::base::_move::Move;

// TODO: fix mate encoding and printing
pub fn print_uci_info(depth: usize, eval: i32, nodes: u64, pv: Vec<Move>, elapsed: Duration) {
    let score = if eval.abs() > CHECKMATE_EVAL - 1000 {
        let n_moves = ((CHECKMATE_EVAL - eval.abs()) as f64 / 2.).ceil();
        let mate_in = if eval > 0 { n_moves } else { -n_moves };
        format!("mate {}", mate_in)
    } else {
        format!("cp {}", eval)
    };

    print!(
        "info depth {} score {} time {} nodes {} nps {} pv ",
        depth,
        score,
        elapsed.as_millis(),
        nodes,
        (nodes as f64 / elapsed.as_secs_f64()).round()
    );

    for m in pv.iter().rev() {
        print!("{} ", m);
    }
    println!();
}

pub fn print_best_move(best_move: Option<Move>) {
    if let Some(m) = best_move {
        println!("bestmove {}", m);
    } else {
        println!("bestmove 0000");
    }
}
