use strum::IntoEnumIterator;

use super::masks::{MaskType, MASKS_LIST, RANK_ATKS};
use crate::bitboard::{BitBoard, Direction, Square};
use crate::position::Piece;

use super::traits::GenerateSlidingMoves;

#[derive(Clone, Copy)]
pub struct HyperbolaQuintessence {}

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

pub(crate) static HYPERBOLA_QUINTESSENCE: HyperbolaQuintessence = HyperbolaQuintessence {};

#[cfg(test)]
mod tests {
    use super::Square::*;
    use super::*;
    use test_case::test_case;

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
