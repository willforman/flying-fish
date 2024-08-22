pub mod all_pieces;
pub mod hyperbola_quintessence;
pub mod leaping_pieces;
pub mod magic_bitboard;
mod traits;

use arrayvec::ArrayVec;

use crate::position::{Move, Position};

use self::hyperbola_quintessence::HYPERBOLA_QUINTESSENCE;
use self::leaping_pieces::LEAPING_PIECES;
pub use self::traits::GenerateMoves;

#[derive(Clone, Copy)]
pub struct HyperbolaQuintessenceMoveGen;

impl GenerateMoves for HyperbolaQuintessenceMoveGen {
    fn gen_moves(&self, position: &Position) -> ArrayVec<Move, 80> {
        all_pieces::gen_moves(position, LEAPING_PIECES, HYPERBOLA_QUINTESSENCE)
    }

    fn gen_checkers(&self, position: &Position) -> crate::bitboard::BitBoard {
        all_pieces::get_checkers(position, LEAPING_PIECES, HYPERBOLA_QUINTESSENCE)
    }
}

pub static HYPERBOLA_QUINTESSENCE_MOVE_GEN: HyperbolaQuintessenceMoveGen =
    HyperbolaQuintessenceMoveGen {};
