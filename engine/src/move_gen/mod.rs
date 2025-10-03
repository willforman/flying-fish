pub mod all_pieces;
pub mod hyperbola_quintessence;
pub mod leaping_pieces;
pub mod magic_bitboard;
mod masks;
mod traits;

use arrayvec::ArrayVec;

use crate::position::{Move, Position};

use self::hyperbola_quintessence::SLIDING_PIECES_MOVE_GEN;
use self::leaping_pieces::LEAPING_PIECES;
pub use self::traits::GenerateMoves;

#[derive(Clone, Copy)]
pub struct MoveGen;

impl GenerateMoves for MoveGen {
    fn gen_moves(&self, position: &Position) -> ArrayVec<Move, 218> {
        all_pieces::gen_moves(position, LEAPING_PIECES, SLIDING_PIECES_MOVE_GEN)
    }

    fn gen_checkers(&self, position: &Position) -> crate::bitboard::BitBoard {
        all_pieces::get_checkers(position, LEAPING_PIECES, SLIDING_PIECES_MOVE_GEN)
    }
}

pub static MOVE_GEN: MoveGen = MoveGen {};
