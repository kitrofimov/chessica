use crate::game::Game;

pub fn perft(game: &mut Game, depth: usize) -> u64 {
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
        nodes += perft(game, depth-1);
        game.unmake_move();
    }
    return nodes;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    #[ignore]
    // https://www.chessprogramming.org/Perft_Results#Initial_Position
    fn perft_initial_0_5() {
        let mut game = Game::default();
        assert_eq!(perft(&mut game, 0), 1);
        assert_eq!(perft(&mut game, 1), 20);
        assert_eq!(perft(&mut game, 2), 400);
        assert_eq!(perft(&mut game, 3), 8_902);
        assert_eq!(perft(&mut game, 4), 197_281);
        assert_eq!(perft(&mut game, 5), 4_865_609);
    }

    #[test]
    #[ignore]
    // https://www.chessprogramming.org/Perft_Results#Position_2
    fn perft_kiwipete_1_5() {
        let mut game = Game::new(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "));
        assert_eq!(perft(&mut game, 1), 48);
        assert_eq!(perft(&mut game, 2), 2_039);
        assert_eq!(perft(&mut game, 3), 97_862);
        assert_eq!(perft(&mut game, 4), 4_085_603);
        assert_eq!(perft(&mut game, 5), 193_690_690);
    }

    #[test]
    #[ignore]
    // https://www.chessprogramming.org/Perft_Results#Position_3
    fn perft_position3_1_5() {
        let mut game = Game::new(Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 "));
        assert_eq!(perft(&mut game, 1), 14);
        assert_eq!(perft(&mut game, 2), 191);
        assert_eq!(perft(&mut game, 3), 2_812);
        assert_eq!(perft(&mut game, 4), 43_238);
        assert_eq!(perft(&mut game, 5), 674_624);
    }

    #[test]
    #[ignore]
    // https://www.chessprogramming.org/Perft_Results#Position_4
    fn perft_position4_1_5() {
        let mut game = Game::new(Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"));
        assert_eq!(perft(&mut game, 1), 6);
        assert_eq!(perft(&mut game, 2), 264);
        assert_eq!(perft(&mut game, 3), 9_467);
        assert_eq!(perft(&mut game, 4), 422_333);
        assert_eq!(perft(&mut game, 5), 15_833_292);
    }

    #[test]
    #[ignore]
    // https://www.chessprogramming.org/Perft_Results#Position_5
    fn perft_position5_1_5() {
        let mut game = Game::new(Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"));
        assert_eq!(perft(&mut game, 1), 44);
        assert_eq!(perft(&mut game, 2), 1_486);
        assert_eq!(perft(&mut game, 3), 62_379);
        assert_eq!(perft(&mut game, 4), 2_103_487);
        assert_eq!(perft(&mut game, 5), 89_941_194);
    }

    #[test]
    #[ignore]
    // https://www.chessprogramming.org/Perft_Results#Position_6
    fn perft_position6_1_5() {
        let mut game = Game::new(Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"));
        assert_eq!(perft(&mut game, 1), 46);
        assert_eq!(perft(&mut game, 2), 2_079);
        assert_eq!(perft(&mut game, 3), 89_890);
        assert_eq!(perft(&mut game, 4), 3_894_594);
        assert_eq!(perft(&mut game, 5), 164_075_551);
    }
}
