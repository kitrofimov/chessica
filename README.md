# chess-engine
Hobby-level, bitboard-representation chess engine

TODO list towards a minimal-working prototype:
- [ ] Move generation:
    - [x] Implement castling
    - [x] Discard illegal moves ~~/ generate only legal moves~~
    - [ ] Thoroughly debug all the `perft` tests
        - [x] Implement per-move output (Stockfish-like divide)
        - [ ] `depth = 3`, initial position, wrong node counts for some moves (see #1)
    - [ ] Time the `perft` function. Is it fast enough?
- [ ] Searching the tree:
    - [ ] Naive material evaluation function
    - [ ] Minimax searching algorithm
    - [ ] Alpha-beta pruning
- [ ] Universal Chess Interface (UCI)

## Building

```bash
git clone https://github.com/kitrofimov/chess-engine
cd chess-engine
cargo build
```

## Acknowledgements
- [Magic Move-Bitboard Generation in Computer Chess](http://pradu.us/old/Nov27_2008/Buzz/research/magic/Bitboards.pdf) by Pradyumna Kannan
- [Fast Chess Move Generation With Magic Bitboards](https://rhysre.net/fast-chess-move-generation-with-magic-bitboards.html) by Rhys Rustad‑Elliott
- [Generating Legal Chess Moves Efficiently](https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/) by Peter Ellis Jones
