use arrayvec::ArrayVec;

use crate::bitboard::BitBoard;
use crate::position::{Move, Position};

pub trait GenerateMoves {
    fn gen_moves(&self, position: &Position) -> ArrayVec<Move, 218>;
    fn gen_checkers(&self, position: &Position) -> BitBoard;
}
