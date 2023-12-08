use crate::position::{Piece,Side,Position,Move};
use crate::bitboard::{BitBoard,Square,Direction};
use crate::bitboard::Square::*;

use std::collections::HashSet;

use strum::IntoEnumIterator;

pub mod leaping_pieces;
pub mod hyperbola_quintessence;

#[derive(thiserror::Error, Debug)]
pub enum MoveGenError {
    #[error("no piece at {0}")]
    NoPiece(String),

    #[error("internal state error: set in sides {0} but not in pieces")]
    InvalidSidesPieces(String)
}

pub trait GenerateLeapingMoves {
    fn gen_knight_king_moves(&self, piece: Piece, square: Square) -> BitBoard;

    fn gen_pawn_pushes(&self, square: Square, side: Side) -> BitBoard;
    fn gen_pawn_atks(&self, square: Square, side: Side) -> BitBoard;
}

pub trait GenerateSlidingMoves {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard;
}

pub trait GenerateAllMoves {
    fn gen_moves(&self, position: &Position) -> HashSet<Move>;
    fn get_checkers(&self, position: &Position) -> BitBoard;
}

pub struct AllPiecesMoveGen {
    leaping_pieces: Box<dyn GenerateLeapingMoves>,
    sliding_pieces: Box<dyn GenerateSlidingMoves>
}

impl AllPiecesMoveGen {
    pub fn new(leaping_pieces: Box<dyn GenerateLeapingMoves>, sliding_pieces: Box<dyn GenerateSlidingMoves>) -> Self {
        AllPiecesMoveGen { leaping_pieces, sliding_pieces }
    }

    fn gen_king_moves(&self, position: &Position, side: Side, king_square: Square, friendly_pieces: BitBoard) -> BitBoard {
        let mut moves = self.leaping_pieces.gen_knight_king_moves(Piece::King, king_square);
        let king_danger_squares = self.gen_attacked_squares(position, side.opposite_side());
        moves &= !king_danger_squares;

        // Castling
        if !king_danger_squares.is_square_set(king_square) {
            if king_square == E1 { // White castling
                if position.state.castling_rights.white_king_side &&
                    !friendly_pieces.is_square_set(F1) && 
                    !friendly_pieces.is_square_set(G1) &&
                    !king_danger_squares.is_square_set(F1) &&
                    !king_danger_squares.is_square_set(G1) 
                {
                    moves.set_square(G1)
                }
                if position.state.castling_rights.white_queen_side &&
                    !friendly_pieces.is_square_set(D1) && 
                    !friendly_pieces.is_square_set(C1) &&
                    !king_danger_squares.is_square_set(D1) &&
                    !king_danger_squares.is_square_set(C1)
                {
                    moves.set_square(C1)
                }
            }
            if king_square == E8 { // Black castling
                if position.state.castling_rights.black_king_side &&
                    !friendly_pieces.is_square_set(F8) && 
                    !friendly_pieces.is_square_set(G8) &&
                    !king_danger_squares.is_square_set(F8) &&
                    !king_danger_squares.is_square_set(G8) 
                {
                    moves.set_square(G8)
                }
                if position.state.castling_rights.black_queen_side &&
                    !friendly_pieces.is_square_set(D8) && 
                    !friendly_pieces.is_square_set(C8) &&
                    !king_danger_squares.is_square_set(D8) &&
                    !king_danger_squares.is_square_set(C8)
                {
                    moves.set_square(C8)
                }
            }
        }
        moves
    }

