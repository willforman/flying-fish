use crate::position::{Piece,Side,Sides,Pieces,Position};
use crate::bitboard::{BitBoard,Square,Move};

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

    fn gen_pawn_pushes(&self, square: Square, side: Side, opp_pieces: BitBoard) -> BitBoard;

    fn gen_pawn_atks(&self, square: Square, side: Side, opp_pieces: BitBoard, maybe_en_passant_target: Option<Square>) -> BitBoard;
}

pub trait GenerateSlidingMoves {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard;
}

pub struct AllPiecesMoveGen {
    leaping_pieces: Box<dyn GenerateLeapingMoves>,
    sliding_pieces: Box<dyn GenerateSlidingMoves>
}

impl AllPiecesMoveGen {
    // TODO: explore passing in as unboxed
    pub fn new(leaping_pieces: Box<dyn GenerateLeapingMoves>, sliding_pieces: Box<dyn GenerateSlidingMoves>) -> Self {
        AllPiecesMoveGen { leaping_pieces, sliding_pieces }
    }

    pub fn gen_moves(&self, position: &Position) -> HashSet<Move> {
        let mut moves = HashSet::new();

        let side = position.state.to_move;

        let friendly_pieces = position.sides.get(side);
        let opp_pieces = position.sides.get(side.opposite_side());

        let occupancy = position.sides.get(Side::White) | position.sides.get(Side::Black);

        for piece_type in Piece::iter() {
            let pieces = position.pieces.get(piece_type).get(side);

            for piece_square in pieces.to_squares() {
                let moves_bb = match piece_type {
                    Piece::Knight | Piece::King => self.leaping_pieces.gen_knight_king_moves(piece_type, piece_square),
                    Piece::Bishop | Piece::Rook | Piece::Queen => self.sliding_pieces.gen_moves(piece_type, piece_square, occupancy),
                    Piece::Pawn => {
                        self.leaping_pieces.gen_pawn_pushes(piece_square, side, opp_pieces) |
                        self.leaping_pieces.gen_pawn_atks(piece_square, side, opp_pieces, position.state.en_passant_target)
                    },
                };

                let moves_bb = moves_bb & !friendly_pieces; // Don't let capture pieces on their own team

                let moves_list: Vec<Move> = moves_bb.to_squares().iter()
                    .map(|&sq| Move { src: piece_square, dest: sq })
                    .collect();

                moves.extend(moves_list);
            }
        }

        moves
    }

    pub fn gen_attacked_squares(&self, position: &Position, side: Side) -> BitBoard {
        let opp_side = side.opposite_side();
        let friendly_pieces = position.sides.get(side);

        // Get occupancy but exclude king to handle kings moving away from checking sliding piece
        let occupancy = position.sides.get(side) & !position.pieces.get(Piece::King).get(opp_side);

        let mut attacked_squares = BitBoard::empty();

        for piece_type in Piece::iter() {
            let pieces = position.pieces.get(piece_type).get(side);
            
            for piece_square in pieces.to_squares() {
                let moves_bb = match piece_type {
                    Piece::Knight | Piece::King => self.leaping_pieces.gen_knight_king_moves(piece_type, piece_square),
                    Piece::Bishop | Piece::Rook | Piece::Queen => self.sliding_pieces.gen_moves(piece_type, piece_square, occupancy),
                    Piece::Pawn => self.leaping_pieces.gen_pawn_atks(piece_square, side, BitBoard::full(), None),
                };

                let moves_bb = moves_bb & !friendly_pieces; // Don't let capture pieces on their own team

                attacked_squares |= moves_bb;
            }
        }
        attacked_squares
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;
    use test_case::test_case;

    use crate::move_gen::leaping_pieces::LeapingPiecesMoveGen;
    use crate::move_gen::hyperbola_quintessence::HyperbolaQuintessence;

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
    ]))]
    fn test_gen_moves(position: Position, want: HashSet<Move>) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        let got = move_gen.gen_moves(&position);

        assert_eq!(got, want);
    }

    #[test_case(Position::start(), Side::White, BitBoard::from_squares(&[A3, B3, C3, D3, E3, F3, G3, H3]))]
    fn test_gen_attacked_squares(position: Position, side: Side, want: BitBoard) {
        let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
        let sliding_pieces = Box::new(HyperbolaQuintessence::new());
        let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

        let got = move_gen.gen_attacked_squares(&position, side);

        assert_eq!(got, want);
    }
}
