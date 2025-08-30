# Flying Fish

Minimax chess engine, using [bitboards](https://www.chessprogramming.org/Bitboards) + [hyperbola quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence).

### Performance

[Perft](https://www.chessprogramming.org/Perft): measures move generation + move make/unmake (`perft_bench` in `uci` mode)
- No caching, single threaded: 24.9M positions/second (AMD Ryzen 9 7900x)

### How to run

To build the UCI binary:

```
cargo build -p cli --release
```

then start by running the binary with no arguments.

### Todo

- [X] Bitboard representation for boards
- [X] Leaping piece move generation
- [X] Sliding piece move generation (hyperbola quintessence)
- [X] Handle edge cases: en passant, pinning, etc
- [X] Basic piece square table evaluation
- [X] Minimax search
- [X] Alpha-beta pruning
- [X] UCI (Universal Chess Interface) support
- [X] Make-unmake over copy-make
- [X] Time management
- [X] Quiescence search
- [ ] Move ordering heuristics
- [ ] Zobrist Hashing
- [ ] Transposition Tables
- [ ] Parallel search (multithreading)
- [ ] Null move pruning
