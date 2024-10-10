use std::{cell::LazyCell, sync::LazyLock};

use arrayvec::ArrayVec;
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

fn gen_blocker_boards(sq: Square) -> [BitBoard; 2_usize.pow(14)] {
    let mut blocker_boards: ArrayVec<BitBoard, { 2_usize.pow(14) }> = ArrayVec::new();

    let masks = MASKS_LIST.get(sq);
    let rays = (masks.rank | masks.file) & !masks.bit;
    let ray_squares = rays.to_squares();
    gen_blocker_boards_backtracker(rays, &ray_squares, 0, &mut blocker_boards);
    blocker_boards.try_into().unwrap()
}

fn gen_blocker_boards_backtracker(
    mut curr_board: BitBoard,
    squares: &[Square],
    start_sq_idx: usize,
    blocker_boards: &mut ArrayVec<BitBoard, { 2_usize.pow(14) }>,
) {
    if start_sq_idx == blocker_boards.len() {
        return;
    }

    for curr_sq_idx in start_sq_idx..blocker_boards.len() {
        let sq = squares[curr_sq_idx];
        curr_board.clear_square(sq);

        blocker_boards.push(curr_board);
        gen_blocker_boards_backtracker(curr_board, squares, curr_sq_idx + 1, blocker_boards);

        curr_board.set_square(sq);
    }
}

impl GenerateSlidingMoves for MagicBitboard {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        BitBoard::empty()
    }
}
