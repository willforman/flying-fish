# Rust Chess Engine

Features:
- Board representation: [Bitboards](https://www.chessprogramming.org/Bitboards)
- Sliding piece move generation: [Hyperbola Quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence)

### Performance

Numbers from my M1 macbook

[Perft](https://www.chessprogramming.org/Perft): measures move generation + move make/unmake
- No caching, single threaded: 17.3M positions/second

### How to run

To build the UCI binary:

```
cargo build -p uci --release
```

### Todo

- [X] Bitboard representation for boards
- [X] Handle edge cases: en passant, pinning, etc
- [X] Move generation
- [X] Minimax search
- [X] Alpha-beta pruning for minimax search
- [X] UCI (Universal Chess Interface) support
- [ ] Switch to Magic Bitboards
- [ ] Zobrist Hashing
- [ ] Transposition Tables
