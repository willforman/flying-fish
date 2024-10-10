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
        let blocker_boards = gen_blocker_boards(sq);
        for (board_idx, &board) in blocker_boards.iter().enumerate() {
            let moves = move_gen.gen_moves(Piece::Rook, sq, board);
            let move_idx = (sq as usize) * 2_usize.pow(14) + board_idx;
            rook_moves[move_idx] = moves;
        }
    }
    rook_moves
}

fn gen_blocker_boards(sq: Square) -> [BitBoard; 2_usize.pow(14)] {
    let mut blocker_boards = [BitBoard::empty(); 2_usize.pow(14)];

    let masks = MASKS_LIST.get(sq);
    let rays = (masks.rank | masks.file) & !masks.bit;
    let ray_squares = rays.to_squares();
    gen_blocker_boards_backtracker(rays, &ray_squares, 0, &mut blocker_boards, &mut 0);
    blocker_boards
}

fn gen_blocker_boards_backtracker(
    mut curr_board: BitBoard,
    squares: &[Square],
    start_sq_idx: usize,
    blocker_boards: &mut [BitBoard; 2_usize.pow(14)],
    blocker_boards_idx: &mut usize,
) {
    blocker_boards[*blocker_boards_idx] = curr_board;
    *blocker_boards_idx += 1;

    for curr_sq_idx in start_sq_idx..squares.len() {
        let sq = squares[curr_sq_idx];
        curr_board.clear_square(sq);

        gen_blocker_boards_backtracker(
            curr_board,
            squares,
            curr_sq_idx + 1,
            blocker_boards,
            blocker_boards_idx,
        );

        curr_board.set_square(sq);
    }
}

impl GenerateSlidingMoves for MagicBitboard {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        BitBoard::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Square::*;
    use super::*;
    use test_case::test_case;

    #[test_case(H4, &[BitBoard::from_squares(&[H1]), BitBoard::from_squares(&[H5, H7, A4, B4, F4])] ; "H4")]
    fn test_gen_blocker_boards(square: Square, boards_want: &[BitBoard]) {
        let blocker_boards = gen_blocker_boards(square);

        for board in boards_want {
            assert!(blocker_boards.contains(board));
        }

        let empty_bb_idxs = blocker_boards
            .into_iter()
            .enumerate()
            .filter(|(_, bb)| bb.is_empty())
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>();

        // Squares has length 14. The first 14 calls will all clear one of the 14 squares,
        // resulting in an empty bitboard by the 15th iteration
        assert_eq!(empty_bb_idxs, vec![14]);
    }
}
