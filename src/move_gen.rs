use std::ops::{Index, IndexMut};

use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Square, Direction};

struct SquareToMoveDatabase([BitBoard; 64]);

impl Index<Square> for SquareToMoveDatabase {
    type Output = BitBoard;

    fn index(&self, square: Square) -> &Self::Output {
        &self.0[square as usize]
    }
}

pub struct MoveGen {
    pawn_atks: [SquareToMoveDatabase; 2],
    knight_atks: SquareToMoveDatabase,
    bishop_atks: SquareToMoveDatabase,
    king_atks: SquareToMoveDatabase,
}

fn calc_knight_atks() -> SquareToMoveDatabase {
    let dirs: Vec<Vec<Direction>> = vec![
        vec![Direction::North, Direction::North, Direction::East], 
        vec![Direction::North, Direction::North, Direction::West], 
        vec![Direction::South, Direction::South, Direction::East], 
        vec![Direction::South, Direction::South, Direction::West], 
        vec![Direction::North, Direction::East, Direction::East], 
        vec![Direction::North, Direction::West, Direction::West], 
        vec![Direction::South, Direction::East, Direction::East], 
        vec![Direction::South, Direction::West, Direction::West], 
    ];

    let bbs: [BitBoard; 64] = Square::iter()
        .map(|sq| BitBoard::from_square_shifts(sq, &dirs))
        .collect::<Vec<BitBoard>>()
        .try_into()
        .unwrap();
    SquareToMoveDatabase(bbs)
}

fn calc_king_atks() -> SquareToMoveDatabase {
    let dirs: Vec<Vec<Direction>> = vec![
        vec![Direction::North], 
        vec![Direction::East], 
        vec![Direction::West], 
        vec![Direction::South], 
        vec![Direction::North, Direction::East], 
        vec![Direction::North, Direction::West], 
        vec![Direction::South, Direction::East], 
        vec![Direction::South, Direction::West], 
    ];

    let bbs: [BitBoard; 64] = Square::iter()
        .map(|sq| BitBoard::from_square_shifts(sq, &dirs))
        .collect::<Vec<BitBoard>>()
        .try_into()
        .unwrap();
    SquareToMoveDatabase(bbs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;
    use test_case::test_case;
    
    #[test_case(D4, BitBoard::from_squares(&[B5, C6, E6, F5, B3, C2, E2, F3]) ; "center")]
    #[test_case(A8, BitBoard::from_squares(&[B6, C7]) ; "corner")]
    #[test_case(A4, BitBoard::from_squares(&[B6, C5, C3, B2]) ; "edge")]
    fn test_calc_knight_atks(square: Square, want: BitBoard) {
        let got = calc_knight_atks();
        let sq_got = &got[square];
        assert_eq!(sq_got, &want);
    }

    #[test_case(D4, BitBoard::from_squares(&[C5, D5, E5, C4, E4, C3, D3, E3]) ; "center")]
    #[test_case(A8, BitBoard::from_squares(&[A7, B7, B8]) ; "corner")]
    #[test_case(C1, BitBoard::from_squares(&[B1, B2, C2, D2, D1]) ; "edge")]
    fn test_calc_king_atks(square: Square, want: BitBoard) {
        let got = calc_king_atks();
        let sq_got = &got[square];
        assert_eq!(sq_got, &want);
    }
}