    pub fn gen_attacked_squares(&self, position: &Position, side: Side) -> BitBoard {
        // Get occupancy but exclude king to handle kings moving away from checking sliding piece
        let occupancy = position.sides.get(Side::White) | position.sides.get(Side::Black) &
            !position.pieces.get(Piece::King).get(side.opposite_side());

        let mut attacked_squares = BitBoard::empty();

        for piece_type in Piece::iter() {
            let pieces = position.pieces.get(piece_type).get(side);
            
            for piece_square in pieces.to_squares() {
                let moves_bb = match piece_type {
                    Piece::Knight | Piece::King => self.leaping_pieces.gen_knight_king_moves(piece_type, piece_square),
                    Piece::Bishop | Piece::Rook | Piece::Queen => self.sliding_pieces.gen_moves(piece_type, piece_square, occupancy),
                    Piece::Pawn => self.leaping_pieces.gen_pawn_atks(piece_square, side),
                };

                attacked_squares |= moves_bb;
            }
        }
        attacked_squares
    }


    fn get_pin_rays(&self, position: &Position, side: Side) -> (BitBoard, BitBoard) {
        let opp_side = side.opposite_side();

        let pinner_occupancy = position.pieces.get(Piece::King).get(side);
        let king_square = pinner_occupancy.get_lsb();
        let king_ray_occupancy = position.sides.get(opp_side);

        // Rook pin ray
        let king_ray = self.sliding_pieces.gen_moves(Piece::Rook, king_square, king_ray_occupancy);

        let possible_pinners = position.pieces.get(Piece::Rook).get(opp_side) |
            position.pieces.get(Piece::Queen).get(opp_side);
        let pinners = king_ray & possible_pinners;

        let mut rook_pin_ray = BitBoard::empty();
        for pinner_square in pinners.to_squares() {
            let mut moves = self.sliding_pieces.gen_moves(Piece::Rook, pinner_square, pinner_occupancy);
            moves.set_square(pinner_square); // Want to include capturing pinner in ray
            let possible_pin_ray = moves & king_ray;
            // If there's multiple pieces in the ray, then there's no pin
            if (possible_pin_ray & position.sides.get(side)).num_squares_set() > 1 {
                continue;
            }
            rook_pin_ray |= moves & king_ray;
        }

        // Bishop pin ray
        let king_ray = self.sliding_pieces.gen_moves(Piece::Bishop, king_square, king_ray_occupancy);

        let possible_pinners = position.pieces.get(Piece::Bishop).get(opp_side) |
            position.pieces.get(Piece::Queen).get(opp_side);
        let pinners = king_ray & possible_pinners;

        let mut bishop_pin_ray = BitBoard::empty();
        for pinner_square in pinners.to_squares() {
            let mut moves = self.sliding_pieces.gen_moves(Piece::Bishop, pinner_square, pinner_occupancy);
            moves.set_square(pinner_square); // Want to include capturing pinner in ray
            let possible_pin_ray = moves & king_ray;
            // If there's multiple pieces in the ray, then there's no pin
            if (possible_pin_ray & position.sides.get(side)).num_squares_set() > 1 {
                continue;
            }
            bishop_pin_ray |= moves & king_ray;
        }

        (rook_pin_ray, bishop_pin_ray)
    }
}

