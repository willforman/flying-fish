// use std::ops::Index;
use std::fmt;

use std::string::ToString;
use strum_macros::{EnumIter,EnumString,FromRepr,Display};
use strum::IntoEnumIterator;

#[allow(dead_code)]
#[rustfmt::skip]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, FromRepr, Display)]
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
    pub(crate) fn from_squares(squares: &[Square]) -> Self {
        BitBoard(
            squares
            .iter()
            .fold(0, |board, sq| board | 1 << (*sq as u8))
        )
    }

    pub(crate) fn is_piece_at(&self, square: &Square) -> bool {
        self.0 & 1 << (*square as u64) != 0
    }
}

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);

        for (idx, sq) in Square::iter().rev().enumerate() {
            let ch = if self.is_piece_at(&sq) {
                'X'
            } else {
                '.'
            };
            board_str.push(ch);
            if (idx + 1) % 8 == 0 && (idx + 1) != 64 {
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

    #[test]
    fn test_bitboard_from_squares() {
        let got = BitBoard::from_squares(&[A1, A2, E4]);
        let want = BitBoard(0b0000000000000000000000000000000000010000000000000000000100000001);
        assert_eq!(got, want);
    }

    #[test]
    fn test_debug() {
        let got = BitBoard::from_squares(&[H8, G7, F6, E5, D4, C3, B2, A1]);
        let want = "X.......\n.X......\n..X.....\n...X....\n....X...\n.....X..\n......X.\n.......X";
        assert_eq!(format!("{:?}", got), want);
    }

    #[test]
    fn test_is_piece_at() {
        let bb = BitBoard::from_squares(&[A4]);
        for sq in Square::iter() {
            if sq == A4 {
                assert!(bb.is_piece_at(&sq));
            } else {
                assert!(!bb.is_piece_at(&sq));
            }
        }

        let bb = BitBoard(0b1111111111111111111111111111111111111111111111111111111111110111);
        for sq in Square::iter() {
            if sq == D1 {
                assert!(!bb.is_piece_at(&sq));
            } else {
                assert!(bb.is_piece_at(&sq));
            }
        }
    }
}
