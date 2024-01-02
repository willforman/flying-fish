use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard, Direction, Square};
use crate::position::Piece;

use super::GenerateSlidingMoves;

// TODO: Look into using static (lazy_static) for this file

enum MaskType {
    Bit,
    File,
    Diagonal,
    AntiDiagonal,
}

#[derive(Debug, Clone, Copy)]
struct SquareMasks {
    bit: BitBoard,
    file: BitBoard,
    diag: BitBoard,
    anti_diag: BitBoard,
}

impl SquareMasks {
    const fn empty() -> SquareMasks {
        SquareMasks {
            bit: BitBoard::empty(),
            file: BitBoard::empty(),
            diag: BitBoard::empty(),
            anti_diag: BitBoard::empty(),
        }
    }

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
    fn get(&self, square: Square) -> &SquareMasks {
        &self.0[square as usize]
    }
}

static MASKS_LIST: MasksList = calc_masks_list();
static RANK_ATKS: [u8; 64 * 8] = calc_rank_atks();

pub struct HyperbolaQuintessence {}

// o^(o-2r) trick
const fn calc_left_rank_atk(blocking_pieces: u8, rook: u8) -> u8 {
    let occ = blocking_pieces | rook;
    let atks = occ ^ (blocking_pieces.wrapping_sub(rook));
    atks
}

impl HyperbolaQuintessence {
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
        let occupancy_without_self = occupancy & !BitBoard::from_square(square);
        let occ_val = occupancy_without_self.to_val();
        let sq_idx = square as u8;

        let file: usize = (sq_idx & 7).into(); // sq_idx % 8
        let rank_x8 = sq_idx & 56; // Rank times 8

        let rank_occ_x2: usize = ((occ_val >> rank_x8) & 2 * 63).try_into().unwrap(); // 2 times the inner six bit occupancy used as index
        let atks: u64 = RANK_ATKS[usize::from(4 * rank_occ_x2 + file)].into();

        BitBoard::from_val((atks << rank_x8).into())
    }
}

impl GenerateSlidingMoves for HyperbolaQuintessence {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard {
        let masks = MASKS_LIST.get(square);
        let bit_mask = masks.get(MaskType::Bit);

        match piece {
            Piece::Bishop => {
                self.get_moves(occupancy, masks.get(MaskType::Diagonal), bit_mask)
                    | self.get_moves(occupancy, masks.get(MaskType::AntiDiagonal), bit_mask)
            }
            Piece::Rook => {
                self.get_moves(occupancy, masks.get(MaskType::File), bit_mask)
                    | self.get_rank_moves(occupancy, square)
            }
            Piece::Queen => {
                self.get_moves(occupancy, masks.get(MaskType::File), bit_mask)
                    | self.get_rank_moves(occupancy, square)
                    | self.get_moves(occupancy, masks.get(MaskType::Diagonal), bit_mask)
                    | self.get_moves(occupancy, masks.get(MaskType::AntiDiagonal), bit_mask)
            }
            _ => panic!(
                "piece type: want [bishop, rook, queen], got {}",
                piece.to_string()
            ),
        }
    }
}