impl GenerateAllMoves for AllPiecesMoveGen {
    fn gen_moves(&self, position: &Position) -> HashSet<Move> {
        let side = position.state.to_move;

        let friendly_pieces = position.sides.get(side);
        let opp_pieces = position.sides.get(side.opposite_side());

        let occupancy = friendly_pieces | opp_pieces;

        let checkers = self.get_checkers(position);
        let num_checkers = checkers.to_squares().len();

        // In the case of check, what squares are allowed to be captured and blocked
        let mut capture_mask = BitBoard::full();
        let mut push_mask = BitBoard::full();

        let (rook_pin_ray, bishop_pin_ray) = self.get_pin_rays(position, side);

        // If the king has more than one checker, than the only legal moves are to move the king
        if num_checkers > 1 {
            let king_square = position.pieces.get(Piece::King).get(side).get_lsb();
            let mut moves_bb = self.gen_king_moves(position, side, king_square, friendly_pieces);
            moves_bb &= !friendly_pieces;
            let moves: HashSet<Move> = moves_bb.to_squares().iter()
                .map(|&sq| Move::new(king_square, sq) )
                .collect();
            return moves;
        }

        if num_checkers == 1 {
            capture_mask = checkers;
            if let Some(ep_target) = position.state.en_passant_target {
                let ep_dir = if side.opposite_side() == Side::White { Direction::North } else { Direction::South };
                let ep_src_bb = BitBoard::from_square_shifts(ep_target, &vec![vec![ep_dir]]);
                if ep_src_bb == checkers {
                    capture_mask |= BitBoard::from_square(ep_target);
                }
            }

            let checker_square = checkers.get_lsb();
            let (checker_piece_type, _) = position.is_piece_at(checker_square).unwrap();
            push_mask = if checker_piece_type.is_slider() {
                let king_square = position.pieces.get(Piece::King).get(side).get_lsb();
                BitBoard::from_ray_excl(checker_square, king_square)
            } else {
                BitBoard::empty()
            }
        }

        let mut moves = HashSet::new();

        for piece_type in Piece::iter() {
            let pieces = position.pieces.get(piece_type).get(side);

            for piece_square in pieces.to_squares() {
                let mut moves_bb = match piece_type {
                    Piece::Knight => self.leaping_pieces.gen_knight_king_moves(Piece::Knight, piece_square),
                    Piece::King => self.gen_king_moves(position, side, piece_square, friendly_pieces),
                    Piece::Bishop | Piece::Rook | Piece::Queen => self.sliding_pieces.gen_moves(piece_type, piece_square, occupancy),
                    Piece::Pawn => {
                        let mut pushes = self.leaping_pieces.gen_pawn_pushes(piece_square, side);
                        pushes &= !opp_pieces; // Can't push into opposing piece

                        // This ensures that if a single push is blocked, then a double push isn't
                        // possible too
                        let mut all_pieces_except_self = opp_pieces | friendly_pieces;
                        all_pieces_except_self.clear_square(piece_square);
                        let shift_dir = if side == Side::White { Direction::North } else { Direction::South };
                        all_pieces_except_self.shift(shift_dir);
                        pushes &= !all_pieces_except_self;

                        let mut possible_atks = opp_pieces;
                        if let Some(ep_target) = position.state.en_passant_target {
                            possible_atks |= BitBoard::from_square(ep_target)
                        }

                        let atks = self.leaping_pieces.gen_pawn_atks(piece_square, side) & possible_atks;
                        pushes | atks
                    },
                };

                moves_bb &= !friendly_pieces; // Don't let capture pieces on their own team

                // If in check, make sure only capturing moves or blocking moves
                if piece_type != Piece::King {
                    moves_bb &= capture_mask | push_mask;
                }

                if rook_pin_ray.is_square_set(piece_square) {
                    moves_bb &= rook_pin_ray;
                }
                if bishop_pin_ray.is_square_set(piece_square) {
                    moves_bb &= bishop_pin_ray;
                }

                // For each promotion, we need to add 4 moves to the list,
                // 1 for each piece type
                let moves_list: Vec<Move> = if piece_type == Piece::Pawn && 
                    ((side == Side::White && (piece_square >= A7 && piece_square <= H7)) ||
                    (side == Side::Black && (piece_square >= A2 && piece_square <= H2)))
                {
                    moves_bb.to_squares().iter()
                        .flat_map(|&sq| {
                            [
                                Move::with_promotion(piece_square, sq, Piece::Knight),
                                Move::with_promotion(piece_square, sq, Piece::Bishop),
                                Move::with_promotion(piece_square, sq, Piece::Rook),
                                Move::with_promotion(piece_square, sq, Piece::Queen)
                            ]
                        })
                        .collect()
                } else {
                    moves_bb.to_squares().iter()
                        .map(|&sq| Move::new(piece_square, sq))
                        .collect()
                };

                moves.extend(moves_list);
            }
        }

        moves
    }

