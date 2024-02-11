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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use testresult::TestResult;

    use crate::bitboard::Square::*;
    use crate::evaluation::POSITION_EVALUATOR;
    use crate::move_gen::HYPERBOLA_QUINTESSENCE_MOVE_GEN;
    use crate::position::Move;

    #[test_case(Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap(), 
    &[
        Move::new(D2, D4), Move::new(E7, E6),
        Move::new(C2, C4), Move::new(B8, C6),
        Move::new(B1, C3), Move::new(D8, H4),
        Move::new(G1, F3), Move::new(H4, G4),
        Move::new(H2, H3), Move::new(G4, G6),
        Move::new(E2, E4), Move::new(F8, B4),
        Move::new(F1, D3), Move::new(G6, G2),
        Move::new(A2, A3), Move::new(G2, H1),
        Move::new(E1, E2)
    ]; "random game that caused crash")]
    fn test_search(mut position: Position, moves: &[Move]) -> TestResult {
        for mve in moves {
            position.make_move(mve)?;
        }
        search(
            &position,
            3,
            HYPERBOLA_QUINTESSENCE_MOVE_GEN,
            POSITION_EVALUATOR,
        );
        Ok(())
    }
}
