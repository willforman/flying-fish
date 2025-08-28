use crate::bitboard::{BitBoard, Direction, Square};

pub(super) enum MaskType {
    Bit,
    File,
    Rank,
    Diagonal,
    AntiDiagonal,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SquareMasks {
    pub(super) bit: BitBoard,
    pub(super) file: BitBoard,
    pub(super) rank: BitBoard,
    pub(super) diag: BitBoard,
    pub(super) anti_diag: BitBoard,
}

impl SquareMasks {
    const fn empty() -> SquareMasks {
        SquareMasks {
            bit: BitBoard::empty(),
            file: BitBoard::empty(),
            rank: BitBoard::empty(),
            diag: BitBoard::empty(),
            anti_diag: BitBoard::empty(),
        }
    }

    pub(super) const fn get(&self, mask_type: MaskType) -> BitBoard {
        match mask_type {
            MaskType::Bit => self.bit,
            MaskType::File => self.file,
            MaskType::Rank => self.rank,
            MaskType::Diagonal => self.diag,
            MaskType::AntiDiagonal => self.anti_diag,
        }
    }
}

pub struct MasksList([SquareMasks; 64]);

impl MasksList {
    pub(super) const fn get(&self, square: Square) -> &SquareMasks {
        &self.0[square as usize]
    }
}

pub(super) const MASKS_LIST: MasksList = calc_masks_list();
pub(super) const RANK_ATKS: [u8; 64 * 8] = calc_rank_atks();

pub(super) const fn calc_masks_list() -> MasksList {
    let mut masks_list = [SquareMasks::empty(); 64];

    let mut idx: usize = 0;
    while idx < 64 {
        let sq = Square::from_repr(idx as u8).unwrap();
        let bit_mask = BitBoard::from_square(sq);

        masks_list[idx].bit = bit_mask;
        idx += 1;
    }

    let mut file: usize = 0;
    let mut curr_file = BitBoard::from_squares(&[
        Square::A1,
        Square::A2,
        Square::A3,
        Square::A4,
        Square::A5,
        Square::A6,
        Square::A7,
        Square::A8,
    ]);

    while file < 8 {
        let mut rank = 0;
        while rank < 8 {
            let idx = rank * 8 + file;
            let bit_mask = masks_list[idx].bit;
            let file_mask = curr_file.const_bit_and(bit_mask.const_bit_not());

            masks_list[idx].file = file_mask;
            rank += 1;
        }
        curr_file.shift(Direction::IncFile);
        file += 1;
    }

    let mut rank: usize = 0;
    let mut curr_rank = BitBoard::from_squares(&[
        Square::A1,
        Square::B1,
        Square::C1,
        Square::D1,
        Square::E1,
        Square::F1,
        Square::G1,
        Square::H1,
    ]);

    while rank < 8 {
        let mut file = 0;
        while file < 8 {
            let idx = rank * 8 + file;
            let bit_mask = masks_list[idx].bit;
            let rank_mask = curr_rank.const_bit_and(bit_mask.const_bit_not());

            masks_list[idx].rank = rank_mask;
            file += 1;
        }
        curr_rank.shift(Direction::IncRank);
        rank += 1;
    }

    let mut curr_anti_diag = BitBoard::from_squares(&[
        Square::A1,
        Square::B2,
        Square::C3,
        Square::D4,
        Square::E5,
        Square::F6,
        Square::G7,
        Square::H8,
    ]);
    let mut file = 0;

    while file < 8 {
        let mut idx = file;
        // Continue up the anti diagonal until we reach the last file
        while idx % 8 != 0 || idx == file {
            let bit_mask = masks_list[idx].bit;
            let anti_diag_mask = curr_anti_diag.const_bit_and(bit_mask.const_bit_not());

            masks_list[idx].anti_diag = anti_diag_mask;
            idx += 9;
        }
        curr_anti_diag.shift(Direction::IncFile);
        file += 1;
    }

    let mut rank = 1;
    let mut curr_anti_diag = BitBoard::from_squares(&[
        Square::A2,
        Square::B3,
        Square::C4,
        Square::D5,
        Square::E6,
        Square::F7,
        Square::G8,
    ]);
    while rank < 8 {
        let mut idx = rank * 8;
        // Continue up the anti diagonal until we reach the last rank
        while idx / 8 != 8 {
            let bit_mask = masks_list[idx].bit;
            let anti_diag_mask = curr_anti_diag.const_bit_and(bit_mask.const_bit_not());

            masks_list[idx].anti_diag = anti_diag_mask;
            idx += 9;
        }
        curr_anti_diag.shift(Direction::IncRank);
        rank += 1;
    }

    let mut curr_diag = BitBoard::from_squares(&[
        Square::A8,
        Square::B7,
        Square::C6,
        Square::D5,
        Square::E4,
        Square::F3,
        Square::G2,
        Square::H1,
    ]);
    let mut file = 0;

    while file < 8 {
        let mut idx: usize = 56 + file;
        // Continue up the anti diagonal until we reach the last file
        loop {
            let _sq = Square::from_repr(idx as u8).unwrap();
            let bit_mask = masks_list[idx].bit;
            let diag_mask = curr_diag.const_bit_and(bit_mask.const_bit_not());

            masks_list[idx].diag = diag_mask;
            if idx % 8 == 7 {
                break;
            }
            idx -= 7;
        }
        curr_diag.shift(Direction::IncFile);
        file += 1;
    }

    let mut file = 6;
    let mut curr_diag = BitBoard::from_squares(&[
        Square::G1,
        Square::F2,
        Square::E3,
        Square::D4,
        Square::C5,
        Square::B6,
        Square::A7,
    ]);
    loop {
        let mut idx = file;
        // Continue up the anti diagonal until we pass the last rank
        loop {
            let bit_mask = masks_list[idx].bit;
            let diag_mask = curr_diag.const_bit_and(bit_mask.const_bit_not());

            masks_list[idx].diag = diag_mask;
            if idx % 8 == 0 {
                break;
            }
            idx += 7;
        }
        if file == 0 {
            break;
        }
        curr_diag.shift(Direction::DecFile);
        file -= 1;
    }

    MasksList(masks_list)
}

// o^(o-2r) trick
pub(super) const fn calc_left_rank_atk(blocking_pieces: u8, rook: u8) -> u8 {
    let occ = blocking_pieces | rook;
    occ ^ (blocking_pieces.wrapping_sub(rook))
}

const fn calc_rank_atks() -> [u8; 64 * 8] {
    let mut rank_atks_list: [u8; 64 * 8] = [0; 64 * 8];

    let mut pieces = 0;
    while pieces < 64 {
        let mut rook_shift = 0;
        while rook_shift < 8 {
            let rook = 1 << rook_shift;

            let shifted_pieces = pieces << 1; // Ignore the first and last bit

            let left_atks = calc_left_rank_atk(shifted_pieces, rook);
            let right_atks = calc_left_rank_atk(shifted_pieces.reverse_bits(), rook.reverse_bits())
                .reverse_bits();

            let atks = left_atks | right_atks;
            rank_atks_list[pieces as usize * 8 + rook_shift] = atks;

            rook_shift += 1;
        }
        pieces += 1;
    }
    rank_atks_list
}

/// Splits a bishop ray into the diagonal and anti diagonal components.
pub(crate) fn split_bishop_ray(bishop_ray: BitBoard, start_square: Square) -> (BitBoard, BitBoard) {
    let diag = bishop_ray & MASKS_LIST.get(start_square).get(MaskType::Diagonal);
    let anti_diag = bishop_ray & MASKS_LIST.get(start_square).get(MaskType::AntiDiagonal);
    (diag, anti_diag)
}

#[cfg(test)]
mod tests {
    use super::Square::*;
    use super::*;
    use test_case::test_case;

