use crate::{bitboard::BitBoard, Piece, Square};

use super::traits::GenerateSlidingMoves;

pub struct MagitBitboard;

// def gen_rook() -> [64 * pow(2, 14) * 8; BitBoard] {
const fn gen_rook_moves() -> [BitBoard; 64 * 2_usize.pow(14)] {
    let mut rook_moves = [BitBoard::empty(); 64 * 2_usize.pow(14)];

    let mut sq_idx = 0;
    while sq_idx < 64 {
        let sq = Square::from_repr(sq_idx).unwrap();
        let bb = BitBoard::ray


        sq_idx += 1;
    }
    rook_moves
}

impl GenerateSlidingMoves for MagitBitboard {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        BitBoard::empty()
    }
}
