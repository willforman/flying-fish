// use std::ops::Index;
use std::fmt;

use strum_macros::{EnumIter,EnumString,FromRepr,Display};
use strum::IntoEnumIterator;

#[allow(dead_code)]
#[rustfmt::skip]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, EnumString, FromRepr, Display)]
pub(crate) enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[derive(PartialEq, Eq)]
pub(crate) struct BitBoard(u64);

impl BitBoard {
    pub(crate) fn new() -> Self {
        BitBoard(0)
    }

    pub(crate) fn from_squares(squares: &[Square]) -> Self {
        BitBoard(
            squares
            .iter()
            .fold(0, |board, sq| board | 1 << (*sq as u8))
        )
    }

    pub(crate) fn add_piece(&mut self, square: &Square) {
        self.0 |= 1 << *square as u64
    }

    pub(crate) fn is_piece_at(&self, square: &Square) -> bool {
        self.0 & 1 << (*square as u64) != 0
    }
}

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);

        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = Square::from_repr(rank * 8 + file).unwrap();
                let ch = if self.is_piece_at(&square) {
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
                assert!(bb.is_piece_at(&sq));
                assert!(!inv_bb.is_piece_at(&sq));
            } else {
                assert!(!bb.is_piece_at(&sq));
                assert!(inv_bb.is_piece_at(&sq));
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
                assert!(bb.is_piece_at(&sq));
            } else {
                assert!(!bb.is_piece_at(&sq));
            }
        }
    }


}
