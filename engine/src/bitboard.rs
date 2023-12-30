use std::fmt;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, FromRepr};

#[allow(dead_code)]
#[rustfmt::skip]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, EnumString, FromRepr, Display, PartialOrd, Ord, Hash)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    pub(crate) const fn abs_diff(self, other: Square) -> u8 {
        (self as u8).abs_diff(other as u8)
    }

    pub(crate) const fn to_rank_file(self) -> (u8, u8) {
        (self as u8 / 8, self as u8 % 8)
    }

    pub(crate) const fn from_u8(idx: u8) -> Square {
        match Square::from_repr(idx) {
            Some(sq) => sq,
            None => panic!("square out of bounds"),
        }
    }

    #[rustfmt::skip]
    pub const fn list_white_perspective() -> [Square; 64] {
        [
            Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
            Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
            Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
            Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
            Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
            Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
            Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
            Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
        ]
    }

    #[rustfmt::skip]
    pub const fn list_black_perspective() -> [Square; 64] {
        [
            Square::H1, Square::G1, Square::F1, Square::E1, Square::D1, Square::C1, Square::B1, Square::A1,
            Square::H2, Square::G2, Square::F2, Square::E2, Square::D2, Square::C2, Square::B2, Square::A2,
            Square::H3, Square::G3, Square::F3, Square::E3, Square::D3, Square::C3, Square::B3, Square::A3,
            Square::H4, Square::G4, Square::F4, Square::E4, Square::D4, Square::C4, Square::B4, Square::A4,
            Square::H5, Square::G5, Square::F5, Square::E5, Square::D5, Square::C5, Square::B5, Square::A5,
            Square::H6, Square::G6, Square::F6, Square::E6, Square::D6, Square::C6, Square::B6, Square::A6,
            Square::H7, Square::G7, Square::F7, Square::E7, Square::D7, Square::C7, Square::B7, Square::A7,
            Square::H8, Square::G8, Square::F8, Square::E8, Square::D8, Square::C8, Square::B8, Square::A8,
        ]
    }
}

#[repr(isize)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Direction {
    IncRank = 8,
    IncFile = 1,
    DecRank = -8,
    DecFile = -1,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct BitBoard(u64);

impl BitBoard {
    pub(crate) const fn empty() -> Self {
        BitBoard(0)
    }

    pub(crate) const fn full() -> Self {
        BitBoard(u64::max_value())
    }

    pub(crate) const fn from_square(square: Square) -> Self {
        BitBoard(1 << (square as u8))
    }

    // TODO: convert to From<&[Square]>
    pub(crate) fn from_squares(squares: &[Square]) -> Self {
        BitBoard(squares.iter().fold(0, |board, sq| board | 1 << (*sq as u8)))
    }

    pub(crate) const fn from_val(val: u64) -> Self {
        BitBoard(val)
    }

    pub(crate) fn from_square_shifts(
        square: Square,
        shift_dirs_list: &Vec<Vec<Direction>>,
    ) -> Self {
        let start = BitBoard::from_square(square);
        let res = shift_dirs_list
            .iter()
            .fold(start.clone(), |acc, shift_dirs| {
                let mut shifted = start.clone();
                for &sd in shift_dirs {
                    shifted = shifted.shift(sd);
                }
                acc | shifted
            });
        res & !start
    }

    pub(crate) const fn from_ray_excl(sq1: Square, sq2: Square) -> Self {
        let (sq1_rank, sq1_file) = sq1.to_rank_file();
        let (sq2_rank, sq2_file) = sq2.to_rank_file();

        let dirs: &[Direction] = if sq1_file == sq2_file {
            if sq1_rank < sq2_rank {
                &[Direction::IncRank]
            } else {
                &[Direction::DecRank]
            }
        } else if sq1_rank == sq2_rank {
            if sq1_file < sq2_file {
                &[Direction::IncFile]
            } else {
                &[Direction::DecFile]
            }
        } else if sq1_file < sq2_file {
            if sq1_rank < sq2_rank {
                &[Direction::IncRank, Direction::IncFile]
            } else {
                &[Direction::DecRank, Direction::IncFile]
            }
        } else {
            if sq1_rank < sq2_rank {
                &[Direction::IncRank, Direction::DecFile]
            } else {
                &[Direction::DecRank, Direction::DecFile]
            }
        };
        let mut curr_bb = BitBoard::from_square(sq1);
        let end_bb = BitBoard::from_square(sq2);

        let mut ray = BitBoard::empty();

        while !curr_bb.const_equals(end_bb) {
            let mut dir_idx = 0;
            while dir_idx < dirs.len() {
                curr_bb = curr_bb.shift(dirs[dir_idx]);
                dir_idx += 1;
            }
            ray = ray.const_bit_or(curr_bb);
        }
        ray.const_bit_and(end_bb.const_bit_not())
    }