const fn calc_masks_list() -> MasksList {
    let mut masks_list = [SquareMasks::empty(); 64];
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
    let mut curr_diag = BitBoard::from_squares(&[Square::A1]);
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

    let mut reset_diag = curr_diag;
    let mut reset_anti_diag = curr_anti_diag;

    let mut idx: usize = 0;
    while idx < 64 {
        let sq = Square::from_repr(idx as u8).unwrap();
        let bit_mask = BitBoard::from_square(sq);
        masks_list[idx] = SquareMasks {
            bit: bit_mask,
            file: curr_file.const_bit_and(bit_mask.const_bit_not()),
            diag: curr_diag.const_bit_and(bit_mask.const_bit_not()),
            anti_diag: curr_anti_diag.const_bit_and(bit_mask.const_bit_not()),
        };
        // Add square at:
        // B7: A8
        // C7: B8
        if (idx + 1) % 8 == 0 {
            curr_file = curr_file.const_shr(7);

            reset_diag.shift(Direction::DecRank);
            curr_diag = reset_diag;

            reset_anti_diag.shift(Direction::IncRank);
            curr_anti_diag = reset_anti_diag;
        } else {
            curr_file.shift(Direction::IncFile);
            curr_diag.shift(Direction::IncFile);
            curr_anti_diag.shift(Direction::IncFile);
        }
        idx += 1;
    }
    MasksList(masks_list)
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

#[cfg(test)]
mod tests {
    use super::Square::*;
    use super::*;
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
        let got = MASKS_LIST.get(check_square).get(mask_type);
        assert_eq!(got, want);
    }

    #[test_case(H4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[A4, B4, C4, D4, E4, F4, G4]) ; "empty")]
    #[test_case(D4, BitBoard::from_squares(&[B4]), BitBoard::from_squares(&[B4, C4, E4, F4, G4, H4]) ; "one side")]
    #[test_case(D4, BitBoard::from_squares(&[A4, B4]), BitBoard::from_squares(&[B4, C4, E4, F4, G4, H4]) ; "one side irrelevant blocker")]
    #[test_case(B4, BitBoard::from_squares(&[D4, E4, H4]), BitBoard::from_squares(&[A4, C4, D4]) ; "one side multiple irrelevant blocker")]
    #[test_case(D4, BitBoard::from_squares(&[A4, F4]), BitBoard::from_squares(&[A4, B4, C4, E4, F4]) ; "both sides")]
    #[test_case(D4, BitBoard::from_squares(&[C4, E4]), BitBoard::from_squares(&[C4, E4]) ; "both sides close")]
    fn test_gen_rank_moves(square: Square, occupancy: BitBoard, want: BitBoard) {
        let hq = HyperbolaQuintessence {};

        let got = hq.get_rank_moves(occupancy, square);
        assert_eq!(got, want);
    }

    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[A1, B2, C3, E5, F6, G7, H8, C5, B6, A7, E3, F2, G1]) ; "bishop no blockers")]
    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[B2, A7, E5]), BitBoard::from_squares(&[B2, C3, E5, C5, B6, A7, E3, F2, G1]) ; "bishop many blockers")]
    #[test_case(Piece::Bishop, D4, BitBoard::from_squares(&[B2, A7, E5, A1, B1, F8, G6, C4]), BitBoard::from_squares(&[B2, C3, E5, C5, B6, A7, E3, F2, G1]) ; "bishop irrelevant blockers")]
    #[test_case(Piece::Rook, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[D1, D2, D3, D5, D6, D7, D8, A4, B4, C4, E4, F4, G4, H4]) ; "rook no blockers")]
    #[test_case(Piece::Rook, D4, BitBoard::from_squares(&[A4, D7, F4, D3]), BitBoard::from_squares(&[D3, D5, D6, D7, A4, B4, C4, E4, F4]) ; "rook blockers")]
    #[test_case(Piece::Rook, D4, BitBoard::from_squares(&[A4, D7, D8, F4, D3, D2, D1]), BitBoard::from_squares(&[D3, D5, D6, D7, A4, B4, C4, E4, F4]) ; "rook irrelevant blockers")]
    #[test_case(Piece::Rook, E3, BitBoard::from_squares(&[E3]), BitBoard::from_squares(&[E1, E2, E4, E5, E6, E7, E8, A3, B3, C3, D3, F3, G3, H3]) ; "rook irrelevant blockers 2")]
    #[test_case(Piece::Queen, D4, BitBoard::from_squares(&[]), BitBoard::from_squares(&[A1, B2, C3, E5, F6, G7, H8, C5, B6, A7, E3, F2, G1, D1, D2, D3, D5, D6, D7, D8, A4, B4, C4, E4, F4, G4, H4]) ; "queen no blockers")]
    #[test_case(Piece::Queen, D4, BitBoard::from_squares(&[D5, B2, H4]), BitBoard::from_squares(&[B2, C3, E5, F6, G7, H8, C5, B6, A7, E3, F2, G1, D1, D2, D3, D5, A4, B4, C4, E4, F4, G4, H4]) ; "queen blockers")]
    fn test_gen_moves(piece: Piece, square: Square, occupancy: BitBoard, want: BitBoard) {
        let hq = HyperbolaQuintessence {};

        let got = hq.gen_moves(piece, square, occupancy);
        assert_eq!(got, want);
    }
}
