use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Square, Direction};

#[derive(Debug)]
struct SquareMasks {
    file: BitBoard,
    diag: BitBoard,
    anti_diag: BitBoard
}

pub struct HyperbolaQuintessence {
    masks_list: [SquareMasks; 64]
}


impl HyperbolaQuintessence {
    fn new() -> Self {
        let mut file_dirs = vec![
            vec![Direction::North; 1],
            vec![Direction::North; 2],
            vec![Direction::North; 3],
            vec![Direction::North; 4],
            vec![Direction::North; 5],
            vec![Direction::North; 6],
            vec![Direction::North; 7],
        ];
        let mut diag_dirs = vec![
            [vec![Direction::North; 1], vec![Direction::West; 1]].concat(),
            [vec![Direction::North; 2], vec![Direction::West; 2]].concat(),
            [vec![Direction::North; 3], vec![Direction::West; 3]].concat(),
            [vec![Direction::North; 4], vec![Direction::West; 4]].concat(),
            [vec![Direction::North; 5], vec![Direction::West; 5]].concat(),
            [vec![Direction::North; 6], vec![Direction::West; 6]].concat(),
            [vec![Direction::North; 7], vec![Direction::West; 7]].concat(),
        ];
        let mut anti_diag_dirs = vec![
            [vec![Direction::North; 1], vec![Direction::East; 1]].concat(),
            [vec![Direction::North; 2], vec![Direction::East; 2]].concat(),
            [vec![Direction::North; 3], vec![Direction::East; 3]].concat(),
            [vec![Direction::North; 4], vec![Direction::East; 4]].concat(),
            [vec![Direction::North; 5], vec![Direction::East; 5]].concat(),
            [vec![Direction::North; 6], vec![Direction::East; 6]].concat(),
            [vec![Direction::North; 7], vec![Direction::East; 7]].concat(),
        ];

        let mut masks_list = Vec::with_capacity(64);

        for (idx, square) in Square::iter().enumerate() {
            // println!("{:?}", anti_diag_dirs);
            masks_list.push(SquareMasks {
                file: BitBoard::from_square_shifts(square, &file_dirs),
                diag: BitBoard::from_square_shifts(square, &diag_dirs),
                anti_diag: BitBoard::from_square_shifts(square, &anti_diag_dirs),
            });

            if (idx + 1) % 8 == 0 && idx != 63 {
                let rank = (idx + 1) / 8;

                file_dirs[7 - rank] = vec![Direction::South; rank];
                diag_dirs[7 - rank] = [vec![Direction::South; rank], vec![Direction::East; rank]].concat();
                anti_diag_dirs[7 - rank] = [vec![Direction::South; rank], vec![Direction::West; rank]].concat();
            }
        }

        Self {
            masks_list: masks_list.try_into().unwrap(),
        }
    }

    fn get(&self, square: Square) -> &SquareMasks {
        &self.masks_list[square as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;
    use test_case::test_case;

    #[test_case(A8, BitBoard::from_squares(&[A7, A6, A5, A4, A3, A2, A1]) ; "corner")]
    #[test_case(D4, BitBoard::from_squares(&[D8, D7, D6, D5, D3, D2, D1]) ; "middle")]
    fn test_files_mask(check_square: Square, want: BitBoard) {
        let hq = HyperbolaQuintessence::new();
        assert_eq!(hq.get(check_square).file, want);
    }

    #[test_case(A8, BitBoard::from_squares(&[B7, C6, D5, E4, F3, G2, H1]) ; "main")]
    #[test_case(D4, BitBoard::from_squares(&[A7, B6, C5, E3, F2, G1]) ; "off main")]
    #[test_case(A1, BitBoard::from_squares(&[]) ; "empty")]
    fn test_diag_mask(check_square: Square, want: BitBoard) {
        let hq = HyperbolaQuintessence::new();
        assert_eq!(hq.get(check_square).diag, want);
    }

    #[test_case(H8, BitBoard::from_squares(&[G7, F6, E5, D4, C3, B2, A1]) ; "main")]
    #[test_case(D5, BitBoard::from_squares(&[G8, F7, E6, C4, B3, A2]) ; "off main")]
    #[test_case(A8, BitBoard::from_squares(&[]) ; "empty")]
    fn test_anti_diag_mask(check_square: Square, want: BitBoard) {
        let hq = HyperbolaQuintessence::new();
        assert_eq!(hq.get(check_square).anti_diag, want);
    }
}
