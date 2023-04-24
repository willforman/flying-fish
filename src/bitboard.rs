// use std::ops::Index;
use std::fmt;

#[allow(dead_code)]
#[rustfmt::skip]
#[repr(u8)]
#[derive(Clone, Copy)]
pub(crate) enum Square {
    A1, A2, A3, A4, A5, A6, A7, A8,
    B1, B2, B3, B4, B5, B6, B7, B8,
    C1, C2, C3, C4, C5, C6, C7, C8,
    D1, D2, D3, D4, D5, D6, D7, D8,
    E1, E2, E3, E4, E5, E6, E7, E8,
    F1, F2, F3, F4, F5, F6, F7, F8,
    G1, G2, G3, G4, G5, G6, G7, G8,
    H1, H2, H3, H4, H5, H6, H7, H8,
}

#[derive(PartialEq, Eq)]
pub(crate) struct BitBoard(u64);

impl BitBoard {
    fn new(squares: &[Square]) -> Self {
        BitBoard(
            squares
            .iter()
            .fold(0, |board, sq| board | 1 << (63 - (*sq as u8)))
        )
    }
}

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);
        for rank in 0..8 {
            for file in (0..8).rev() {
                let idx = rank * 8 + file;
                let ch = if 1 << idx & self.0 != 0 {
                    'X'
                } else {
                    '.'
                };
                board_str.push(ch);
            }
            if rank != 7 {
                board_str.push('\n');
            }
        }
        write!(f, "{}", board_str)
    }
}

// impl Index<Square> for BitBoard {
//     type Output = ;
// }
//

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;

    #[test]
    fn test_new_bitboard() {
        let got = BitBoard::new(&[A1, A3, A8]);
        let want = BitBoard(0b1010000100000000000000000000000000000000000000000000000000000000);
        assert_eq!(got, want);
    }
}
