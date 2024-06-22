#![feature(const_mut_refs)]
#![feature(const_option)]
#![feature(const_trait_impl)]
#![feature(effects)]

mod algebraic_notation;
mod bitboard;
mod evaluation;
mod move_gen;
mod perft;
mod position;
mod search;

pub const NAME: &str = "Will's Engine";
pub const AUTHOR: &str = "Will Forman";

pub use algebraic_notation::move_to_algebraic_notation;
pub use bitboard::Square;
pub use evaluation::{EvaluatePosition, POSITION_EVALUATOR};
pub use move_gen::{GenerateMoves, HyperbolaQuintessenceMoveGen, HYPERBOLA_QUINTESSENCE_MOVE_GEN};
pub use perft::{perft, PerftDepthResult, PerftResult};
pub use position::{Move, Piece, Position, PositionError, Side};
pub use search::{search, SearchParams, SearchResultInfo};