    fn get_checkers(&self, position: &Position) -> BitBoard {
        let side = position.state.to_move;
        let opp_side = side.opposite_side();

        let king_square = position.pieces.get(Piece::King).get(side).pop_lsb();
        let occupancy = position.sides.get(Side::White) | position.sides.get(Side::Black);

        let mut checkers = BitBoard::empty();

        for piece_type in Piece::iter() {
            let moves = match piece_type {
                Piece::Knight => self.leaping_pieces.gen_knight_king_moves(piece_type, king_square),
                Piece::Bishop | Piece::Rook | Piece::Queen => self.sliding_pieces.gen_moves(piece_type, king_square, occupancy),
                Piece::Pawn => self.leaping_pieces.gen_pawn_atks(king_square, side),
                Piece::King => BitBoard::empty() // Pass
            };
            let pieces = position.pieces.get(piece_type).get(opp_side);
            checkers |= moves & pieces;
        }

        checkers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use testresult::TestResult;

    use crate::move_gen::leaping_pieces::LeapingPiecesMoveGen;
    use crate::move_gen::hyperbola_quintessence::HyperbolaQuintessence;

    macro_rules! assert_eq_collections {
        ($coll_a:expr, $coll_b:expr) => {
            let set_a: HashSet<_> = HashSet::from_iter($coll_a.iter().cloned());
            let set_b: HashSet<_> = HashSet::from_iter($coll_b.iter().cloned());

            let diff_a_b: HashSet<_> = set_a.difference(&set_b).cloned().collect();
            let diff_b_a: HashSet<_> = set_b.difference(&set_a).cloned().collect();

            if !diff_a_b.is_empty() || !diff_b_a.is_empty() {
                panic!("collections don't have the same elements. \
                       \nin {} but not {}: {:?}. \
                       \nin {} but not {}: {:?}.", stringify!($coll_a), stringify!($coll_b), diff_a_b, stringify!($coll_b), stringify!($coll_a), diff_b_a);
            }

        }
    }

    #[test_case(Position::start(), HashSet::from_iter([
        Move::new(A2, A3), Move::new(A2, A4),
        Move::new(B2, B3), Move::new(B2, B4),
        Move::new(C2, C3), Move::new(C2, C4),
        Move::new(D2, D3), Move::new(D2, D4),
        Move::new(E2, E3), Move::new(E2, E4),
        Move::new(F2, F3), Move::new(F2, F4),
        Move::new(G2, G3), Move::new(G2, G4),
        Move::new(H2, H3), Move::new(H2, H4),
        Move::new(B1, A3), Move::new(B1, C3),
        Move::new(G1, F3), Move::new(G1, H3)
    ]))]
    #[test_case(Position::from_fen("8/8/p7/1p1p4/1P6/P1P3kp/5p2/1b5K w - - 0 51").unwrap(), HashSet::from_iter([
        Move::new(C3, C4), Move::new(A3, A4),
    ]) ; "random position from my game")]
    #[test_case(Position::from_fen("8/8/8/8/k2Pp3/8/8/7K b - d3 0 1").unwrap(), HashSet::from_iter([
        Move::new(A4, A5), Move::new(A4, B5),
        Move::new(A4, A3), Move::new(A4, B3),
        Move::new(A4, B4),
        Move::new(E4, E3), Move::new(E4, D3),
    ]) ; "en passant")]
    #[test_case(Position::from_fen("8/8/4k3/8/8/4R3/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E6, D7), Move::new(E6, F7),
        Move::new(E6, D6), Move::new(E6, F6),
        Move::new(E6, D5), Move::new(E6, F5),
    ]) ; "king cant move into check")]
    #[test_case(Position::from_fen("8/8/4k3/8/5N2/8/3b4/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E6, E7), Move::new(E6, E5),
        Move::new(E6, D7), Move::new(E6, F7),
        Move::new(E6, D6), Move::new(E6, F6),
        Move::new(E6, F5), Move::new(D2, F4),
    ]) ; "capture checker")]
    #[test_case(Position::from_fen("k7/6r1/8/8/8/R7/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(A8, B8), Move::new(A8, B7),
        Move::new(G7, A7),
    ]) ; "block checker")]
    #[test_case(Position::from_fen("8/8/4k3/6N1/8/4R3/3b4/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E6, D6), Move::new(E6, F6),
        Move::new(E6, D5), Move::new(E6, F5),
        Move::new(E6, D7),
    ]) ; "double check")]
    #[test_case(Position::from_fen("8/8/8/2k5/3Pp3/8/8/7K b - d3 0 1").unwrap(), HashSet::from_iter([
        Move::new(C5, B6), Move::new(C5, D6),
        Move::new(C5, B5), Move::new(C5, D5),
        Move::new(C5, B4), Move::new(C5, D4),
        Move::new(C5, C6), Move::new(C5, C4),
        Move::new(E4, D3),
    ]) ; "en passant capture to end check")]
    #[test_case(Position::from_fen("7k/8/7r/8/7Q/8/8/K7 b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(H8, G7), Move::new(H8, H7),
        Move::new(H8, G8),
        Move::new(H6, H7), Move::new(H6, H5),
        Move::new(H6, H4),
    ]) ; "cant move out of pin file")]
    #[test_case(Position::from_fen("k7/1r6/8/3Q4/8/8/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(A8, B8), Move::new(A8, A7),
    ]) ; "cant move out of pin diagonal")]
    #[test_case(Position::from_fen("8/8/8/8/k2Pp2R/8/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(A4, A5), Move::new(A4, B5),
        Move::new(A4, A3), Move::new(A4, B3),
        Move::new(A4, B4),
        Move::new(E4, E3),
    ]) ; "prevent en passant discovered check")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/8/P6P/R3K2R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E1, F1), Move::new(E1, D1),
        Move::new(E1, F2), Move::new(E1, D2),
        Move::new(E1, E2),
        Move::new(E1, G1), Move::new(E1, C1), // Castling
        Move::new(A1, B1), Move::new(A1, C1),
        Move::new(A1, D1), Move::new(H1, G1),
        Move::new(H1, F1),
        Move::new(A2, A3), Move::new(A2, A4),
        Move::new(H2, H3), Move::new(H2, H4),
    ]) ; "white castling")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/3bb3/P6P/R3K2R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E1, D1),
        Move::new(A1, B1), Move::new(A1, C1),
        Move::new(A1, D1), Move::new(H1, G1),
        Move::new(H1, F1),
        Move::new(A2, A3), Move::new(A2, A4),
        Move::new(H2, H3), Move::new(H2, H4),
    ]) ; "white castling cant through check")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/8/P6P/R1N1KB1R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E1, D1),
        Move::new(E1, F2), Move::new(E1, D2),
        Move::new(E1, E2),
        Move::new(A1, B1),
        Move::new(H1, G1),
        Move::new(A2, A3), Move::new(A2, A4),
        Move::new(H2, H3), Move::new(H2, H4),
        Move::new(F1, G2), Move::new(F1, H3),
        Move::new(F1, E2), Move::new(F1, D3),
        Move::new(F1, C4), Move::new(F1, B5),
        Move::new(F1, A6),
        Move::new(C1, B3), Move::new(C1, D3),
        Move::new(C1, E2)
    ]) ; "white castling cant through pieces")]
    #[test_case(Position::from_fen("4k3/8/8/8/1b6/8/P6P/R3K2R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E1, F1), Move::new(E1, D1),
        Move::new(E1, F2), Move::new(E1, E2),
    ]) ; "white cant castle while in check")]
    #[test_case(Position::from_fen("r3k2r/p6p/8/8/8/8/8/4K3 b kq - 0 1").unwrap(), HashSet::from_iter([
        Move::new(E8, F8), Move::new(E8, D8),
        Move::new(E8, F7), Move::new(E8, D7),
        Move::new(E8, E7),
        Move::new(E8, G8), Move::new(E8, C8), // Castling
        Move::new(A8, B8), Move::new(A8, C8),
        Move::new(A8, D8), Move::new(H8, G8),
        Move::new(H8, F8),
        Move::new(A7, A6), Move::new(A7, A5),
        Move::new(H7, H6), Move::new(H7, H5),
    ]) ; "black castling")]
    #[test_case(Position::from_fen("r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4").unwrap(), HashSet::from_iter([]) ; "checkmate")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0").unwrap(), HashSet::from_iter([
        Move::new(A2, A3), Move::new(A2, A4),
        Move::new(B2, B3), Move::new(G2, G3),
        Move::new(D5, D6), Move::new(D5, E6),
        Move::new(G2, G4), Move::new(G2, H3),
        Move::new(C3, A4), Move::new(C3, B5),
        Move::new(C3, B1), Move::new(C3, D1),
        Move::new(E5, C6), Move::new(E5, G6),
        Move::new(E5, D7), Move::new(E5, F7),
        Move::new(E5, C4), Move::new(E5, G4),
        Move::new(E5, D3), Move::new(D2, C1),
        Move::new(D2, E3), Move::new(D2, F4),
        Move::new(D2, G5), Move::new(D2, H6),
        Move::new(E2, D1), Move::new(E2, F1),
        Move::new(E2, D3), Move::new(E2, C4),
        Move::new(E2, B5), Move::new(E2, A6),
        Move::new(A1, B1), Move::new(A1, C1),
        Move::new(A1, D1), Move::new(H1, G1),
        Move::new(H1, F1), Move::new(F3, E3),
        Move::new(F3, D3), Move::new(F3, G3),
        Move::new(F3, H3), Move::new(F3, F4),
        Move::new(F3, F5), Move::new(F3, F6),
        Move::new(F3, G4), Move::new(F3, H5),
        Move::new(E1, D1), Move::new(E1, C1),
        Move::new(E1, F1), Move::new(E1, G1),
    ]) ; "kiwipete")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1").unwrap(), HashSet::from_iter([
        Move::new(A8, B8), Move::new(A8, C8),
        Move::new(A8, D8), Move::new(E8, C8),
        Move::new(E8, D8), Move::new(E8, F8),
        Move::new(E8, G8), Move::new(H8, G8),
        Move::new(H8, F8), Move::new(C7, C6),
        Move::new(C7, C5), Move::new(D7, D6),
        Move::new(E7, D8), Move::new(E7, F8),
        Move::new(E7, D6), Move::new(E7, C5),
        Move::new(G7, F8), Move::new(G7, H6),
        Move::new(A6, C8), Move::new(A6, B7),
        Move::new(A6, B5), Move::new(A6, C4),
        Move::new(A6, D3), Move::new(A6, E2),
        Move::new(B6, A4), Move::new(B6, C4),
        Move::new(B6, C8), Move::new(B6, D5),
        Move::new(E6, D5), Move::new(F6, G8),
        Move::new(F6, H7), Move::new(F6, D5),
        Move::new(F6, H5), Move::new(F6, E4),
        Move::new(F6, G4), Move::new(G6, G5),
        Move::new(B4, A3), Move::new(B4, B3),
        Move::new(B4, C3), Move::new(H3, G2),
        Move::new(H8, H7), Move::new(H8, H6),
        Move::new(H8, H5), Move::new(H8, H4),
    ]) ; "kiwipete depth 2")]
    #[test_case(Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap(), HashSet::from_iter([
        Move::new(G1, H1),
        Move::new(F1, F2),
        Move::new(F3, D4),
        Move::new(B4, C5),
        Move::new(C4, C5),
        Move::new(D2, D4),
    ]) ; "perft results position4")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/8/r4PPK/r7 w - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(H2, H3), Move::new(H2, G3),
        Move::new(G2, G3), Move::new(G2, G4),
        Move::new(F2, F3), Move::new(F2, F4),
    ]) ; "double pin")]
    #[test_case(Position::from_fen("k7/1b6/8/8/8/8/6R1/r6K w - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(H1, H2)
    ]) ; "move to another pin")]
    #[test_case(Position::from_fen("k7/8/8/8/8/8/6N1/2rR3K w - - 0 1").unwrap(), HashSet::from_iter([
        Move::new(D1, C1),
        Move::new(D1, E1),
        Move::new(D1, F1),
        Move::new(D1, G1),
        Move::new(G2, E1),
        Move::new(G2, E3),
        Move::new(G2, F4),
        Move::new(G2, H4),
        Move::new(H1, G1),
        Move::new(H1, H2),
    ]))]
    fn test_gen_moves(position: Position, want: HashSet<Move>) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        println!("{:?}", position);
        let got = move_gen.gen_moves(&position);

        assert_eq_collections!(got, want);
    }


    // #[test_case(
    //     Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap(),
    //     Vec::from([
    //         Move::new(E1, F1),
    //         Move::new(H3, G2),
    //     ]),
    //     HashSet::from_iter([
    //         Move::new(F1, G1),
    //         Move::new(F1, G2),
    //         Move::new(F1, E1),
    //         Move::new(F3, G2),
    //     ]) ; "perft results position4"
    // )]
    // #[test_case(
    //     Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(),
    //     Vec::from([
    //         Move::new(G1, H1),
    //         Move::new(B2, A1),
    //     ]),
    //     HashSet::from_iter([
    //         Move::new(F1, G1),
    //         Move::new(F1, G2),
    //         Move::new(F1, E1),
    //         Move::new(F3, G2),
    //     ]) ; "kiwipete pawn check"
    // )]
    fn test_gen_moves_from_moves(mut start_position: Position, moves_to_make: Vec<Move>, want: HashSet<Move>) -> TestResult {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        for mve_to_make in moves_to_make {
            start_position.make_move(&mve_to_make)?;
        }

        let got = move_gen.gen_moves(&start_position);

        assert_eq_collections!(got, want);
        Ok(())
    }

    #[test_case(Position::start(), Side::White, BitBoard::from_squares(&[
        B1, C1, D1, E1, F1, G1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3]))]
    fn test_gen_attacked_squares(position: Position, side: Side, want: BitBoard) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        let got = move_gen.gen_attacked_squares(&position, side);

        assert_eq!(got, want);
    }

    #[test_case(Position::from_fen("6B1/8/4r3/3k4/2r5/1Q6/8/7K w - - 0 1").unwrap(), BitBoard::empty(), BitBoard::from_squares(&[B3, C4, E6, F7, G8]) ; "bishop")]
    #[test_case(Position::from_fen("8/8/8/3k1n1R/3n4/3Q4/8/7K w - - 0 1").unwrap(), BitBoard::from_squares(&[D3, D4, E5, F5, G5, H5]), BitBoard::empty() ; "rook")]
    #[test_case(Position::from_fen("6B1/5N2/4r3/3k4/2r5/1Q6/8/7K w - - 0 1").unwrap(), BitBoard::empty(), BitBoard::from_squares(&[B3, C4]) ; "bishop block pin")]
    fn test_get_pin_rays(position: Position, want_rook_pin_ray: BitBoard, want_bishop_pin_ray: BitBoard) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        let (got_rook_pin_ray, got_bishop_pin_ray) = move_gen.get_pin_rays(&position, Side::Black);
        assert_eq!(got_rook_pin_ray, want_rook_pin_ray);
        assert_eq!(got_bishop_pin_ray, want_bishop_pin_ray);
    }
}
