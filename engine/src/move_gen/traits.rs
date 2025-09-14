use arrayvec::ArrayVec;

use crate::bitboard::{BitBoard, Square};
use crate::position::{Move, Piece, Position, Side};

pub trait GenerateMoves {
    fn gen_moves(&self, position: &Position) -> ArrayVec<Move, 80>;
    fn gen_checkers(&self, position: &Position) -> BitBoard;
}
