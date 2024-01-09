pub mod all_pieces;
pub mod hyperbola_quintessence;
pub mod leaping_pieces;

use std::collections::HashSet;

use crate::bitboard::BitBoard;
use crate::position::{Move, Position};

use self::{
    all_pieces::{GenerateAllMoves, ALL_PIECES_MOVE_GEN},
    hyperbola_quintessence::HYPERBOLA_QUINTESSENCE,
    leaping_pieces::LEAPING_PIECES_MOVE_GEN,
};

pub fn gen_moves_hyperbola_quintessence(position: &Position) -> HashSet<Move> {
    ALL_PIECES_MOVE_GEN.gen_moves(position, LEAPING_PIECES_MOVE_GEN, HYPERBOLA_QUINTESSENCE)
}

pub fn get_checkers_hyperbola_quintessence(position: &Position) -> BitBoard {
    ALL_PIECES_MOVE_GEN.get_checkers(position, LEAPING_PIECES_MOVE_GEN, HYPERBOLA_QUINTESSENCE)
}
