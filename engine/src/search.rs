use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::evaluation::EvaluatePosition;
use crate::move_gen::GenerateMoves;
use crate::position::{Move, Position};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SearchParams {
    pub search_moves: Option<Vec<Move>>,
    pub ponder: bool,
    pub white_time_msec: Option<u64>,
    pub black_time_msec: Option<u64>,
    pub white_inc_msec: Option<u64>,
    pub black_inc_msec: Option<u64>,
    pub moves_to_go: Option<u64>,
    pub max_depth: Option<u64>,
    pub max_nodes: Option<u64>,
    pub mate: Option<u64>,
    pub move_time_msec: Option<u64>,
    pub infinite: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchInfo {
    pub positions_processed: u64,
    pub time_elapsed: Duration,
}

pub fn search(
    position: &Position,
    params: &SearchParams,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    terminate: Arc<AtomicBool>,
) -> (Option<Move>, SearchInfo) {
    let mut positions_processed: u64 = 0;
    let start = Instant::now();
    let (best_move, _best_val) = search_helper(
        position,
        params,
        0,
        &mut positions_processed,
        &start,
        f64::MIN,
        f64::MAX,
        move_gen,
        position_eval,
        Arc::clone(&terminate),
    );
    let search_info = SearchInfo {
        positions_processed,
        time_elapsed: start.elapsed(),
    };

    (best_move, search_info)
}

#[allow(clippy::too_many_arguments)]
fn search_helper(
    position: &Position,
    params: &SearchParams,
    curr_depth: u64,
    positions_processed: &mut u64,
    start_time: &Instant,
    mut alpha: f64,
    beta: f64,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    terminate: Arc<AtomicBool>,
) -> (Option<Move>, f64) {
    // If this search has been terminated, return early
    if terminate.load(std::sync::atomic::Ordering::Relaxed) {
        return (None, 0.0);
    }
    // If this search is at the max number of nodes, return early
    if let Some(max_nodes) = params.max_nodes {
        debug_assert!(*positions_processed <= max_nodes);
        if *positions_processed == max_nodes {
            return (None, 0.0);
        }
    }
    // If search has exceeded total time, return early
    if let Some(move_time_msec) = params.move_time_msec {
        if start_time.elapsed().as_millis() >= u128::from(move_time_msec) {
            return (None, 0.0);
        }
    }

    let moves = move_gen.gen_moves(position);

    let mut best_val = f64::MIN;
    let mut best_move: Option<Move> = None;
    for mve in moves {
        let mut move_position = position.clone();
        move_position.make_move(&mve).unwrap();

        if let Some(max_depth) = params.max_depth {
            if curr_depth + 1 == max_depth {
                let val = position_eval.evaluate(&move_position);
                return (Some(mve), val);
            }
        }

        let (got_mve, got_val) = search_helper(
            &move_position,
            params,
            curr_depth + 1,
            positions_processed,
            start_time,
            -beta,
            -alpha,
            move_gen,
            position_eval,
            Arc::clone(&terminate),
        );

        // Search has been terminated, return best move we found
        if got_mve.is_none() && got_val == 0.0 {
            break;
        }

        let got_val = -got_val;

        if got_val >= best_val {
            best_val = got_val;
            best_move = Some(mve);
        }

        best_val = f64::max(best_val, got_val);

        alpha = f64::max(alpha, got_val);

        if alpha >= beta {
            break;
        }
    }

    (best_move, best_val)
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
            &SearchParams {
                max_depth: Some(3),
                ..SearchParams::default()
            },
            HYPERBOLA_QUINTESSENCE_MOVE_GEN,
            POSITION_EVALUATOR,
            Arc::new(AtomicBool::new(false)),
        );
        Ok(())
    }
}
