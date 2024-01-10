use std::borrow::BorrowMut;
use std::string::ToString;

use crate::bitboard::{BitBoard, Direction, Square};
use crate::position::{Piece, Side};

use super::traits::GenerateLeapingMoves;

struct SquareToMoveDatabase([BitBoard; 64]);

impl SquareToMoveDatabase {
    const fn get_bitboard(&self, square: Square) -> BitBoard {
        self.0[square as usize]
    }

    const fn get_bitboard_mut(&mut self, square: Square) -> &mut BitBoard {
        &mut self.0[square as usize]
    }
}

struct ColoredSquareToMoveDatabase {
    white: SquareToMoveDatabase,
    black: SquareToMoveDatabase,
}

impl ColoredSquareToMoveDatabase {
    fn get_square_db(&self, side: Side) -> &SquareToMoveDatabase {
        match side {
            Side::White => &self.white,
            Side::Black => &self.black,
        }
    }
}

#[derive(Clone, Copy)]
pub struct LeapingPiecesMoveGen;

impl GenerateLeapingMoves for LeapingPiecesMoveGen {
    fn gen_knight_king_moves(&self, piece: Piece, square: Square) -> BitBoard {
        match piece {
            Piece::Knight => KNIGHT_ATKS.get_bitboard(square),
            Piece::King => KING_ATKS.get_bitboard(square),
            _ => panic!("piece type: want [knight, king], got {}", piece.to_string()),
        }
    }

    fn gen_pawn_pushes(&self, square: Square, side: Side) -> BitBoard {
        PAWN_PUSHES.get_square_db(side).get_bitboard(square)
    }

    fn gen_pawn_atks(&self, square: Square, side: Side) -> BitBoard {
        PAWN_ATKS.get_square_db(side).get_bitboard(square)
    }
}

const fn calc_square_to_move_database(dirs: &[&[Direction]]) -> SquareToMoveDatabase {
    let mut bbs = [BitBoard::empty(); 64];

    let mut bb_idx = 0;

    while bb_idx < bbs.len() {
        let sq = Square::from_u8(bb_idx as u8);

        let mut sq_bb = BitBoard::empty();

        let mut dirs_idx = 0;
        while dirs_idx < dirs.len() {
            let dirs = dirs[dirs_idx];

            let mut dir_sq_bb = BitBoard::from_square(sq);
            let mut curr_dir_idx = 0;

            while curr_dir_idx < dirs.len() {
                let curr_dir = dirs[curr_dir_idx];
                dir_sq_bb.shift(curr_dir);
                // If a shift goes out of bounds, then we break early
                if dir_sq_bb.is_empty() {
                    break;
                }
                curr_dir_idx += 1;
            }
            sq_bb = sq_bb.const_bit_or(dir_sq_bb);
            dirs_idx += 1;
        }
        bbs[bb_idx] = sq_bb;
        bb_idx += 1;
    }

    SquareToMoveDatabase(bbs)
}

const fn calc_pawn_pushes_square_to_move_databases() -> ColoredSquareToMoveDatabase {
    // Adds double pushes for the 2nd rank for white and the 7th for black
    let mut moves = ColoredSquareToMoveDatabase {
        white: calc_square_to_move_database(&[&[Direction::IncRank]]),
        black: calc_square_to_move_database(&[&[Direction::DecRank]]),
    };

    let mut white_idx = 8;
    const WHITE_END_IDX: usize = 16;
    const WHITE_SHIFT_DIR: Direction = Direction::IncRank;
    while white_idx < WHITE_END_IDX {
        let sq = Square::from_repr(white_idx as u8).unwrap();

        let mut double_shift_bb = moves.white.get_bitboard(sq);
        double_shift_bb.shift(WHITE_SHIFT_DIR);

        let bb: &mut BitBoard = moves.white.get_bitboard_mut(sq);
        bb.const_bit_or_mut(double_shift_bb);

        white_idx += 1;
    }

    let mut black_idx = 48;
    const BLACK_END_IDX: usize = 56;
    const BLACK_SHIFT_DIR: Direction = Direction::DecRank;
    while black_idx < BLACK_END_IDX {
        let sq = Square::from_repr(black_idx as u8).unwrap();

        let mut double_shift_bb = moves.black.get_bitboard(sq);
        double_shift_bb.shift(BLACK_SHIFT_DIR);

        let bb: &mut BitBoard = moves.black.get_bitboard_mut(sq);
        bb.const_bit_or_mut(double_shift_bb);

        black_idx += 1;
    }

    moves
}

static PAWN_PUSHES: ColoredSquareToMoveDatabase = calc_pawn_pushes_square_to_move_databases();

