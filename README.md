# Rust Chess Engine

Features:
- Board representation: [Bitboards](https://www.chessprogramming.org/Bitboards)
- Sliding piece move generation: [Hyperbola Quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence)

Can generate ~2 million positions / second (on my machine).

Todo:
- [X] Bitboard representation for boards
- [X] Handle edge cases: en passant, pinning, etc
- [X] Move generation
- [X] Minimax search
- [X] Alpha-beta pruning for minimax search
- [X] UCI (Universal Chess Interface) support
- [ ] Switch to Magic Bitboards
- [ ] Zobrist Hashing
- [ ] Transposition Tables
