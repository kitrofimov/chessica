# Chessica

<div align="center">

![build_status](https://img.shields.io/github/actions/workflow/status/kitrofimov/chessica/rust.yml)
![license_mit](https://img.shields.io/badge/License-MIT-blue.svg)
![language_rust](https://img.shields.io/badge/language-rust-B7410E)

</div>

> I think, therefore I mate.

Hobby-level, bitboard-representation chess engine that **is in active development stage, hence many features are still missing**.

## Roadmap
- [x] Move generation
- [x] Alpha-beta pruned minimax search
- [x] Zobrist hashing
- [x] (A little bit of) optimization
- [ ] Move ordering
- [ ] Quiescence Search
- [ ] Fix forced mate evaluation
- [ ] Transposition table
- [ ] Better evaluation function
- [ ] Pondering
- [ ] Better time control (adaptive `moves_remaining` in `uci::compute_movetime`)
- [ ] Opening book
- [ ] Endgame database

## Building

```bash
git clone https://github.com/kitrofimov/chessica
cd chessica
cargo build
```

## Acknowledgements
- [Chess Programming Wiki](https://www.chessprogramming.org/)
- Move generation:
    - [Magic Move-Bitboard Generation in Computer Chess](http://pradu.us/old/Nov27_2008/Buzz/research/magic/Bitboards.pdf) by Pradyumna Kannan
    - [Fast Chess Move Generation With Magic Bitboards](https://rhysre.net/fast-chess-move-generation-with-magic-bitboards.html) by Rhys Rustad‑Elliott
    - [Generating Legal Chess Moves Efficiently](https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/) by Peter Ellis Jones
- UCI implementation:
    - [UCI specification](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf) by [DOBRO](https://github.com/DOBRO)
- Zobrist Hashing:
    - [Transposition Tables & Zobrist Keys](https://www.youtube.com/watch?v=QYNRvMolN20)
    - [Zobrist Hashing](https://dev.to/larswaechter/zobrist-hashing-72n) by Lars Wächter
    - [Transposition Table](https://www.chessprogramming.org/Transposition_Table) on CPW
    - Articles on [Wikipedia](https://en.wikipedia.org/wiki/Zobrist_hashing) and [CPW](https://www.chessprogramming.org/Zobrist_Hashing)
    - [How to implement Zobrist tables?](https://chess.stackexchange.com/questions/42708/how-to-implement-zobrist-tables) on Chess StackExchange
    - [Why is the initial state of Zobrist hashing random?](https://cs.stackexchange.com/questions/22033/why-is-the-initial-state-of-zobrist-hashing-random) on CS StackExchange
