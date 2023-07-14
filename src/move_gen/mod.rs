use crate::position::{Piece,Side, Sides, Pieces,Position};
use crate::bitboard::{BitBoard,Square, Move};

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
    fn gen_moves(&self, piece: Piece, square: Square, side: Side, opp_pieces: BitBoard, en_passant_target: Option<Square>) -> BitBoard;
}

pub trait GenerateSlidingMoves {
    fn gen_moves(&self, piece: Piece, square: Square, occupancy: BitBoard) -> BitBoard;
}

pub struct AllPiecesMoveGen {
    leaping_pieces: Box<dyn GenerateLeapingMoves>,
    sliding_pieces: Box<dyn GenerateSlidingMoves>
}

impl AllPiecesMoveGen {
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
                    Piece::Pawn | Piece::Knight | Piece::King => self.leaping_pieces.gen_moves(piece_type, piece_square, side, opp_pieces, position.state.en_passant_target),
                    Piece::Bishop | Piece::Rook | Piece::Queen => self.sliding_pieces.gen_moves(piece_type, piece_square, occupancy)
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
}
