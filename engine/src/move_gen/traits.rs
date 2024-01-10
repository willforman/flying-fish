use std::collections::HashSet;

use crate::bitboard::{BitBoard, Square};
use crate::position::{Move, Piece, Position, Side};

pub(super) trait GenerateLeapingMoves {
    fn gen_knight_king_moves(&self, piece: Piece, square: Square) -> BitBoard;

    fn gen_pawn_pushes(&self, square: Square, side: Side) -> BitBoard;
    fn gen_pawn_atks(&self, square: Square, side: Side) -> BitBoard;
}

pub(super) trait GenerateSlidingMoves {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard;
}

pub(crate) trait GenerateCheckers {
    fn gen_checkers(&self, position: &Position) -> BitBoard;
}

pub trait GenerateMoves {
    fn gen_moves(&self, position: &Position) -> HashSet<Move>;
}
