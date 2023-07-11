use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Square, Direction};
use crate::position::{Piece};

use super::GenerateSlidingMoves;

enum MaskType {
    Bit,
    File,
    Diagonal,
    AntiDiagonal,
}

#[derive(Debug)]
struct SquareMasks {
    bit: BitBoard,
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
            masks_list.push(SquareMasks {
                bit: BitBoard::from_square(square),
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

    fn get_mask(&self, square: Square, mask_type: MaskType) -> BitBoard {
        match mask_type {
            MaskType::Bit => self.masks_list[square as usize].bit,
            MaskType::File => self.masks_list[square as usize].file,
            MaskType::Diagonal => self.masks_list[square as usize].diag,
            MaskType::AntiDiagonal => self.masks_list[square as usize].anti_diag,
        }
    }
    
    fn get_moves(&self, occupancy: BitBoard, mask: BitBoard, bit_mask: BitBoard) -> BitBoard {
        let mut forward = occupancy & mask;
        let mut reverse = forward.swap_bytes();
        forward -= bit_mask;
        reverse -= bit_mask.swap_bytes();
        forward ^= reverse.swap_bytes();
        forward &= mask;
        forward
    }
}

impl GenerateSlidingMoves for HyperbolaQuintessence {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        let bit_mask = self.get_mask(square, MaskType::Bit);

        match piece {
            Piece::Bishop => { 
                self.get_moves(occupancy, self.get_mask(square, MaskType::Diagonal), bit_mask) |
                self.get_moves(occupancy, self.get_mask(square, MaskType::AntiDiagonal), bit_mask)
            }
            Piece::Rook => {
                self.get_moves(occupancy, self.get_mask(square, MaskType::File), bit_mask)// |
                // self.get_moves(occupancy, self.get_mask(square, MaskType::AntiDiagonal), bit_mask)
            }
            Piece::Queen => { todo!() }
            _ => panic!("piece type: want [bishop, rook, queen], got {}", piece.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;
    use test_case::test_case;

    #[test_case(MaskType::Bit, D4, BitBoard::from_square(D4) ; "bit")]
    #[test_case(MaskType::File, A8, BitBoard::from_squares(&[A7, A6, A5, A4, A3, A2, A1]) ; "file corner")]
    #[test_case(MaskType::File, D4, BitBoard::from_squares(&[D8, D7, D6, D5, D3, D2, D1]) ; "file middle")]
    #[test_case(MaskType::Diagonal, A8, BitBoard::from_squares(&[B7, C6, D5, E4, F3, G2, H1]) ; "diagonal main")]
    #[test_case(MaskType::Diagonal, D4, BitBoard::from_squares(&[A7, B6, C5, E3, F2, G1]) ; "diagonal off main")]
    #[test_case(MaskType::Diagonal, A1, BitBoard::from_squares(&[]) ; "diagonal empty")]
    #[test_case(MaskType::AntiDiagonal, H8, BitBoard::from_squares(&[G7, F6, E5, D4, C3, B2, A1]) ; "anti diagonal main")]
    #[test_case(MaskType::AntiDiagonal, D5, BitBoard::from_squares(&[G8, F7, E6, C4, B3, A2]) ; "anti diagonal off main")]
    #[test_case(MaskType::AntiDiagonal, A8, BitBoard::from_squares(&[]) ; "anti diagonal empty")]
    fn test_mask(mask_type: MaskType, check_square: Square, want: BitBoard) {
        let hq = HyperbolaQuintessence::new();
        assert_eq!(hq.get_mask(check_square, mask_type), want);
    }

    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[A1, B2, C3, E5, F6, G7, H8, C5, B6, A7, E3, F2, G1]) ; "bishop no blockers")]
    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[B2, A7, E5]), BitBoard::from_squares(&[B2, C3, E5, C5, B6, A7, E3, F2, G1]) ; "bishop many blockers")]
    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[B2, A7, E5, A1, B1, F8, G6, C4]), BitBoard::from_squares(&[B2, C3, E5, C5, B6, A7, E3, F2, G1]) ; "bishop irrelevant blockers")]
    #[test_case(Piece::Rook, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[D1, D2, D3, D5, D6, D7, D8]) ; "rook no blockers")]
    fn test_gen_moves(piece: Piece, square: Square, occupancy: BitBoard, want: BitBoard) {
        let hq = HyperbolaQuintessence::new();
        let got = hq.gen_moves(piece, square, occupancy);
        assert_eq!(got, want);
    }
}