    #[test_case(MaskType::Bit, D4, BitBoard::from_square(D4) ; "bit")]
    #[test_case(MaskType::File, A8, BitBoard::from_squares(&[A7, A6, A5, A4, A3, A2, A1]) ; "file corner")]
    #[test_case(MaskType::File, D4, BitBoard::from_squares(&[D8, D7, D6, D5, D3, D2, D1]) ; "file middle")]
    #[test_case(MaskType::Rank, A8, BitBoard::from_squares(&[B8, C8, D8, E8, F8, G8, H8]) ; "rank corner")]
    #[test_case(MaskType::Rank, D4, BitBoard::from_squares(&[A4, B4, C4, E4, F4, G4, H4]) ; "rank middle")]
    #[test_case(MaskType::Diagonal, A8, BitBoard::from_squares(&[B7, C6, D5, E4, F3, G2, H1]) ; "diagonal main corner")]
    #[test_case(MaskType::Diagonal, E5, BitBoard::from_squares(&[B8, C7, D6, F4, G3, H2]) ; "diagonal middle")]
    #[test_case(MaskType::Diagonal, D4, BitBoard::from_squares(&[A7, B6, C5, E3, F2, G1]) ; "diagonal off main")]
    #[test_case(MaskType::Diagonal, A1, BitBoard::from_squares(&[]) ; "diagonal empty")]
    #[test_case(MaskType::AntiDiagonal, H8, BitBoard::from_squares(&[G7, F6, E5, D4, C3, B2, A1]) ; "anti diagonal main")]
    #[test_case(MaskType::AntiDiagonal, D5, BitBoard::from_squares(&[G8, F7, E6, C4, B3, A2]) ; "anti diagonal off main")]
    #[test_case(MaskType::AntiDiagonal, A8, BitBoard::from_squares(&[]) ; "anti diagonal empty")]
    fn test_mask(mask_type: MaskType, check_square: Square, want: BitBoard) {
        let masks = calc_masks_list();
        let got = masks.get(check_square).get(mask_type);
        assert_eq!(got, want);
    }
}