    pub(crate) fn to_val(self) -> u64 {
        self.0
    }

    pub(crate) fn to_squares(mut self) -> Vec<Square> {
        let mut sqs = Vec::with_capacity(14);
        while self.0 != 0 {
            let sq = self.pop_lsb();
            sqs.push(sq);
        }
        sqs
    }

    pub(crate) fn move_piece(&mut self, src: Square, dest: Square) {
        self.clear_square(src);
        self.set_square(dest);
    }

    pub(crate) fn set_square(&mut self, square: Square) {
        self.0 |= 1 << square as u64
    }

    pub(crate) fn clear_square(&mut self, square: Square) {
        self.0 &= !(1 << square as u64)
    }

    pub(crate) fn is_square_set(&self, square: Square) -> bool {
        self.0 & 1 << (square as u64) != 0
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn shift(mut self, dir: Direction) -> BitBoard {
        const EAST_SHIFT_MASK: u64 = 0x7F7F7F7F7F7F7F7F;
        const WEST_SHIFT_MASK: u64 = 0xFEFEFEFEFEFEFEFE;
        match dir {
            Direction::IncFile => self.0 &= EAST_SHIFT_MASK,
            Direction::DecFile => self.0 &= WEST_SHIFT_MASK,
            _ => (),
        }
        let shift_amt = dir as isize;
        if shift_amt >= 0 {
            self.0 <<= shift_amt
        } else {
            self.0 >>= -shift_amt
        }
        self
    }

    pub(crate) fn get_lsb(&self) -> Square {
        debug_assert!(self.0 != 0, "want != 0, got 0");
        let idx: u8 = self.0.trailing_zeros().try_into().unwrap();
        Square::from_repr(idx).unwrap()
    }

    pub(crate) fn pop_lsb(&mut self) -> Square {
        let lsb = self.get_lsb();
        self.0 &= self.0 - 1;
        lsb
    }

    pub(crate) fn swap_bytes(&self) -> BitBoard {
        BitBoard(self.0.swap_bytes())
    }

    pub(crate) fn num_squares_set(mut self) -> u8 {
        let mut count = 0;

        while self.0 != 0 {
            count += 1;
            self.0 &= self.0 - 1;
        }

        count
    }

    pub(crate) const fn const_bit_or(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }

    pub(crate) const fn const_bit_and(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }

    pub(crate) const fn const_equals(self, other: BitBoard) -> bool {
        self.0 == other.0
    }

    pub(crate) const fn const_bit_not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 = self.0 ^ rhs.0
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl Sub for BitBoard {
    type Output = BitBoard;

    fn sub(self, other: BitBoard) -> Self::Output {
        Self(self.0.wrapping_sub(other.0))
    }
}

impl SubAssign for BitBoard {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0)
    }
}

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);

        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = Square::from_repr(rank * 8 + file).unwrap();
                let ch = if self.is_square_set(square) { 'X' } else { '.' };
                board_str.push(ch);
            }
            if rank != 0 {
                board_str.push('\n');
            }
        }

        write!(f, "{}", board_str)
    }
}

#[cfg(test)]
mod tests {
    use super::Square::*;
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_bitboard_from_squares() {
        let got = BitBoard::from_squares(&[A1, A2, E4]);
        let want = BitBoard(0b0000000000000000000000000000000000010000000000000000000100000001);
        assert_eq!(got, want);
    }

    #[test]
    fn test_debug() {
        let got = BitBoard::from_squares(&[A8, B7, C6, D5, E4, F3, G2, H1]);
        let want = "X.......\n.X......\n..X.....\n...X....\n....X...\n.....X..\n......X.\n.......X";
        assert_eq!(format!("{:?}", got), want);
    }

    #[test_case([B8, G6, A4, F1] ; "first")]
    fn test_is_piece_at(piece_squares: [Square; 4]) {
        let all_other_squares: Vec<Square> = Square::iter()
            .filter(|s| !piece_squares.contains(s))
            .collect();

        let bb = BitBoard::from_squares(&piece_squares);
        let inv_bb = BitBoard::from_squares(&all_other_squares);

        for sq in Square::iter() {
            if piece_squares.contains(&sq) {
                assert!(bb.is_square_set(sq));
                assert!(!inv_bb.is_square_set(sq));
            } else {
                assert!(!bb.is_square_set(sq));
                assert!(inv_bb.is_square_set(sq));
            }
        }
    }

