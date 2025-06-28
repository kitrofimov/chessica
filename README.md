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
- [ ] Subtleties & polishing:
    - [x] 50 move rule
        - [x] Move `halfmove_clock` inside `Game`
    - [x] Threefold repetition
        - [x] Zobrist hashing
            - [x] Naive
            - [x] Incremental updating when calling `make_move`
        - [x] Repetition table
    - [x] Insufficient material
    - [ ] Why `perft` became so slow? Did the cloning of `Game`s become too expensive?
    - [ ] What does happen when minimax stumbles upon checkmate or stalemate?
    - [ ] Forced mate evaluation?
    - [ ] Do all the TODO comments
    - [ ] Come up with a cool name and rename the project
- [x] Universal Chess Interface (UCI)
    - [x] Should be handling CLI input on the second thread
    - [x] Add `go depth X`, `go movetime X`, `go wtime X btime Y winc Z binc W`
    - [x] Try to hook this up to some GUI
        - Seems to work! [Here](https://pastebin.com/bDw9PsFe) is the wonderful game we played - the engine seems to go crazy with the evaluation after some time...
        - [One more game](https://pastebin.com/9nXVNefR) they played against each other - draw by repetition. Still the 2 billion bug
        - [x] Evaluation = `2147483647` (2 billion)?
    - [x] Fix `go movetime X` being not-as-correct
- [x] Refactoring!!!
    - [x] Separation of concepts!!! Why do I have `Position` and `Game`? Should `perft` be inside `Game`? `go_*` functions in the `uci` module seem messy, there are two search functions... why?
    - 26 Jun 2025:
        - Moved everything important into `core`
        - Split `constants` into multiple modules, created square indices' constants & replaced magic numbers with them
        - Refactor `movegen` functions
        - Extracted most of structs from `position.rs` into new modules (e.g. `Piece`, `Move`, `Player`)
        - Make `Position` a *pure struct*: only data, no knowledge about rules / move making / etc. => extracted `rules.rs` for this purpose. Refactored `make_move`
        - Implement FEN validation
        - A ton of other small tweaks: comments & etc.

## Building

```bash
git clone https://github.com/kitrofimov/chess-engine
cd chess-engine
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
