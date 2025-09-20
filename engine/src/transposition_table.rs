use crate::bitboard::Square;
use crate::evaluation::Eval;
use crate::position::{Move, Position, ZobristHash};

#[derive(Debug, Clone, Copy)]
pub enum TranspositionTableScore {
    Exact(Eval),
    UpperBound(Eval),
    LowerBound(Eval),
}

#[derive(Debug, Clone)]
pub struct TranspositionTableEntry {
    hash: ZobristHash,
    pub score: TranspositionTableScore,
    pub best_move: Move,
    pub depth: u8,
}

impl TranspositionTableEntry {
    fn empty() -> Self {
        Self {
            hash: ZobristHash::empty(),
            best_move: Move::new(Square::A1, Square::A1),
            score: TranspositionTableScore::Exact(Eval::Draw),
            depth: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.best_move.src == Square::A1 && self.best_move.dest == Square::A1
    }
}

const TRANSPOSITION_TABLE_ENTRIES: usize = 1 << 12;
const SIZE: usize = std::mem::size_of::<TranspositionTableEntry>();
const SSIZE: usize = std::mem::size_of::<TranspositionTableScore>();
const EVAL_SIZE: usize = std::mem::size_of::<Eval>();
const ZSIZE: usize = std::mem::size_of::<ZobristHash>();
const MOVE: usize = std::mem::size_of::<Move>();
const EVAL_SIZE2: usize = std::mem::size_of::<f64>();

#[derive(Debug, Clone)]
pub struct TranspositionTable {
    entries: [TranspositionTableEntry; TRANSPOSITION_TABLE_ENTRIES],
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            entries: std::array::from_fn(|_| TranspositionTableEntry::empty()),
        }
    }

    pub fn get(&self, position: &Position) -> Option<&TranspositionTableEntry> {
        let entry = &self.entries[self.index(position)];
        if !entry.is_empty() {
            if entry.hash == position.zobrist_hash {
                return Some(entry);
            }
        }
        None
    }

    pub fn store(
        &mut self,
        position: &Position,
        score: TranspositionTableScore,
        best_move: Move,
        depth: u8,
    ) {
        let idx = self.index(position);
        let entry = &self.entries[idx];
        if !entry.is_empty() {
            if entry.depth > depth {
                return;
            }
        }

        self.entries[idx] = TranspositionTableEntry {
            hash: position.zobrist_hash,
            score,
            depth,
            best_move,
        }
    }

    fn index(&self, position: &Position) -> usize {
        (position.zobrist_hash.value() as usize) & (TRANSPOSITION_TABLE_ENTRIES - 1)
    }
}
