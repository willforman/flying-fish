# Flying Fish

Minimax chess engine, using [bitboards](https://www.chessprogramming.org/Bitboards) + [hyperbola quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence).

### Performance

[Perft](https://www.chessprogramming.org/Perft): measures move generation + move make/unmake (`cargo bench -p engine -- perft`)
- No caching, single threaded: 58.0M positions/second (M4 Pro)

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
- [X] Most-Valuable-Victim Least-Valuable-Attacker
- [X] Zobrist Hashing
- [ ] Transposition Tables
- [ ] Other move ordering heuristics
- [ ] Parallel search (multithreading)


### Notes

Tricky move generation bugs I encountered found:
- En passant pinning
- A pawn that can be en passant'd is checking the king
- A piece that is pinned can move to block/capture a checker