static PAWN_ATKS: ColoredSquareToMoveDatabase = ColoredSquareToMoveDatabase {
    white: calc_square_to_move_database(&[
        &[Direction::IncRank, Direction::IncFile],
        &[Direction::IncRank, Direction::DecFile],
    ]),
    black: calc_square_to_move_database(&[
        &[Direction::DecRank, Direction::IncFile],
        &[Direction::DecRank, Direction::DecFile],
    ]),
};

static KNIGHT_ATKS: SquareToMoveDatabase = calc_square_to_move_database(&[
    &[Direction::IncRank, Direction::IncRank, Direction::IncFile],
    &[Direction::IncRank, Direction::IncRank, Direction::DecFile],
    &[Direction::DecRank, Direction::DecRank, Direction::IncFile],
    &[Direction::DecRank, Direction::DecRank, Direction::DecFile],
    &[Direction::IncRank, Direction::IncFile, Direction::IncFile],
    &[Direction::IncRank, Direction::DecFile, Direction::DecFile],
    &[Direction::DecRank, Direction::IncFile, Direction::IncFile],
    &[Direction::DecRank, Direction::DecFile, Direction::DecFile],
]);

static KING_ATKS: SquareToMoveDatabase = calc_square_to_move_database(&[
    &[Direction::IncRank],
    &[Direction::IncFile],
    &[Direction::DecFile],
    &[Direction::DecRank],
    &[Direction::IncRank, Direction::IncFile],
    &[Direction::IncRank, Direction::DecFile],
    &[Direction::DecRank, Direction::IncFile],
    &[Direction::DecRank, Direction::DecFile],
]);

pub(crate) static LEAPING_PIECES: LeapingPiecesMoveGen = LeapingPiecesMoveGen {};

#[cfg(test)]
mod tests {
    use super::Square::*;
    use super::*;
    use test_case::test_case;

    #[test_case(D4, BitBoard::from_squares(&[B5, C6, E6, F5, B3, C2, E2, F3]) ; "center")]
    #[test_case(A8, BitBoard::from_squares(&[B6, C7]) ; "corner")]
    #[test_case(A4, BitBoard::from_squares(&[B6, C5, C3, B2]) ; "edge")]
    fn test_calc_knight_atks(square: Square, want: BitBoard) {
        let sq_got = KNIGHT_ATKS.get_bitboard(square);
        assert_eq!(sq_got, want);
    }

    #[test_case(D4, BitBoard::from_squares(&[C5, D5, E5, C4, E4, C3, D3, E3]) ; "center")]
    #[test_case(A8, BitBoard::from_squares(&[A7, B7, B8]) ; "corner")]
    #[test_case(C1, BitBoard::from_squares(&[B1, B2, C2, D2, D1]) ; "edge")]
    fn test_calc_king_atks(square: Square, want: BitBoard) {
        let sq_got = KING_ATKS.get_bitboard(square);
        assert_eq!(sq_got, want);
    }

    #[test_case(D2, Side::White, BitBoard::from_squares(&[D3, D4]) ; "white double")]
    #[test_case(B3, Side::White, BitBoard::from_squares(&[B4]) ; "white single")]
    #[test_case(G7, Side::White, BitBoard::from_squares(&[G8]) ; "white single edge")]
    #[test_case(G8, Side::White, BitBoard::from_squares(&[]) ; "white edge")]
    #[test_case(D7, Side::Black, BitBoard::from_squares(&[D6, D5]) ; "black double")]
    #[test_case(B6, Side::Black, BitBoard::from_squares(&[B5]) ; "black single")]
    #[test_case(G2, Side::Black, BitBoard::from_squares(&[G1]) ; "black single edge")]
    #[test_case(G1, Side::Black, BitBoard::from_squares(&[]) ; "black edge")]
    fn test_calc_pawn_pushes(square: Square, side: Side, want: BitBoard) {
        let sq_got = PAWN_PUSHES.get_square_db(side).get_bitboard(square);
        assert_eq!(sq_got, want);
    }

    #[test_case(D2, Side::White, BitBoard::from_squares(&[C3, E3]) ; "white")]
    #[test_case(A7, Side::White, BitBoard::from_squares(&[B8]) ; "white edge")]
    // Even though a pawn would never actually be in the "back rank", important for
    // finding checkers if the king is on the back rank
    #[test_case(F1, Side::White, BitBoard::from_squares(&[E2, G2]) ; "white back rank")]
    #[test_case(D7, Side::Black, BitBoard::from_squares(&[C6, E6]) ; "black")]
    #[test_case(A2, Side::Black, BitBoard::from_squares(&[B1]) ; "black edge")]
    #[test_case(F8, Side::Black, BitBoard::from_squares(&[E7, G7]) ; "black back rank")]
    fn test_calc_pawn_atks(square: Square, side: Side, want: BitBoard) {
        let sq_got = PAWN_ATKS.get_square_db(side).get_bitboard(square);
        assert_eq!(sq_got, want);
    }
}
