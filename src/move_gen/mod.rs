use crate::position::{Piece,Side,Sides,Pieces,Position, SLIDING_PIECES};
use crate::bitboard::{BitBoard,Square,Move, Direction};
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


    fn get_pin_rays(&self, position: &Position, side: Side) -> BitBoard {
        const PIN_RAY_PIECE_CHECKS: [Piece; 2] = [Piece::Bishop, Piece::Rook];

        let mut pin_rays = BitBoard::empty();
        let opp_side = side.opposite_side();

        let pinner_occupancy = position.pieces.get(Piece::King).get(side);
        let king_square = pinner_occupancy.get_lsb();
        let king_ray_occupancy = position.sides.get(opp_side);

        for pin_ray_piece in PIN_RAY_PIECE_CHECKS {
            let king_ray = self.sliding_pieces.gen_moves(pin_ray_piece, king_square, king_ray_occupancy);

            let possible_pinners = position.pieces.get(pin_ray_piece).get(opp_side) |
                position.pieces.get(Piece::Queen).get(opp_side);
            let pinners = king_ray & possible_pinners;

            for pinner_square in pinners.to_squares() {
                let mut moves = self.sliding_pieces.gen_moves(pin_ray_piece, pinner_square, pinner_occupancy);
                moves.set_square(pinner_square); // Want to include capturing pinner in ray
                pin_rays |= moves & king_ray;
            }
        }

        pin_rays
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

        let pin_rays = self.get_pin_rays(position, side);

        // If the king has more than one checker, than the only legal moves are to move the king
        if num_checkers > 1 {
            let king_square = position.pieces.get(Piece::King).get(side).get_lsb();
            let mut moves_bb = self.gen_king_moves(position, side, king_square, friendly_pieces);
            moves_bb &= !friendly_pieces;
            let moves: HashSet<Move> = moves_bb.to_squares().iter()
                .map(|&sq| Move { src: king_square, dest: sq} )
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

                if pin_rays.is_square_set(piece_square) {
                    moves_bb &= pin_rays;
                }

                let moves_list: Vec<Move> = moves_bb.to_squares().iter()
                    .map(|&sq| Move { src: piece_square, dest: sq })
                    .collect();

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
                Piece::Pawn => self.leaping_pieces.gen_pawn_atks(king_square, opp_side),
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

    use crate::move_gen::leaping_pieces::LeapingPiecesMoveGen;
    use crate::move_gen::hyperbola_quintessence::HyperbolaQuintessence;

    macro_rules! assert_empty {
        ($container:expr) => {
            if !$container.is_empty() {
                panic!("expected {} to be empty, but got: {:?}", stringify!($container), $container);
            }
        };
    }

    #[test_case(Position::start(), HashSet::from_iter([
        Move { src: A2, dest: A3 }, Move { src: A2, dest: A4 },
        Move { src: B2, dest: B3 }, Move { src: B2, dest: B4 },
        Move { src: C2, dest: C3 }, Move { src: C2, dest: C4 },
        Move { src: D2, dest: D3 }, Move { src: D2, dest: D4 },
        Move { src: E2, dest: E3 }, Move { src: E2, dest: E4 },
        Move { src: F2, dest: F3 }, Move { src: F2, dest: F4 },
        Move { src: G2, dest: G3 }, Move { src: G2, dest: G4 },
        Move { src: H2, dest: H3 }, Move { src: H2, dest: H4 },
        Move { src: B1, dest: A3 }, Move { src: B1, dest: C3 },
        Move { src: G1, dest: F3 }, Move { src: G1, dest: H3 }
    ]))]
    #[test_case(Position::from_fen("8/8/p7/1p1p4/1P6/P1P3kp/5p2/1b5K w - - 0 51").unwrap(), HashSet::from_iter([
        Move { src: C3, dest: C4 }, Move { src: A3, dest: A4 },
    ]) ; "random position from my game")]
    #[test_case(Position::from_fen("8/8/8/8/k2Pp3/8/8/7K b - d3 0 1").unwrap(), HashSet::from_iter([
        Move { src: A4, dest: A5 }, Move { src: A4, dest: B5 },
        Move { src: A4, dest: A3 }, Move { src: A4, dest: B3 },
        Move { src: A4, dest: B4 },
        Move { src: E4, dest: E3 }, Move { src: E4, dest: D3 },
    ]) ; "en passant")]
    #[test_case(Position::from_fen("8/8/4k3/8/8/4R3/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E6, dest: D7 }, Move { src: E6, dest: F7 },
        Move { src: E6, dest: D6 }, Move { src: E6, dest: F6 },
        Move { src: E6, dest: D5 }, Move { src: E6, dest: F5 },
    ]) ; "king cant move into check")]
    #[test_case(Position::from_fen("8/8/4k3/8/5N2/8/3b4/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E6, dest: E7 }, Move { src: E6, dest: E5 },
        Move { src: E6, dest: D7 }, Move { src: E6, dest: F7 },
        Move { src: E6, dest: D6 }, Move { src: E6, dest: F6 },
        Move { src: E6, dest: F5 }, Move { src: D2, dest: F4 },
    ]) ; "capture checker")]
    #[test_case(Position::from_fen("k7/6r1/8/8/8/R7/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: A8, dest: B8 }, Move { src: A8, dest: B7 },
        Move { src: G7, dest: A7 },
    ]) ; "block checker")]
    #[test_case(Position::from_fen("8/8/4k3/6N1/8/4R3/3b4/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E6, dest: D6 }, Move { src: E6, dest: F6 },
        Move { src: E6, dest: D5 }, Move { src: E6, dest: F5 },
        Move { src: E6, dest: D7 },
    ]) ; "double check")]
    #[test_case(Position::from_fen("8/8/8/2k5/3Pp3/8/8/7K b - d3 0 1").unwrap(), HashSet::from_iter([
        Move { src: C5, dest: B6 }, Move { src: C5, dest: D6 },
        Move { src: C5, dest: B5 }, Move { src: C5, dest: D5 },
        Move { src: C5, dest: B4 }, Move { src: C5, dest: D4 },
        Move { src: C5, dest: C6 }, Move { src: C5, dest: C4 },
        Move { src: E4, dest: D3 },
    ]) ; "en passant capture to end check")]
    #[test_case(Position::from_fen("7k/8/7r/8/7Q/8/8/K7 b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: H8, dest: G7 }, Move { src: H8, dest: H7 },
        Move { src: H8, dest: G8 },
        Move { src: H6, dest: H7}, Move { src: H6, dest: H5 },
        Move { src: H6, dest: H4},
    ]) ; "cant move out of pin file")]
    #[test_case(Position::from_fen("k7/1r6/8/3Q4/8/8/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: A8, dest: B8 }, Move { src: A8, dest: A7 },
    ]) ; "cant move out of pin diagonal")]
    #[test_case(Position::from_fen("8/8/8/8/k2Pp2R/8/8/7K b - - 0 1").unwrap(), HashSet::from_iter([
        Move { src: A4, dest: A5 }, Move { src: A4, dest: B5 },
        Move { src: A4, dest: A3 }, Move { src: A4, dest: B3 },
        Move { src: A4, dest: B4 },
        Move { src: E4, dest: E3 },
    ]) ; "prevent en passant discovered check")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/8/P6P/R3K2R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E1, dest: F1 }, Move { src: E1, dest: D1 },
        Move { src: E1, dest: F2 }, Move { src: E1, dest: D2 },
        Move { src: E1, dest: E2 },
        Move { src: E1, dest: G1 }, Move { src: E1, dest: C1 }, // Castling
        Move { src: A1, dest: B1 }, Move { src: A1, dest: C1 },
        Move { src: A1, dest: D1 }, Move { src: H1, dest: G1 },
        Move { src: H1, dest: F1 },
        Move { src: A2, dest: A3 }, Move { src: A2, dest: A4 },
        Move { src: H2, dest: H3 }, Move { src: H2, dest: H4 },
    ]) ; "white castling")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/3bb3/P6P/R3K2R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E1, dest: D1 },
        Move { src: A1, dest: B1 }, Move { src: A1, dest: C1 },
        Move { src: A1, dest: D1 }, Move { src: H1, dest: G1 },
        Move { src: H1, dest: F1 },
        Move { src: A2, dest: A3 }, Move { src: A2, dest: A4 },
        Move { src: H2, dest: H3 }, Move { src: H2, dest: H4 },
    ]) ; "white castling cant through check")]
    #[test_case(Position::from_fen("4k3/8/8/8/8/8/P6P/R1N1KB1R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E1, dest: D1 },
        Move { src: E1, dest: F2 }, Move { src: E1, dest: D2 },
        Move { src: E1, dest: E2 },
        Move { src: A1, dest: B1 },
        Move { src: H1, dest: G1 },
        Move { src: A2, dest: A3 }, Move { src: A2, dest: A4 },
        Move { src: H2, dest: H3 }, Move { src: H2, dest: H4 },
        Move { src: F1, dest: G2 }, Move { src: F1, dest: H3 },
        Move { src: F1, dest: E2 }, Move { src: F1, dest: D3 },
        Move { src: F1, dest: C4 }, Move { src: F1, dest: B5 },
        Move { src: F1, dest: A6 },
        Move { src: C1, dest: B3 }, Move { src: C1, dest: D3 },
        Move { src: C1, dest: E2 }
    ]) ; "white castling cant through pieces")]
    #[test_case(Position::from_fen("4k3/8/8/8/1b6/8/P6P/R3K2R w KQ - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E1, dest: F1 }, Move { src: E1, dest: D1 },
        Move { src: E1, dest: F2 }, Move { src: E1, dest: E2 },
    ]) ; "white cant castle while in check")]
    #[test_case(Position::from_fen("r3k2r/p6p/8/8/8/8/8/4K3 b kq - 0 1").unwrap(), HashSet::from_iter([
        Move { src: E8, dest: F8 }, Move { src: E8, dest: D8 },
        Move { src: E8, dest: F7 }, Move { src: E8, dest: D7 },
        Move { src: E8, dest: E7 },
        Move { src: E8, dest: G8 }, Move { src: E8, dest: C8 }, // Castling
        Move { src: A8, dest: B8 }, Move { src: A8, dest: C8 },
        Move { src: A8, dest: D8 }, Move { src: H8, dest: G8 },
        Move { src: H8, dest: F8 },
        Move { src: A7, dest: A6 }, Move { src: A7, dest: A5 },
        Move { src: H7, dest: H6 }, Move { src: H7, dest: H5 },
    ]) ; "black castling")]
    #[test_case(Position::from_fen("r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4").unwrap(), HashSet::from_iter([]) ; "checkmate")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0").unwrap(), HashSet::from_iter([
        Move { src: A2, dest: A3 }, Move { src: A2, dest: A4 },
        Move { src: B2, dest: B3 }, Move { src: G2, dest: G3 },
        Move { src: D5, dest: D6 }, Move { src: D5, dest: E6 },
        Move { src: G2, dest: G4 }, Move { src: G2, dest: H3 },
        Move { src: C3, dest: A4 }, Move { src: C3, dest: B5 },
        Move { src: C3, dest: B1 }, Move { src: C3, dest: D1 },
        Move { src: E5, dest: C6 }, Move { src: E5, dest: G6 },
        Move { src: E5, dest: D7 }, Move { src: E5, dest: F7 },
        Move { src: E5, dest: C4 }, Move { src: E5, dest: G4 },
        Move { src: E5, dest: D3 }, Move { src: D2, dest: C1 },
        Move { src: D2, dest: E3 }, Move { src: D2, dest: F4 },
        Move { src: D2, dest: G5 }, Move { src: D2, dest: H6 },
        Move { src: E2, dest: D1 }, Move { src: E2, dest: F1 },
        Move { src: E2, dest: D3 }, Move { src: E2, dest: C4 },
        Move { src: E2, dest: B5 }, Move { src: E2, dest: A6 },
        Move { src: A1, dest: B1 }, Move { src: A1, dest: C1 },
        Move { src: A1, dest: D1 }, Move { src: H1, dest: G1 },
        Move { src: H1, dest: F1 }, Move { src: F3, dest: E3 },
        Move { src: F3, dest: D3 }, Move { src: F3, dest: G3 },
        Move { src: F3, dest: H3 }, Move { src: F3, dest: F4 },
        Move { src: F3, dest: F5 }, Move { src: F3, dest: F6 },
        Move { src: F3, dest: G4 }, Move { src: F3, dest: H5 },
        Move { src: E1, dest: D1 }, Move { src: E1, dest: C1 },
        Move { src: E1, dest: F1 }, Move { src: E1, dest: G1 },
    ]) ; "kiwipete")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1").unwrap(), HashSet::from_iter([
        Move { src: A8, dest: B8 }, Move { src: A8, dest: C8 },
        Move { src: A8, dest: D8 }, Move { src: E8, dest: C8 },
        Move { src: E8, dest: D8 }, Move { src: E8, dest: F8 },
        Move { src: E8, dest: G8 }, Move { src: H8, dest: G8 },
        Move { src: H8, dest: F8 }, Move { src: C7, dest: C6 },
        Move { src: C7, dest: C5 }, Move { src: D7, dest: D6 },
        Move { src: E7, dest: D8 }, Move { src: E7, dest: F8 },
        Move { src: E7, dest: D6 }, Move { src: E7, dest: C5 },
        Move { src: G7, dest: F8 }, Move { src: G7, dest: H6 },
        Move { src: A6, dest: C8 }, Move { src: A6, dest: B7 },
        Move { src: A6, dest: B5 }, Move { src: A6, dest: C4 },
        Move { src: A6, dest: D3 }, Move { src: A6, dest: E2 },
        Move { src: B6, dest: A4 }, Move { src: B6, dest: C4 },
        Move { src: B6, dest: C8 }, Move { src: B6, dest: D5 },
        Move { src: E6, dest: D5 }, Move { src: F6, dest: G8 },
        Move { src: F6, dest: H7 }, Move { src: F6, dest: D5 },
        Move { src: F6, dest: H5 }, Move { src: F6, dest: E4 },
        Move { src: F6, dest: G4 }, Move { src: G6, dest: G5 },
        Move { src: B4, dest: A3 }, Move { src: B4, dest: B3 },
        Move { src: B4, dest: C3 }, Move { src: H3, dest: G2 },
        Move { src: H8, dest: H7 }, Move { src: H8, dest: H6 },
        Move { src: H8, dest: H5 }, Move { src: H8, dest: H4 },
    ]) ; "kiwipete depth 2")]
    fn test_gen_moves(position: Position, want: HashSet<Move>) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        let got = move_gen.gen_moves(&position);

        let in_got_not_want: HashSet<_> = got.difference(&want).collect();
        assert_empty!(in_got_not_want);

        let in_want_not_got: HashSet<_> = want.difference(&got).collect();
        assert_empty!(in_want_not_got);
    }

    #[test_case(
        Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(),
        Vec::from([
            Move { src: E1, dest: F1 },
            Move { src: H3, dest: G2 },
        ]),
        HashSet::from_iter([
            Move { src: F1, dest: G1 },
            Move { src: F1, dest: G2 },
            Move { src: F1, dest: E1 },
            Move { src: F3, dest: G2 },
        ]) ; "kiwipete pawn check"
    )]
    fn test_gen_moves_from_moves(mut start_position: Position, moves_to_make: Vec<Move>, want: HashSet<Move>) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        for mve_to_make in moves_to_make {
            start_position.make_move(mve_to_make);
        }

        let got = move_gen.gen_moves(&start_position);
        println!("{:?}", move_gen.get_checkers(&start_position));

        let in_got_not_want: HashSet<_> = got.difference(&want).collect();
        assert_empty!(in_got_not_want);

        let in_want_not_got: HashSet<_> = want.difference(&got).collect();
        assert_empty!(in_want_not_got);

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

    #[test_case(Position::from_fen("6B1/8/4r3/3k4/2r5/1Q6/8/7K w - - 0 1").unwrap(), BitBoard::from_squares(&[B3, C4, E6, F7, G8]) ; "bishop")]
    #[test_case(Position::from_fen("8/8/8/3k1n1R/3n4/3Q4/8/7K w - - 0 1").unwrap(), BitBoard::from_squares(&[D3, D4, E5, F5, G5, H5]) ; "rook")]
    #[test_case(Position::from_fen("6B1/5N2/4r3/3k4/2r5/1Q6/8/7K w - - 0 1").unwrap(), BitBoard::from_squares(&[B3, C4]) ; "bishop block pin")]
    fn test_get_pin_rays(position: Position, want: BitBoard) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        let got = move_gen.get_pin_rays(&position, Side::Black);
        assert_eq!(got, want);
    }
}
