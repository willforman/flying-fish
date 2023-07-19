use std::fmt;
use std::ops::{BitAnd,BitOr,BitXor,Not,Sub, SubAssign, BitXorAssign, BitAndAssign, BitOrAssign};

use strum_macros::{EnumIter,EnumString,FromRepr,Display};
use strum::IntoEnumIterator;

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
    pub(crate) fn abs_diff(self, other: Square) -> u8 {
        (self as u8).abs_diff(other as u8)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Move {
    pub src: Square,
    pub dest: Square,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    North,
    East,
    South,
    West
}

// Need to clear MSB/LSB from bb to prevent overlap
const EAST_SHIFT_MASK: u64 = 0x7F7F7F7F7F7F7F7F;
const WEST_SHIFT_MASK: u64 = 0xFEFEFEFEFEFEFEFE;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct BitBoard(u64);

impl BitBoard {
    pub(crate) fn empty() -> Self {
        BitBoard(0)
    }

    pub(crate) fn full() -> Self {
        BitBoard(u64::max_value())
    }

    pub(crate) fn from_square(square: Square) -> Self {
        BitBoard(1 << (square as u8))
    }

    // TODO: convert to From<&[Square]>
    pub(crate) fn from_squares(squares: &[Square]) -> Self {
        BitBoard(
            squares
            .iter()
            .fold(0, |board, sq| board | 1 << (*sq as u8))
        )
    }

    pub(crate) fn from_val(val: u64) -> Self {
        BitBoard(val)
    }

    pub(crate) fn from_square_shifts(square: Square, shift_dirs_list: &Vec<Vec<Direction>>) -> Self {
        let start = BitBoard::from_square(square);
        let res = shift_dirs_list.iter()
            .fold(start.clone(), |acc, shift_dirs| {
                let mut shifted = start.clone();
                for &sd in shift_dirs {
                    shifted.shift(sd);
                }
                acc | shifted
            });
        res & !start
    }

    pub(crate) fn from_ray_excl(start: Square, end: Square) -> Self {
        let diff = start.abs_diff(end);
        let dir = if diff % 8 == 0 {
            if start > end {
                Direction::South
            } else {
                Direction::North
            }
        } else {
            if start > end {
                Direction::West
            } else {
                Direction::East
            }
        };
        let iterations = if diff % 8 == 0 {
            diff / 8
        } else {
            diff
        } - 1;

        let mut curr_bb = BitBoard::from_square(start);
        let mut ray = BitBoard::empty();

        for _ in 0..iterations {
            curr_bb.shift(dir);
            ray |= curr_bb;
        }
        ray
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

    pub(crate) fn move_piece(&mut self, mve: Move) {
        self.clear_square(mve.src);
        self.set_square(mve.dest);
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

    pub(crate) fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn shift(&mut self, dir: Direction) {
        const EAST_SHIFT_MASK: u64 = 0x7F7F7F7F7F7F7F7F;
        const WEST_SHIFT_MASK: u64 = 0xFEFEFEFEFEFEFEFE;
        match dir {
            Direction::North => self.0 <<= 8,
            Direction::South => self.0 >>= 8,
            Direction::East => {
                self.0 = (self.0 & EAST_SHIFT_MASK) << 1
            },
            Direction::West => {
                self.0 = (self.0 & WEST_SHIFT_MASK) >> 1
            },
        }
    }

    pub(crate) fn get_lsb(&self) -> Square {
        let idx = self.0.trailing_zeros();
        Square::from_repr(idx.try_into().unwrap()).unwrap()
    }

    pub(crate) fn pop_lsb(&mut self) -> Square {
        let lsb = self.get_lsb();
        self.0 &= self.0 - 1;
        lsb
    }

    pub(crate) fn swap_bytes(&self) -> BitBoard {
        BitBoard(self.0.swap_bytes())
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
                let ch = if self.is_square_set(square) {
                    'X'
                } else {
                    '.'
                };
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
    use super::*;
    use super::Square::*;
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

    #[test_case(BitBoard::from_square(D4), vec![Direction::North], BitBoard::from_square(D5) ; "n")]
    #[test_case(BitBoard::from_square(D4), vec![Direction::South], BitBoard::from_square(D3) ; "s")]
    #[test_case(BitBoard::from_square(D4), vec![Direction::East], BitBoard::from_square(E4) ; "e")]
    #[test_case(BitBoard::from_square(D4), vec![Direction::West], BitBoard::from_square(C4) ; "w")]
    #[test_case(BitBoard::from_square(D4), vec![Direction::East, Direction::East], BitBoard::from_square(F4) ; "ee")]
    #[test_case(BitBoard::from_square(A6), vec![Direction::West], BitBoard(0) ; "overlap")]
    fn test_shift(mut inp: BitBoard, shift_dirs: Vec<Direction>, want: BitBoard) {
        for shift_dir in shift_dirs {
            inp.shift(shift_dir);
        }
        assert_eq!(inp, want);
    }

    #[test_case(D4, vec![vec![Direction::North]], BitBoard::from_square(D5) ; "one")]
    #[test_case(D4, vec![vec![Direction::North], vec![Direction::South]], BitBoard::from_squares(&[D5, D3]) ; "two")]
    #[test_case(D4, vec![
        vec![Direction::North], 
        vec![Direction::South],
        vec![Direction::East],
        vec![Direction::West],
    ], BitBoard::from_squares(&[D5, D3, E4, C4]) ; "all")]

    #[test_case(D4, vec![vec![Direction::North, Direction::East]], BitBoard::from_square(E5) ; "multi")]
    fn test_from_square_shifts(inp_square: Square, shift_dirs_list: Vec<Vec<Direction>>, want: BitBoard) {
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

    #[test_case(A8, A3, BitBoard::from_squares(&[A4, A5, A6, A7]))]
    #[test_case(A8, D8, BitBoard::from_squares(&[B8, C8]))]
    fn test_from_ray_excl(start: Square, end: Square, want: BitBoard) {
        let got = BitBoard::from_ray_excl(start, end);
        assert_eq!(got, want);
    }
}
