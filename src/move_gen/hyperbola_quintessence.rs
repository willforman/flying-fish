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

impl SquareMasks {
    fn get(&self, mask_type: MaskType) -> BitBoard {
        match mask_type {
            MaskType::Bit => self.bit,
            MaskType::File => self.file,
            MaskType::Diagonal => self.diag,
            MaskType::AntiDiagonal => self.anti_diag,
        }
    }
}

pub struct MasksList([SquareMasks; 64]);

impl MasksList {
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
        MasksList(masks_list.try_into().unwrap())
    }

    fn get(&self, square: Square) -> &SquareMasks {
        &self.0[square as usize]
    }
}

fn calc_rank_atks() -> [u8; 64 * 8] {
    const ROOK_OPTIONS: [u8; 8] = [
        1 << 7,
        1 << 6,
        1 << 5,
        1 << 4,
        1 << 3,
        1 << 2,
        1 << 1,
        1 << 0,
    ];

    let mut rank_atks_list = Vec::with_capacity(64 * 8);

    for rook in ROOK_OPTIONS {
        for pieces in 0..64 {
            let shifted_pieces = pieces << 1; // Ignore the first and last bit
            let occ = shifted_pieces | rook;
            let atks = occ ^ (shifted_pieces.wrapping_sub(rook));
            rank_atks_list.push(atks);
        }
    }

    rank_atks_list.try_into().unwrap()
}

pub struct HyperbolaQuintessence {
    masks_list: MasksList,
    rank_atks: [u8; 64 * 8],
}

impl HyperbolaQuintessence {
    fn new(masks_list: MasksList, rank_atks: [u8; 64 * 8]) -> Self {
        Self {
            masks_list,
            rank_atks,
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

    fn get_rank_moves(&self, occupancy: BitBoard, square: Square) -> BitBoard {
        let occ_val = occupancy.to_val();
        let sq_idx = square as u8;

        let file = sq_idx & 7;
        let rank_x8 = sq_idx & 56; // Rank times 8

        let rank_occ_x2 = u8::try_from((occ_val >> rank_x8) & 2 * 63).unwrap(); // 2 times the inner six bit occupancy used as index
        let atks = self.rank_atks[usize::from(4 * rank_occ_x2 + file)];

        return BitBoard::from_val((atks.wrapping_shl(rank_x8.into())).into());
    }
}

impl GenerateSlidingMoves for HyperbolaQuintessence {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        let masks = self.masks_list.get(square);
        let bit_mask = masks.get(MaskType::Bit);

        match piece {
            Piece::Bishop => { 
                self.get_moves(occupancy, masks.get(MaskType::Diagonal), bit_mask) |
                self.get_moves(occupancy, masks.get(MaskType::AntiDiagonal), bit_mask)
            }
            Piece::Rook => {
                self.get_moves(occupancy, masks.get(MaskType::File), bit_mask)// |
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
        let masks_list = MasksList::new();
        assert_eq!(masks_list.get(check_square).get(mask_type), want);
    }

    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[A1, B2, C3, E5, F6, G7, H8, C5, B6, A7, E3, F2, G1]) ; "bishop no blockers")]
    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[B2, A7, E5]), BitBoard::from_squares(&[B2, C3, E5, C5, B6, A7, E3, F2, G1]) ; "bishop many blockers")]
    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[B2, A7, E5, A1, B1, F8, G6, C4]), BitBoard::from_squares(&[B2, C3, E5, C5, B6, A7, E3, F2, G1]) ; "bishop irrelevant blockers")]
    #[test_case(Piece::Rook, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[D1, D2, D3, D5, D6, D7, D8]) ; "rook no blockers")]
    fn test_gen_moves(piece: Piece, square: Square, occupancy: BitBoard, want: BitBoard) {
        let masks_list = MasksList::new();
        let rank_atks = calc_rank_atks();
        let hq = HyperbolaQuintessence::new(masks_list, rank_atks);

        let got = hq.gen_moves(piece, square, occupancy);
        assert_eq!(got, want);
    }

    #[test_case(0, 0b11111110)]
    #[test_case(1, 0b00000010)]
    #[test_case(2, 0b00000110)]
    #[test_case(3, 0b00000010)]
    #[test_case(4, 0b00001110)]
    #[test_case(5, 0b00000010)]
    #[test_case(6, 0b00000110)]
    #[test_case(7, 0b00000010)]
    #[test_case(8, 0b00011110)]
    fn test_calc_rank_atks(rank_atks_idx: usize, want: u8) {
        let rank_atks = calc_rank_atks();
        let got = rank_atks[rank_atks_idx];
        assert_eq!(got, want);
    }

    #[test_case(H4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[A4, B4, C4, D4, E4, F4, G4]))]
    fn test_gen_rank_moves(square: Square, occupancy: BitBoard, want: BitBoard) {
        let masks_list = MasksList::new();
        let rank_atks = calc_rank_atks();
        let hq = HyperbolaQuintessence::new(masks_list, rank_atks);

        let got = hq.get_rank_moves(occupancy, square);
        assert_eq!(got, want);
    }
}
