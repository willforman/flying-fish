use crate::evaluation::evaluate;
use crate::postition::{Position,Move};
use crate::move_gen::{GenerateAllMoves};

pub trait EvaluatePosition {
    pub fn evaluate(position: &Position) -> f64;
}

pub fn find_move(position: &Position, evaluate_position: &dyn EvaluatePosition, generate_moves: &dyn GenerateAllMoves) -> Move {
    let moves = generate_moves.gen_moves(position);

    moves.iter()
        .max_by_key(| &mve| {
            let mut move_position = position.clone();
            move_position.make_move(mve);
            evaluate_position.evaluate(move_position)
        })
}
