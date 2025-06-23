use crate::game::Game;

pub fn perft(game: &mut Game, depth: usize, n_calls: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = game.generate_pseudo_moves();
    let mut nodes = 0;

    for m in &moves {
        let legal = game.try_to_make_move(m);
        if !legal {
            continue;
        }
        let branches = perft(game, depth-1, n_calls+1);
        nodes += branches;
        game.unmake_move();

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
    use crate::position::Position;

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Initial_Position
    fn perft_initial_0_5() {
        let mut game = Game::default();
        assert_eq!(perft(&mut game, 0, 0), 1);
        assert_eq!(perft(&mut game, 1, 0), 20);
        assert_eq!(perft(&mut game, 2, 0), 400);
        assert_eq!(perft(&mut game, 3, 0), 8_902);
        assert_eq!(perft(&mut game, 4, 0), 197_281);
        assert_eq!(perft(&mut game, 5, 0), 4_865_609);
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_2
    fn perft_kiwipete_1_5() {
        let mut game = Game::new(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"));
        assert_eq!(perft(&mut game, 1, 0), 48);
        assert_eq!(perft(&mut game, 2, 0), 2_039);
        assert_eq!(perft(&mut game, 3, 0), 97_862);
        assert_eq!(perft(&mut game, 4, 0), 4_085_603);
        // assert_eq!(perft(&mut game, 5, 0), 193_690_690);
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_3
    fn perft_position3_1_5() {
        let mut game = Game::new(Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"));
        assert_eq!(perft(&mut game, 1, 0), 14);
        assert_eq!(perft(&mut game, 2, 0), 191);
        assert_eq!(perft(&mut game, 3, 0), 2_812);
        assert_eq!(perft(&mut game, 4, 0), 43_238);
        assert_eq!(perft(&mut game, 5, 0), 674_624);
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_4
    fn perft_position4_1_5() {
        let mut game = Game::new(Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"));
        assert_eq!(perft(&mut game, 1, 0), 6);
        assert_eq!(perft(&mut game, 2, 0), 264);
        assert_eq!(perft(&mut game, 3, 0), 9_467);
        assert_eq!(perft(&mut game, 4, 0), 422_333);
        // assert_eq!(perft(&mut game, 5, 0), 15_833_292);
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_5
    fn perft_position5_1_5() {
        let mut game = Game::new(Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"));
        assert_eq!(perft(&mut game, 1, 0), 44);
        assert_eq!(perft(&mut game, 2, 0), 1_486);
        assert_eq!(perft(&mut game, 3, 0), 62_379);
        assert_eq!(perft(&mut game, 4, 0), 2_103_487);
        // assert_eq!(perft(&mut game, 5, 0), 89_941_194);
    }

    #[test]
    // https://www.chessprogramming.org/Perft_Results#Position_6
    fn perft_position6_1_5() {
        let mut game = Game::new(Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"));
        assert_eq!(perft(&mut game, 1, 0), 46);
        assert_eq!(perft(&mut game, 2, 0), 2_079);
        assert_eq!(perft(&mut game, 3, 0), 89_890);
        assert_eq!(perft(&mut game, 4, 0), 3_894_594);
        // assert_eq!(perft(&mut game, 5, 0), 164_075_551);
    }
}
