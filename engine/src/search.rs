use crate::evaluation::EvaluatePosition;
use crate::move_gen::GenerateMoves;
use crate::position::Side;
use crate::position::{Move, Position};

pub fn find_move(
    position: &Position,
    depth: u32,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
) -> Move {
    let (mve, _best_val) = find_move_helper(position, 0, depth, move_gen, position_eval);
    mve
}

fn find_move_helper(
    position: &Position,
    curr_depth: u32,
    max_depth: u32,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
) -> (Move, f64) {
    let mut best_move: Option<Move> = None;
    if position.state.to_move == Side::White {
        let mut best_val = f64::MIN;
        for mve in move_gen.gen_moves(position) {
            let mut move_position = position.clone();
            move_position.make_move(&mve).unwrap();
            if curr_depth == (max_depth - 1) {
                let eval_score = position_eval.evaluate(&move_position);
                return (mve, eval_score);
            }
            let (_mve, got_val) = find_move_helper(
                &move_position,
                curr_depth + 1,
                max_depth,
                move_gen,
                position_eval,
            );
            if got_val > best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
        return (best_move.expect("Should have found a move"), best_val);
    } else {
        let mut best_val = f64::MAX;
        for mve in move_gen.gen_moves(position) {
            let mut move_position = position.clone();
            move_position.make_move(&mve).unwrap();
            if curr_depth == (max_depth - 1) {
                let eval_score = position_eval.evaluate(&move_position);
                return (mve, eval_score);
            }
            let (_mve, got_val) = find_move_helper(
                &move_position,
                curr_depth + 1,
                max_depth,
                move_gen,
                position_eval,
            );
            if got_val < best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
        return (best_move.expect("Should have found a move"), best_val);
    }
}
