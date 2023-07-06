use std::ops::{Index, IndexMut};

use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Square, Direction};
use crate::position::Side;

struct SquareToMoveDatabase([BitBoard; 64]);

impl SquareToMoveDatabase {
    fn get_bitboard(&self, square: Square) -> &BitBoard {
        &self.0[square as usize]
    }
}

struct ColoredSquareToMoveDatabase {
    white: SquareToMoveDatabase,
    black: SquareToMoveDatabase,
}

impl ColoredSquareToMoveDatabase {
    fn get_square_database(&self, side: Side) -> &SquareToMoveDatabase {
        match side {
            Side::White => &self.white,
            Side::Black => &self.black,
        }
    }
}

pub struct MoveGen {
    pawn_atks: ColoredSquareToMoveDatabase,
    knight_atks: SquareToMoveDatabase,
    bishop_atks: SquareToMoveDatabase,
    king_atks: SquareToMoveDatabase,
}

fn calc_pawn_atks() -> ColoredSquareToMoveDatabase {
    let white_single_push_dirs: Vec<Vec<Direction>> = vec![vec![Direction::North]]; 
    let white_double_push_dirs: Vec<Vec<Direction>> = vec![vec![Direction::North], vec![Direction::North, Direction::North]]; 
    let black_single_push_dirs: Vec<Vec<Direction>> = vec![vec![Direction::South]]; 
    let black_double_push_dirs: Vec<Vec<Direction>> = vec![vec![Direction::South], vec![Direction::South, Direction::South]]; 
    let edge_push_dirs: Vec<Vec<Direction>> = vec![]; 

    // [A8, H8]: edge
    // [A7, H7]: black double
    // [A6, H3]: double
    // [A2, H2]: white double
    // [A1, H1]: edge
    let white_bbs: [BitBoard; 64] = Square::iter()
        .map(|sq| {
            if sq >= Square::A8 || sq <= Square::H1 {
                BitBoard::from_square_shifts(sq, &edge_push_dirs)
            } else if sq <= Square::H2 {
                BitBoard::from_square_shifts(sq, &white_double_push_dirs)
            } else {
                BitBoard::from_square_shifts(sq, &white_single_push_dirs)
            }
        })
        .collect::<Vec<BitBoard>>()
        .try_into()
        .unwrap();

    let black_bbs: [BitBoard; 64] = Square::iter()
        .map(|sq| {
            if sq >= Square::A8 || sq <= Square::H1 {
                BitBoard::from_square_shifts(sq, &edge_push_dirs)
            } else if sq >= Square::A7 {
                BitBoard::from_square_shifts(sq, &black_double_push_dirs)
            } else {
                BitBoard::from_square_shifts(sq, &black_single_push_dirs)
            }
        })
        .collect::<Vec<BitBoard>>()
        .try_into()
        .unwrap();

    ColoredSquareToMoveDatabase {
        white: SquareToMoveDatabase(white_bbs),
        black: SquareToMoveDatabase(black_bbs),
    }
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
        let sq_got = got.get_bitboard(square);
        assert_eq!(sq_got, &want);
    }

    #[test_case(D4, BitBoard::from_squares(&[C5, D5, E5, C4, E4, C3, D3, E3]) ; "center")]
    #[test_case(A8, BitBoard::from_squares(&[A7, B7, B8]) ; "corner")]
    #[test_case(C1, BitBoard::from_squares(&[B1, B2, C2, D2, D1]) ; "edge")]
    fn test_calc_king_atks(square: Square, want: BitBoard) {
        let got = calc_king_atks();
        let sq_got = got.get_bitboard(square);
        assert_eq!(sq_got, &want);
    }

    #[test_case(D2, Side::White, BitBoard::from_squares(&[D3, D4]) ; "white double")]
    #[test_case(B3, Side::White, BitBoard::from_squares(&[B4]) ; "white single")]
    #[test_case(G7, Side::White, BitBoard::from_squares(&[G8]) ; "white single edge")]
    #[test_case(G8, Side::White, BitBoard::from_squares(&[]) ; "white edge")]
    #[test_case(D7, Side::Black, BitBoard::from_squares(&[D6, D5]) ; "black double")]
    #[test_case(B6, Side::Black, BitBoard::from_squares(&[B5]) ; "black single")]
    #[test_case(G2, Side::Black, BitBoard::from_squares(&[G1]) ; "black single edge")]
    #[test_case(G1, Side::Black, BitBoard::from_squares(&[]) ; "black edge")]
    fn test_calc_pawn_atks(square: Square, side: Side, want: BitBoard) {
        let got = calc_pawn_atks();
        let sq_got = got.get_square_database(side).get_bitboard(square);
        assert_eq!(sq_got, &want);
    }
}
