# chess-engine
Hobby-level, bitboard-representation chess engine

TODO list towards a minimal-working prototype:
- [x] Move generation:
    - [x] Implement castling
    - [x] Discard illegal moves ~~/ generate only legal moves~~
    - [x] Thoroughly debug all the `perft` tests
        - [x] Implement per-move output (Stockfish-like divide)
        - [x] Issues #1, #3, #4, #5, #6
        - Move generation seems to be right now!
    - [x] Optimize the `perft` function
        - [x] Calculate nodes per second
            - `--release`: ~23 million nodes per second... enough for now
- [x] Searching the tree:
    - [x] Naive material evaluation function
    - [x] Minimax searching algorithm
    - [x] Alpha-beta pruning
- [ ] Subtleties:
    - [ ] 50 move rule
    - [ ] What does happen when minimax stumbles upon checkmate?
- [ ] Universal Chess Interface (UCI)
    - [ ] Should be handling CLI input on the second thread
    - [ ] Add `go depth X`, `go movetime X`, `go wtime X btime Y winc Z binc W`
    - [ ] Try to hook this up to some GUI

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
