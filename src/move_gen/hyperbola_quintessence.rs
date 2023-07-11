use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Square, Direction};

#[derive(Debug)]
struct SquareMasks {
    file: BitBoard,
    // diag: BitBoard,
    // anti_diag: BitBoard
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

        let mut masks_list = Vec::with_capacity(64);
        // for rank in (0..8).rev() {
        //     for file in 0..8 {
        //         let sq = Square::from_repr(rank * 8 + file).unwrap();
        //
        //         masks_list.push(SquareMasks {
        //             file: BitBoard::from_square_shifts(sq, &file_dirs)
        //         });
        //     }
        //     // Replace the biggest south vector with the smallest north vector
        //     if rank > 0 {
        //         file_dirs[usize::from(rank - 1)] = vec![Direction::North; (7 - rank).into()];
        //     }
        // }
        //
        for (idx, square) in Square::iter().enumerate() {
            println!("{}: {:?}", square.to_string(), file_dirs);
            masks_list.push(SquareMasks {
                file: BitBoard::from_square_shifts(square, &file_dirs)
            });

            if (idx + 1) % 8 == 0 && idx != 63 {
                let rank = (idx + 1) / 8;
                file_dirs[7 - rank] = vec![Direction::South; rank];
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

    #[test_case(A8, BitBoard::from_squares(&[A7, A6, A5, A4, A3, A2, A1]))]
    #[test_case(D4, BitBoard::from_squares(&[D8, D7, D6, D5, D3, D2, D1]))]
    fn test_new_files(inp_square: Square, want: BitBoard) {
        let hq = HyperbolaQuintessence::new();
        assert_eq!(hq.get(inp_square).file, want);
    }
}