    #[test_case([B8, G6, A4, F1],
        0b0000001000000000010000000000000000000001000000000000000000100000
    ; "first")]
    fn test_is_piece_at_binary_number(piece_squares: [Square; 4], bin_num: u64) {
        let bb = BitBoard(bin_num);
        for sq in Square::iter() {
            if piece_squares.contains(&sq) {
                assert!(bb.is_square_set(sq));
            } else {
                assert!(!bb.is_square_set(sq));
            }
        }
    }

    #[test_case(BitBoard::from_square(D4), &[Direction::IncRank], BitBoard::from_square(D5) ; "n")]
    #[test_case(BitBoard::from_square(D4), &[Direction::DecRank], BitBoard::from_square(D3) ; "s")]
    #[test_case(BitBoard::from_square(D4), &[Direction::IncFile], BitBoard::from_square(E4) ; "e")]
    #[test_case(BitBoard::from_square(D4), &[Direction::DecFile], BitBoard::from_square(C4) ; "w")]
    #[test_case(BitBoard::from_square(D4), &[Direction::IncFile, Direction::IncFile], BitBoard::from_square(F4) ; "ee")]
    #[test_case(BitBoard::from_square(D4), &[Direction::IncRank, Direction::IncFile], BitBoard::from_square(E5) ; "ne")]
    #[test_case(BitBoard::from_square(D4), &[Direction::IncRank, Direction::DecFile], BitBoard::from_square(C5) ; "nw")]
    #[test_case(BitBoard::from_square(D4), &[Direction::DecRank, Direction::IncFile], BitBoard::from_square(E3) ; "se")]
    #[test_case(BitBoard::from_square(D4), &[Direction::DecRank, Direction::DecFile], BitBoard::from_square(C3) ; "sw")]
    #[test_case(BitBoard::from_square(A6), &[Direction::DecFile], BitBoard(0) ; "overlap w")]
    #[test_case(BitBoard::from_square(H3), &[Direction::IncFile], BitBoard(0) ; "overlap e")]
    #[test_case(BitBoard::from_square(A2), &[Direction::DecRank, Direction::DecFile], BitBoard(0) ; "overlap sw")]
    #[test_case(BitBoard::from_square(H7), &[Direction::IncRank, Direction::IncFile], BitBoard(0) ; "overlap ne")]
    fn test_shift(mut inp: BitBoard, shift_dirs: &[Direction], want: BitBoard) {
        for &shift_dir in shift_dirs {
            inp = inp.shift(shift_dir);
        }
        assert_eq!(inp, want);
    }

    #[test_case(D4, vec![vec![Direction::IncRank]], BitBoard::from_square(D5) ; "one")]
    #[test_case(D4, vec![vec![Direction::IncRank], vec![Direction::DecRank]], BitBoard::from_squares(&[D5, D3]) ; "two")]
    #[test_case(D4, vec![
        vec![Direction::IncRank],
        vec![Direction::DecRank],
        vec![Direction::IncFile],
        vec![Direction::DecFile],
    ], BitBoard::from_squares(&[D5, D3, E4, C4]) ; "all")]
    #[test_case(D4, vec![vec![Direction::IncRank, Direction::IncFile]], BitBoard::from_square(E5) ; "multi")]
    fn test_from_square_shifts(
        inp_square: Square,
        shift_dirs_list: Vec<Vec<Direction>>,
        want: BitBoard,
    ) {
        let got = BitBoard::from_square_shifts(inp_square, &shift_dirs_list);
        assert_eq!(got, want);
    }

    #[test_case(BitBoard(0b1001000), D1, BitBoard(0b1000000) ; "D1")]
    #[test_case(BitBoard(0b1000000), G1, BitBoard(0b0000000) ; "G1")]
    fn test_pop_lsb(mut inp: BitBoard, lsb_want: Square, res_want: BitBoard) {
        let lsb_got = inp.pop_lsb();
        assert_eq!(lsb_got, lsb_want);
        assert_eq!(inp, res_want);
    }

    #[test_case(A8, A3, BitBoard::from_squares(&[A4, A5, A6, A7]) ; "s")]
    #[test_case(A8, D8, BitBoard::from_squares(&[B8, C8]) ; "e")]
    #[test_case(B4, E1, BitBoard::from_squares(&[C3, D2]) ; "se")]
    #[test_case(E1, B4, BitBoard::from_squares(&[C3, D2]) ; "nw")]
    fn test_from_ray_excl(start: Square, end: Square, want: BitBoard) {
        let got = BitBoard::from_ray_excl(start, end);
        assert_eq!(got, want);
    }
}
