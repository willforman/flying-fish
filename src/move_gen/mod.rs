use crate::position::{Piece,Side, Sides, Pieces};
use crate::bitboard::{BitBoard,Square};

use std::string::ToString;

use self::leaping_pieces::LeapingPiecesMoveGen;

use strum::IntoEnumIterator;

mod leaping_pieces;
mod hyperbola_quintessence;

#[derive(thiserror::Error, Debug)]
pub enum MoveGenError {
    #[error("no piece at {0}")]
    NoPiece(String),

    #[error("internal state error: set in sides {0} but not in pieces")]
    InvalidSidesPieces(String)
}

struct Move {
    src: Square,
    dest: Square,
}

trait GenerateLeapingMoves {
    fn gen_moves(&self, piece_type: Piece, square: Square, side: Side) -> BitBoard;
}

trait GenerateSlidingMoves {
    fn gen_moves(&self, piece_type: Piece, square: Square, side: Side) -> BitBoard;
}

struct AllPiecesMoveGen {
    leaping_pieces: Box<dyn GenerateLeapingMoves>,
    sliding_pieces: Box<dyn GenerateSlidingMoves>
}

impl AllPiecesMoveGen {
    fn get_moves(&self, square: Square, sides: &Sides, pieces: &Pieces) -> Result<Vec<Square>, MoveGenError> {
        let side = if sides.get(Side::White).is_piece_at(square) {
            Side::White
        } else if sides.get(Side::Black).is_piece_at(square) {
            Side::Black
        } else {
            Err(MoveGenError::NoPiece(square.to_string()))?
        };

        let piece = Piece::iter()
            .find(|&piece| pieces.get(piece).get(side).is_piece_at(square))
            .ok_or(MoveGenError::InvalidSidesPieces(side.to_string()))?;

        let moves_bb = self.leaping_pieces.gen_moves(piece, square, side);

        Ok(moves_bb.to_squares())
    }
}
