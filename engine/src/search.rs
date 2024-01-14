use crate::evaluation::EvaluatePosition;
use crate::move_gen::GenerateMoves;
use crate::position::Position;

pub fn search(
    position: &Position,
    depth: u32,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
) -> f64 {
    search_helper(position, 0, depth, move_gen, position_eval)
}

fn search_helper(
    position: &Position,
    curr_depth: u32,
    max_depth: u32,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
) -> f64 {
    if curr_depth == max_depth {
        return position_eval.evaluate(&position);
    }

    let moves = move_gen.gen_moves(position);

    let mut best_val = f64::MIN;
    for mve in moves {
        let mut move_position = position.clone();
        move_position.make_move(&mve).unwrap();

        let got_val = -search_helper(
            &move_position,
            curr_depth + 1,
            max_depth,
            move_gen,
            position_eval,
        );

        if got_val > best_val {
            best_val = got_val;
        }
    }
    best_val
}
