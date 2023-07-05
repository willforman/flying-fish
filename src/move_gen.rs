use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Square, Direction};

pub struct MoveGen {
    pawn_atks: [[BitBoard; 64]; 2],
    knight_atks: [BitBoard; 64],
    bishop_atks: [BitBoard; 64],
    king_atks: [BitBoard; 64],
}

fn calc_king_atks() -> [BitBoard; 64] {
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

    Square::iter()
        .map(|sq| BitBoard::from_square_shifts(&sq, &dirs))
        .collect::<Vec<BitBoard>>()
        .try_into()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;
    use test_case::test_case;
    
    #[test_case(D4, BitBoard::from_squares(&[C5, D5, E5, C4, E4, C3, D3, E3]) ; "center")]
    #[test_case(A8, BitBoard::from_squares(&[A7, B7, B8]) ; "corner")]
    #[test_case(C1, BitBoard::from_squares(&[B1, B2, C2, D2, D1]) ; "edge")]
    fn test_calc_king_atks(square: Square, want: BitBoard) {
        let got = calc_king_atks();
        let sq_got = &got[square as usize];
        assert_eq!(sq_got, &want);
    }
}
