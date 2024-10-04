use std::{cell::LazyCell, sync::LazyLock};

use strum::IntoEnumIterator;

use crate::{bitboard::BitBoard, GenerateMoves, Piece, Square, HYPERBOLA_QUINTESSENCE_MOVE_GEN};

use super::{masks::MASKS_LIST, traits::GenerateSlidingMoves};

#[derive(Default)]
pub struct MagicBitboard;

static MAGIC_BITBOARD: LazyLock<MagicBitboard> = LazyLock::new(|| MagicBitboard::default());

fn gen_rook_moves(move_gen: &impl GenerateSlidingMoves) -> [BitBoard; 64 * 2_usize.pow(14)] {
    let mut rook_moves = [BitBoard::empty(); 64 * 2_usize.pow(14)];

    for sq in Square::iter() {
        let blocker_boards = gen_blocker_boards(sq, move_gen);
    }
    rook_moves
}

fn gen_blocker_boards(sq: Square, move_gen: &impl GenerateSlidingMoves) -> [BitBoard; 14] {
    let mut blocker_boards = [BitBoard::empty(); 14];

    let rays = move_gen.gen_moves(Piece::Queen, sq, BitBoard::empty());
    for possible_blocker_square in rays.to_squares() {}
    blocker_boards
}

//fn gen_blocker_boards_backtracker(sq: Square, )
//
impl GenerateSlidingMoves for MagicBitboard {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        BitBoard::empty()
    }
}
