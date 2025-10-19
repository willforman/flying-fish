use std::sync::atomic::{AtomicU64, Ordering};

use crate::bitboard::Square;
use crate::evaluation::Eval;
use crate::incr;
use crate::position::{Move, Position, ZobristHash};

use strum_macros::FromRepr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr, PartialEq, Eq)]
pub enum EvalType {
    Exact,
    UpperBound,
    LowerBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionTableEntry {
    hash: ZobristHash,
    pub eval: Eval,
    pub best_move: Move,
    pub depth_and_eval_type: u8,
}

impl TranspositionTableEntry {
    fn empty() -> Self {
        Self {
            hash: ZobristHash::empty(),
            best_move: Move::new(Square::A1, Square::A1),
            eval: Eval::DRAW,
            depth_and_eval_type: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.best_move.src == Square::A1 && self.best_move.dest == Square::A1
    }

    const DEPTH_MASK: u8 = 0b00111111;
    const EVAL_TYPE_MASK: u8 = 0b11000000;
    fn build_depth_and_eval_type(depth: u8, eval_type: EvalType) -> u8 {
        debug_assert!(
            depth & Self::EVAL_TYPE_MASK == 0,
            "Depth value is too big: {}",
            depth
        );
        (depth & Self::DEPTH_MASK) | ((eval_type as u8) << 6)
    }

    pub fn depth(&self) -> u8 {
        self.depth_and_eval_type & Self::DEPTH_MASK
    }

    pub fn eval_type(&self) -> EvalType {
        let eval_type_u8 = self.depth_and_eval_type >> 6;
        EvalType::from_repr(eval_type_u8).expect("Unexpected eval type value")
    }
}

#[derive(Debug, Clone)]
pub struct TranspositionTable {
    entries: Box<[TranspositionTableEntry]>,
}

static TT_LOOKUPS: AtomicU64 = AtomicU64::new(0);
static TT_HITS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn get_transposition_table_hitrate() -> f64 {
    let lookups = TT_LOOKUPS.load(Ordering::Relaxed);
    if lookups == 0 {
        return 0.;
    }
    TT_HITS.load(Ordering::Relaxed) as f64 / lookups as f64
}

pub(crate) fn clear_transpostion_table_hitrate() {
    TT_LOOKUPS.store(0, Ordering::Release);
    TT_HITS.store(0, Ordering::Release);
}

impl TranspositionTable {
    /// Creates a transposition table with size ~64mb
    pub fn new() -> Self {
        Self::with_num_entries_power_of_two(22)
    }

    pub fn with_num_entries_power_of_two(power_of_two: usize) -> Self {
        let num_entries = 1 << power_of_two;
        Self {
            entries: vec![TranspositionTableEntry::empty(); num_entries].into_boxed_slice(),
        }
    }

    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            *e = TranspositionTableEntry::empty();
        }
    }

    pub fn get(&self, position: &Position) -> Option<&TranspositionTableEntry> {
        incr!(TT_LOOKUPS);
        let entry = &self.entries[self.index(position)];
        if !entry.is_empty() {
            if entry.hash == position.zobrist_hash {
                incr!(TT_HITS);
                return Some(entry);
            }
        }
        None
    }

    pub fn store(
        &mut self,
        position: &Position,
        eval: Eval,
        eval_type: EvalType,
        best_move: Move,
        depth: u8,
    ) {
        let idx = self.index(position);
        let entry = &self.entries[idx];
        if !entry.is_empty() {
            if entry.depth() > depth {
                return;
            }
        }

        self.entries[idx] = TranspositionTableEntry {
            hash: position.zobrist_hash,
            eval,
            best_move,
            depth_and_eval_type: TranspositionTableEntry::build_depth_and_eval_type(
                depth, eval_type,
            ),
        }
    }

    fn index(&self, position: &Position) -> usize {
        (position.zobrist_hash.value() as usize) & (self.entries.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square::*;
    use test_case::test_case;

    #[test_case(10, EvalType::Exact)]
    #[test_case(63, EvalType::LowerBound)]
    #[test_case(0, EvalType::UpperBound)]
    fn test_depth_and_eval_type(depth: u8, eval_type: EvalType) {
        let depth_and_eval_type =
            TranspositionTableEntry::build_depth_and_eval_type(depth, eval_type);
        let tt_entry = TranspositionTableEntry {
            hash: ZobristHash::empty(),
            eval: Eval::DRAW,
            best_move: Move::new(A1, A1),
            depth_and_eval_type,
        };

        let depth_got = tt_entry.depth();
        let eval_type_got = tt_entry.eval_type();

        assert_eq!(depth_got, depth);
        assert_eq!(eval_type_got, eval_type);
    }
}
