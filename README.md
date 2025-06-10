# Flying Fish

Minimax chess engine, using [bitboards](https://www.chessprogramming.org/Bitboards) + [hyperbola quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence).

### Performance

Numbers from my M1 macbook:

[Perft](https://www.chessprogramming.org/Perft): measures move generation + move make/unmake (`perft_bench` in `uci` mode)
- No caching, single threaded: 17.3M positions/second

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
- [ ] Time management
- [ ] Quiescence search
- [ ] Zobrist Hashing
- [ ] Transposition Tables
- [ ] Switch to Magic Bitboards
