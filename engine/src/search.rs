use crate::evaluation::EvaluatePosition;
use crate::move_gen::GenerateAllMoves;
use crate::position::Side;
use crate::position::{Move, Position};

pub fn find_move(
    position: &Position,
    evaluate_position: &impl EvaluatePosition,
    generate_moves: &impl GenerateAllMoves,
    depth: u32,
) -> Move {
    let (mve, _best_val) = find_move_helper(position, evaluate_position, generate_moves, 0, depth);
    mve
}

fn find_move_helper(
    position: &Position,
    evaluate_position: &impl EvaluatePosition,
    generate_moves: &impl GenerateAllMoves,
    curr_depth: u32,
    max_depth: u32,
) -> (Move, f64) {
    let mut best_move: Option<Move> = None;
    if position.state.to_move == Side::White {
        let mut best_val = f64::MIN;
        for mve in generate_moves.gen_moves(position) {
            let mut move_position = position.clone();
            move_position.make_move(&mve).unwrap();
            if curr_depth == (max_depth - 1) {
                let eval_score = evaluate_position.evaluate(&move_position);
                return (mve, eval_score);
            }
            let (_mve, got_val) = find_move_helper(
                &move_position,
                evaluate_position,
                generate_moves,
                curr_depth + 1,
                max_depth,
            );
            if got_val > best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
        return (best_move.expect("Should have found a move"), best_val);
    } else {
        let mut best_val = f64::MAX;
        for mve in generate_moves.gen_moves(position) {
            let mut move_position = position.clone();
            move_position.make_move(&mve).unwrap();
            if curr_depth == (max_depth - 1) {
                let eval_score = evaluate_position.evaluate(&move_position);
                return (mve, eval_score);
            }
            let (_mve, got_val) = find_move_helper(
                &move_position,
                evaluate_position,
                generate_moves,
                curr_depth + 1,
                max_depth,
            );
            if got_val < best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
        return (best_move.expect("Should have found a move"), best_val);
    }
}
