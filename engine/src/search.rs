use crate::evaluation::EvaluatePosition;
use crate::move_gen::all_pieces::{GenerateAllMoves, GenerateLeapingMoves, GenerateSlidingMoves};
use crate::position::Side;
use crate::position::{Move, Position};

pub fn find_move(
    position: &Position,
    evaluate_position: &impl EvaluatePosition,
    generate_moves: &impl GenerateAllMoves,
    depth: u32,
    leaping_pieces: impl GenerateLeapingMoves,
    sliding_pieces: impl GenerateSlidingMoves,
    all_pieces: impl GenerateAllMoves,
) -> Move {
    let (mve, _best_val) = find_move_helper(
        position,
        evaluate_position,
        generate_moves,
        0,
        depth,
        leaping_pieces,
        sliding_pieces,
        all_pieces,
    );
    mve
}

fn find_move_helper(
    position: &Position,
    evaluate_position: &impl EvaluatePosition,
    generate_moves: &impl GenerateAllMoves,
    curr_depth: u32,
    max_depth: u32,
    leaping_pieces: impl GenerateLeapingMoves,
    sliding_pieces: impl GenerateSlidingMoves,
    all_pieces: impl GenerateAllMoves,
) -> (Move, f64) {
    let mut best_move: Option<Move> = None;
    if position.state.to_move == Side::White {
        let mut best_val = f64::MIN;
        for mve in all_pieces.gen_moves(position, leaping_pieces, sliding_pieces) {
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
                leaping_pieces,
                sliding_pieces,
                all_pieces,
            );
            if got_val > best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
        return (best_move.expect("Should have found a move"), best_val);
    } else {
        let mut best_val = f64::MAX;
        for mve in all_pieces.gen_moves(position, leaping_pieces, sliding_pieces) {
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
                leaping_pieces,
                sliding_pieces,
                all_pieces,
            );
            if got_val < best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
        return (best_move.expect("Should have found a move"), best_val);
    }
}
