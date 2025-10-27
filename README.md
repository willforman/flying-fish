# Flying Fish

Alpha-beta pruning chess engine, using [bitboards](https://www.chessprogramming.org/Bitboards) + [hyperbola quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence). Currently ~1500 ELO (relative to [CCRL 40/15](https://computerchess.org.uk/ccrl/4040/rating_list_all.html)).

### Performance

[Perft](https://www.chessprogramming.org/Perft): measures move generation + move make/unmake (`cargo bench -p engine -- perft`)
- Non-bulk, single-threaded: 55.6M positions/second (M4 Pro)

### Features

#### Board representation + move generation

- [Bitboards](https://www.chessprogramming.org/Bitboards)
- [Hyperbola quintessence](https://www.chessprogramming.org/Hyperbola_Quintessence)
- [Zobrist hashing](https://www.chessprogramming.org/Zobrist_Hashing)

#### Search

- [Minimax](https://www.chessprogramming.org/Minimax)
- [Alpha-beta pruning](https://www.chessprogramming.org/Alpha-Beta)
- [Iterative deepening](https://www.chessprogramming.org/Iterative_Deepening)
- [Quiescence search](https://www.chessprogramming.org/Quiescence_Search)
- [Transposition table](https://www.chessprogramming.org/Transposition_Table)
- Basic move ordering: [transposition table move](https://www.chessprogramming.org/Hash_Move) and [MVV-LVA](https://www.chessprogramming.org/MVV-LVA)
- [Principal variation search](https://www.chessprogramming.org/Principal_Variation_Search)
- [Null move pruning](https://www.chessprogramming.org/Null_Move_Pruning)

#### Evaluation

- Pieces value
- [Piece square table](https://www.chessprogramming.org/Piece-Square_Tables)
- [Tapered eval](https://www.chessprogramming.org/Tapered_Eval)


### How to run

To build the UCI binary:

```
cargo build -p cli --release
```

then start by running the binary with no arguments.


### Notes

Tricky move generation bugs I encountered found:
- En passant pinning
- A pawn that can be en passant'd is checking the king
- A piece that is pinned can move to block/capture a checker
