use arrayvec::ArrayVec;

use crate::bitboard::{BitBoard, Square};
use crate::position::{Move, Piece, Position, Side};

pub(super) trait GenerateSlidingMoves {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard;
}

pub trait GenerateMoves {
    fn gen_moves(&self, position: &Position) -> ArrayVec<Move, 80>;
    fn gen_checkers(&self, position: &Position) -> BitBoard;
}
