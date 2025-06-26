use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use crate::core::game::Game;

// Is equal to 18_446_744_073_709_551_615 (roughly 18 quintillion = 18 * 10^18)
// Large enough to assume it is never going to arise naturally, because if so,
// one should wait ~19500 years while counting 30 million nodes per second
pub const PERFT_INTERRUPTED: u64 = u64::MAX;

pub fn perft(game: &mut Game, depth: usize, n_calls: usize, stop_flag: &Arc<AtomicBool>) -> u64 {
    if stop_flag.load(Ordering::Relaxed) {
        return PERFT_INTERRUPTED;
    }

    if depth == 0 {
        return 1;
    }

    let moves = game.pseudo_moves();
    let mut nodes = 0;

    for m in &moves {
        let legal = game.try_to_make_move(m);
        if !legal {
            continue;
        }
        let branches = perft(game, depth-1, n_calls+1, stop_flag);
        game.unmake_move();

        if branches == PERFT_INTERRUPTED {
            return PERFT_INTERRUPTED;
        }

        nodes += branches;

        // If we're on the top, print detailed data for every depth-1 call
        if n_calls == 0 {
            println!("{} {}", m.to_string(), branches);
        }
    }
    return nodes;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::position::FenParseError;

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Initial_Position
    fn perft_initial_0_5() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut game = Game::default();
        assert_eq!(perft(&mut game, 0, 0, &stop_flag), 1);
        assert_eq!(perft(&mut game, 1, 0, &stop_flag), 20);
        assert_eq!(perft(&mut game, 2, 0, &stop_flag), 400);
        assert_eq!(perft(&mut game, 3, 0, &stop_flag), 8_902);
        assert_eq!(perft(&mut game, 4, 0, &stop_flag), 197_281);
        assert_eq!(perft(&mut game, 5, 0, &stop_flag), 4_865_609);
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_2
    fn perft_kiwipete_1_5() -> Result<(), FenParseError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut game = Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")?;
        assert_eq!(perft(&mut game, 1, 0, &stop_flag), 48);
        assert_eq!(perft(&mut game, 2, 0, &stop_flag), 2_039);
        assert_eq!(perft(&mut game, 3, 0, &stop_flag), 97_862);
        assert_eq!(perft(&mut game, 4, 0, &stop_flag), 4_085_603);
        // assert_eq!(perft(&mut game, 5, 0), 193_690_690);
        Ok(())
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_3
    fn perft_position3_1_5() -> Result<(), FenParseError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut game = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1")?;
        assert_eq!(perft(&mut game, 1, 0, &stop_flag), 14);
        assert_eq!(perft(&mut game, 2, 0, &stop_flag), 191);
        assert_eq!(perft(&mut game, 3, 0, &stop_flag), 2_812);
        assert_eq!(perft(&mut game, 4, 0, &stop_flag), 43_238);
        assert_eq!(perft(&mut game, 5, 0, &stop_flag), 674_624);
        Ok(())
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_4
    fn perft_position4_1_5() -> Result<(), FenParseError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut game = Game::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")?;
        assert_eq!(perft(&mut game, 1, 0, &stop_flag), 6);
        assert_eq!(perft(&mut game, 2, 0, &stop_flag), 264);
        assert_eq!(perft(&mut game, 3, 0, &stop_flag), 9_467);
        assert_eq!(perft(&mut game, 4, 0, &stop_flag), 422_333);
        // assert_eq!(perft(&mut game, 5, 0), 15_833_292);
        Ok(())
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_5
    fn perft_position5_1_5() -> Result<(), FenParseError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut game = Game::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")?;
        assert_eq!(perft(&mut game, 1, 0, &stop_flag), 44);
        assert_eq!(perft(&mut game, 2, 0, &stop_flag), 1_486);
        assert_eq!(perft(&mut game, 3, 0, &stop_flag), 62_379);
        assert_eq!(perft(&mut game, 4, 0, &stop_flag), 2_103_487);
        // assert_eq!(perft(&mut game, 5, 0), 89_941_194);
        Ok(())
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_6
    fn perft_position6_1_5() -> Result<(), FenParseError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut game = Game::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")?;
        assert_eq!(perft(&mut game, 1, 0, &stop_flag), 46);
        assert_eq!(perft(&mut game, 2, 0, &stop_flag), 2_079);
        assert_eq!(perft(&mut game, 3, 0, &stop_flag), 89_890);
        assert_eq!(perft(&mut game, 4, 0, &stop_flag), 3_894_594);
        // assert_eq!(perft(&mut game, 5, 0), 164_075_551);
        Ok(())
    }
}
