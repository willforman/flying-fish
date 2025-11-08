use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use arrayvec::ArrayVec;
use tracing::{debug, debug_span, error, info};

use crate::evaluation::{Eval, EvaluatePosition};
use crate::move_gen::GenerateMoves;
use crate::position::{Move, Position};
use crate::search::move_ordering::{ButterflyHistoryState, order_moves};
use crate::transposition_table::{
    EvalType, TranspositionTable, clear_transpostion_table_hitrate, get_transposition_table_hitrate,
};
use crate::{Piece, Side, Square};

mod move_ordering;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchParams {
    pub search_moves: Option<Vec<Move>>,
    pub ponder: bool,
    pub white_time: Option<Duration>,
    pub black_time: Option<Duration>,
    pub white_inc: Option<Duration>,
    pub black_inc: Option<Duration>,
    pub moves_to_go: Option<u16>,
    pub max_depth: Option<u8>,
    pub max_nodes: Option<u64>,
    pub mate: Option<u8>,
    pub move_time: Option<Duration>,
    pub infinite: bool,
}

impl Display for SearchParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print out only non default fields
        let default = SearchParams::default();
        let mut parts = vec![];

        if self.search_moves != default.search_moves {
            parts.push(format!(
                "search_moves: {:?}",
                self.search_moves.as_ref().unwrap()
            ));
        }
        if self.ponder != default.ponder {
            parts.push(format!("ponder: {:?}", self.ponder));
        }
        if self.white_time != default.white_time {
            parts.push(format!(
                "white_time: {:?}",
                self.white_time.as_ref().unwrap()
            ));
        }
        if self.black_time != default.black_time {
            parts.push(format!(
                "black_time: {:?}",
                self.black_time.as_ref().unwrap()
            ));
        }
        if self.white_inc != default.white_inc {
            parts.push(format!("white_inc: {:?}", self.white_inc.as_ref().unwrap()));
        }
        if self.black_inc != default.black_inc {
            parts.push(format!("black_inc: {:?}", self.black_inc.as_ref().unwrap()));
        }
        if self.moves_to_go != default.moves_to_go {
            parts.push(format!(
                "moves_to_go: {:?}",
                self.moves_to_go.as_ref().unwrap()
            ));
        }
        if self.max_depth != default.max_depth {
            parts.push(format!("max_depth: {:?}", self.max_depth.as_ref().unwrap()));
        }
        if self.max_nodes != default.max_nodes {
            parts.push(format!("max_nodes: {:?}", self.max_nodes.as_ref().unwrap()));
        }
        if self.mate != default.mate {
            parts.push(format!("mate: {:?}", self.mate.as_ref().unwrap()));
        }
        if self.move_time != default.move_time {
            parts.push(format!("move_time: {:?}", self.move_time.as_ref().unwrap()));
        }
        if self.infinite != default.infinite {
            parts.push(format!("infinite: {:?}", self.infinite));
        }
        write!(f, "SearchParams: {}", parts.join(", "))
    }
}

#[derive(Debug)]
pub struct SearchResultInfo {
    pub positions_processed: u64,
    pub time_elapsed: Duration,
    pub move_evals: HashMap<Move, Eval>,
}

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("Parameters depth and mate are mutually exclusive, both passed: {0}, {1}")]
    DepthAndMatePassed(u8, u8),

    #[error("Couldn't open search file logs: {0}")]
    OpenSearchLogsFile(PathBuf),
}

pub fn search(
    position: &Position,
    params: &SearchParams,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    transposition_table: &mut TranspositionTable,
    terminate: Arc<AtomicBool>,
) -> Result<(Option<Move>, SearchResultInfo), SearchError> {
    debug_span!("search", position = position.to_fen(), params = ?params);
    let mut params = params.clone();
    let mut best_move: Option<Move> = None;

    let mut positions_processed: u64 = 0;
    let start = Instant::now();
    let mut pv_eval = Eval::DRAW;

    let max_depth: usize = match (params.max_depth, params.mate) {
        (Some(max_depth), None) => max_depth.try_into().unwrap(),
        (None, Some(mate)) => mate.try_into().unwrap(),
        (Some(max_depth), Some(mate)) => {
            return Err(SearchError::DepthAndMatePassed(max_depth, mate));
        }
        (None, None) => 20,
    };

    let (maybe_soft_time_limit, maybe_hard_time_limit) =
        get_time_to_use(&params, position.state.to_move);
    debug!(
        "Time for this move: soft limit={:?} hard limit={:?}",
        maybe_soft_time_limit, maybe_hard_time_limit
    );

    if let Some(hard_time_limit) = maybe_hard_time_limit {
        params.move_time = Some(hard_time_limit);
    }

    let mut moves = move_gen.gen_moves(position);

    // Filter out moves not in search moves
    if let Some(search_moves) = &params.search_moves {
        moves.retain(|mve| search_moves.contains(mve));
    }

    let move_positions: HashMap<Move, Position> = moves
        .clone()
        .into_iter()
        .map(|mve| {
            let mut move_position = position.clone();
            move_position.make_move(mve);
            (mve, move_position)
        })
        .collect();

    let mut final_move_vals = HashMap::new();
    let mut butterfly_history_state = ButterflyHistoryState::new();

    let mut move_vals = HashMap::with_capacity(moves.len());

    'outer: for iterative_deepening_max_depth in 1..=max_depth {
        let iteration_start_time = Instant::now();
        debug_span!(
            "search_iterative_deepening_iteration",
            depth = iterative_deepening_max_depth
        );
        debug!("Iteration: {}/{}", iterative_deepening_max_depth, max_depth);
        let iterative_deepening_max_depth: u8 = iterative_deepening_max_depth.try_into().unwrap();
        let mut max_depth_reached: u8 = 1;

        for mve in moves.clone() {
            let mut move_position = move_positions[&mve].clone();

            const ASPIRATION_WINDOWS_DELTA: i32 = 50;
            let (mut alpha, mut beta) = if let Some(prev_move_val) = move_vals.get(&mve)
                && iterative_deepening_max_depth >= 4
            {
                (
                    *prev_move_val - ASPIRATION_WINDOWS_DELTA,
                    *prev_move_val + ASPIRATION_WINDOWS_DELTA,
                )
            } else {
                (Eval::MIN, Eval::MAX)
            };

            loop {
                let maybe_move_eval = search_helper(
                    &mut move_position,
                    &params,
                    1,
                    iterative_deepening_max_depth,
                    &mut max_depth_reached,
                    &mut positions_processed,
                    &start,
                    pv_eval,
                    alpha,
                    beta,
                    move_gen,
                    position_eval,
                    transposition_table,
                    &mut butterfly_history_state,
                    Arc::clone(&terminate),
                );
                if maybe_move_eval.is_none() {
                    write_search_info(
                        iterative_deepening_max_depth,
                        positions_processed,
                        iterative_deepening_max_depth,
                        &start,
                        pv_eval,
                        None,
                    );
                    break 'outer;
                }

                let move_eval = maybe_move_eval.unwrap();
                if alpha != Eval::MIN && move_eval <= alpha {
                    let alpha_val = -(alpha
                        .value()
                        .checked_mul(alpha.value())
                        .unwrap_or(Eval::MIN.value()));
                    alpha = Eval(alpha_val);
                } else if beta != Eval::MAX && move_eval >= beta {
                    let beta_val = beta
                        .value()
                        .checked_mul(beta.value())
                        .unwrap_or(Eval::MAX.value());
                    beta = Eval(beta_val);
                } else {
                    move_vals.insert(mve, move_eval.flip());
                    break;
                }
            }
        }
        final_move_vals = move_vals.clone();

        // Sort moves by descending value, for this depth
        moves.sort_by(|move1, move2| {
            let val1 = move_vals[move1];
            let val2 = move_vals[move2];
            val2.partial_cmp(&val1).unwrap()
        });

        // Find best move
        best_move = Some(moves[0]);

        pv_eval = move_vals[&best_move.unwrap()];

        write_search_info(
            iterative_deepening_max_depth,
            positions_processed,
            max_depth_reached,
            &start,
            pv_eval,
            best_move,
        );

        if tracing::enabled!(tracing::Level::DEBUG) {
            let moves_str = moves
                .iter()
                .map(|mve| format!("{}: {}", mve, move_vals[mve]))
                .collect::<Vec<_>>()
                .join(" | ");
            debug!("MOVES: {}", moves_str);
        }

        // Skip if we've elapsed the max amount of time or that we think the next iteration will
        // definitely go over on time
        let elapsed = start.elapsed();
        if let Some(soft_time_limit) = maybe_soft_time_limit {
            if (elapsed + iteration_start_time.elapsed()) > soft_time_limit {
                debug!(
                    "Search time exceeded soft limit: {:?} > {:?}",
                    elapsed, maybe_soft_time_limit
                );
                break 'outer;
            }
        }
        debug!("Time: {:?} < {:?} to use", elapsed, maybe_soft_time_limit);
    }

    let search_info = SearchResultInfo {
        positions_processed,
        time_elapsed: start.elapsed(),
        move_evals: final_move_vals,
    };
    clear_transpostion_table_hitrate();

    Ok((best_move, search_info))
}

fn get_time_to_use(
    params: &SearchParams,
    side_to_move: Side,
) -> (Option<Duration>, Option<Duration>) {
    let (soft, mut hard) = match (side_to_move, params.white_time, params.black_time) {
        (Side::White, Some(white_time), _) => {
            let (soft, hard) = calc_time_to_use(white_time, params.white_inc, params.moves_to_go);
            (Some(soft), Some(hard))
        }
        (Side::Black, _, Some(black_time)) => {
            let (soft, hard) = calc_time_to_use(black_time, params.black_inc, params.moves_to_go);
            (Some(soft), Some(hard))
        }
        (_, _, _) => (None, None),
    };
    if let Some(move_time) = params.move_time {
        hard = Some(move_time);
    }
    (soft, hard)
}

/// Calculate the time to use during search.
/// Returns a soft and hard limit time.
fn calc_time_to_use(
    time_left: Duration,
    maybe_time_inc: Option<Duration>,
    maybe_moves_to_go: Option<u16>,
) -> (Duration, Duration) {
    let time_inc = maybe_time_inc.unwrap_or(Duration::from_secs(0));
    let usable_time = time_left - (time_left / 20);
    let moves_to_go = maybe_moves_to_go.unwrap_or(40);
    let soft_limit = (usable_time / moves_to_go.into()) + time_inc;
    let hard_limit = soft_limit * 2;
    (soft_limit, hard_limit)
}

#[allow(clippy::too_many_arguments)]
fn search_helper(
    position: &mut Position,
    params: &SearchParams,
    curr_depth: u8,
    max_depth: u8,
    max_depth_reached: &mut u8,
    positions_processed: &mut u64,
    start_time: &Instant,
    pv_eval: Eval,
    mut alpha: Eval,
    beta: Eval,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    transposition_table: &mut TranspositionTable,
    butterfly_history_state: &mut ButterflyHistoryState,
    terminate: Arc<AtomicBool>,
) -> Option<Eval> {
    // If this search has been terminated, return early
    if terminate.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    // If this search is at the max number of nodes, return early
    if let Some(max_nodes) = params.max_nodes {
        debug_assert!(*positions_processed <= max_nodes);
        if *positions_processed == max_nodes {
            return None;
        }
    }
    // If search has exceeded total time, return early
    if let Some(move_time) = params.move_time {
        if start_time.elapsed() >= move_time {
            debug!("Search elapsed total time: {:?}", move_time);
            return None;
        }
    }
    *positions_processed += 1;
    if curr_depth > *max_depth_reached {
        *max_depth_reached = curr_depth;
    }

    if *positions_processed % 250_000 == 0 {
        write_search_info(
            max_depth,
            *positions_processed,
            curr_depth,
            start_time,
            pv_eval,
            None,
        );
    }

    // Once we reach max depth, use quiescence search to extend
    // search.
    if curr_depth == max_depth {
        return quiescence_search(
            position,
            params,
            curr_depth,
            max_depth,
            max_depth_reached,
            positions_processed,
            start_time,
            pv_eval,
            alpha,
            beta,
            move_gen,
            position_eval,
            terminate,
        );
    }

    if position.is_draw() {
        return Some(Eval::DRAW);
    }

    let maybe_tt_best_move = if let Some(tt_entry) = transposition_table.get(position) {
        if !position.is_repetition_possible() && tt_entry.depth() >= (max_depth - curr_depth) {
            let eval_type = tt_entry.eval_type();
            if eval_type == EvalType::Exact
                || (eval_type == EvalType::LowerBound && tt_entry.eval >= beta)
                || (eval_type == EvalType::UpperBound && tt_entry.eval <= alpha)
            {
                return Some(tt_entry.eval);
            }
        }
        Some(tt_entry.best_move)
    } else {
        None
    };

    let is_pv_node = alpha != beta - 1;

    let checkers = move_gen.gen_checkers(position);
    let eval = position_eval.evaluate(position, move_gen);

    // Null Move Pruning
    if !is_pv_node
        && checkers.is_empty()
        && eval >= beta
        && curr_depth >= NULL_MOVE_PRUNING_DEPTH
        && position.has_non_pawn_material()
    {
        const R: u8 = 2;

        let nmp_depth = curr_depth + R;

        if nmp_depth <= max_depth {
            let unmake_en_passant_target = position.make_null_move();

            let nmp_eval = search_helper(
                position,
                params,
                nmp_depth,
                max_depth,
                max_depth_reached,
                positions_processed,
                start_time,
                pv_eval,
                beta.flip(),
                beta.flip() + 1,
                move_gen,
                position_eval,
                transposition_table,
                butterfly_history_state,
                Arc::clone(&terminate),
            )?
            .flip();
            position.unmake_null_move(unmake_en_passant_target);

            if nmp_eval >= beta {
                return Some(nmp_eval);
            }
        }
    }

    let mut moves = move_gen.gen_moves(position);
    if moves.is_empty() {
        if !move_gen.gen_checkers(position).is_empty() {
            return Some(Eval::MIN);
        }
        return Some(Eval::DRAW);
    }

    order_moves(
        &mut moves,
        position,
        maybe_tt_best_move,
        Some(butterfly_history_state),
    );

    let mut best_eval = Eval::MIN;
    let mut best_move = moves[0];
    let original_alpha = alpha;
    for (idx, mve) in moves.into_iter().enumerate() {
        butterfly_history_state.record_considered(mve);

        let unmake_move_state = position.make_move(mve);
        #[cfg(debug_assertions)]
        {
            if let Err(e) = position.validate_position(mve) {
                panic!("Validation failed: {}", e);
            }
        }

        let got_eval = if idx == 0 {
            // Reason for `?`: if the child node is signaling search is terminated,
            // better terminate self.
            search_helper(
                position,
                params,
                curr_depth + 1,
                max_depth,
                max_depth_reached,
                positions_processed,
                start_time,
                pv_eval,
                beta.flip(),
                alpha.flip(),
                move_gen,
                position_eval,
                transposition_table,
                butterfly_history_state,
                Arc::clone(&terminate),
            )?
            .flip()
        } else {
            let got_eval = search_helper(
                position,
                params,
                curr_depth + 1,
                max_depth,
                max_depth_reached,
                positions_processed,
                start_time,
                pv_eval,
                alpha.flip() - 1,
                alpha.flip(),
                move_gen,
                position_eval,
                transposition_table,
                butterfly_history_state,
                Arc::clone(&terminate),
            )?
            .flip();
            if alpha < got_eval && got_eval < beta {
                search_helper(
                    position,
                    params,
                    curr_depth + 1,
                    max_depth,
                    max_depth_reached,
                    positions_processed,
                    start_time,
                    pv_eval,
                    beta.flip(),
                    alpha.flip(),
                    move_gen,
                    position_eval,
                    transposition_table,
                    butterfly_history_state,
                    Arc::clone(&terminate),
                )?
                .flip()
            } else {
                got_eval
            }
        };

        // Flip value because it was relative to the other side
        position.unmake_move(unmake_move_state);

        if got_eval > best_eval {
            best_eval = got_eval;
            best_move = mve;
            if got_eval > alpha {
                alpha = got_eval;
            }
        }

        if alpha >= beta {
            butterfly_history_state.record_cutoff(mve, curr_depth);
            break;
        }
    }
    let tt_eval_type = if best_eval >= beta {
        EvalType::LowerBound
    } else if best_eval <= original_alpha {
        EvalType::UpperBound
    } else {
        EvalType::Exact
    };
    transposition_table.store(
        position,
        best_eval,
        tt_eval_type,
        best_move,
        max_depth - curr_depth,
    );

    Some(best_eval)
}

const NULL_MOVE_PRUNING_DEPTH: u8 = 3;

/// Source: https://www.chessprogramming.org/Quiescence_Search
#[allow(clippy::too_many_arguments)]
fn quiescence_search(
    position: &mut Position,
    params: &SearchParams,
    curr_depth: u8,
    max_depth: u8,
    max_depth_reached: &mut u8,
    positions_processed: &mut u64,
    start_time: &Instant,
    pv_eval: Eval,
    mut alpha: Eval,
    beta: Eval,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    terminate: Arc<AtomicBool>,
) -> Option<Eval> {
    // If this search has been terminated, return early
    if terminate.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    // If search has exceeded total time, return early
    if let Some(move_time) = params.move_time {
        if start_time.elapsed() >= move_time {
            return None;
        }
    }
    *positions_processed += 1;
    if curr_depth > *max_depth_reached {
        *max_depth_reached = curr_depth;
    }

    if *positions_processed % 250_000 == 0 {
        write_search_info(
            max_depth,
            *positions_processed,
            *max_depth_reached,
            start_time,
            pv_eval,
            None,
        );
    }

    if position.is_draw() {
        return Some(Eval::DRAW);
    }

    let checkers = move_gen.gen_checkers(position);
    let mut best_eval = if checkers.is_empty() {
        let standing_pat = position_eval.evaluate(position, move_gen);

        if curr_depth >= max_depth * 3 {
            return Some(standing_pat);
        }

        if standing_pat >= beta {
            return Some(standing_pat);
        }
        if standing_pat > alpha {
            alpha = standing_pat;
        }
        standing_pat
    } else {
        Eval::MIN
    };

    let mut moves: ArrayVec<Move, 218> = move_gen.gen_moves(position);
    if moves.is_empty() {
        if !move_gen.gen_checkers(position).is_empty() {
            return Some(Eval::MIN);
        }
        // Stalemate
        return Some(Eval::DRAW);
    }

    // Filter out quiet moves, but only if in check.
    if checkers.is_empty() {
        moves = moves
            .into_iter()
            .filter(|&mve| {
                position.is_capture(mve)
                    || mve.promotion == Some(Piece::Queen)
                    || mve.promotion == Some(Piece::Knight)
            })
            .collect();
    }

    order_moves(&mut moves, position, None, None);

    for mve in moves {
        let unmake_move_state = position.make_move(mve);
        #[cfg(debug_assertions)]
        {
            if let Err(e) = position.validate_position(mve) {
                panic!("Validation failed: {}", e);
            }
        }

        // Reason for `?`: if the child node is signaling search is terminated,
        // better terminate self.
        let move_eval = quiescence_search(
            position,
            params,
            curr_depth + 1,
            max_depth,
            max_depth_reached,
            positions_processed,
            start_time,
            pv_eval,
            beta.flip(),
            alpha.flip(),
            move_gen,
            position_eval,
            Arc::clone(&terminate),
        )?;
        // Flip value because it was relative to the other side
        let move_eval = move_eval.flip();
        position.unmake_move(unmake_move_state);

        if move_eval >= beta {
            return Some(move_eval);
        }
        if move_eval > best_eval {
            best_eval = move_eval;
        }
        if move_eval > alpha {
            alpha = move_eval;
        }
    }

    Some(best_eval)
}

fn write_search_info(
    iterative_deepening_max_depth: u8,
    nodes_processed: u64,
    max_depth_reached: u8,
    start_time: &Instant,
    pv_eval: Eval,
    best_move: Option<Move>,
) {
    let nps = nodes_processed as f32 / start_time.elapsed().as_secs_f32();
    info!(
        target: "uci",
        "info depth {} seldepth {} multipv {} score {} nodes {} nps {:.0} hashfull {} tbhits {} tthitrate {:.2} time {} pv {}",
        iterative_deepening_max_depth,
        max_depth_reached,
        1,
        pv_eval,
        nodes_processed,
        nps,
        0,
        0,
        get_transposition_table_hitrate(),
        start_time.elapsed().as_millis(),
        best_move.map_or("".to_string(), |mve| mve.to_string().to_ascii_lowercase()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MOVE_GEN, POSITION_EVALUATOR, Square::*};
    use test_case::test_case;

    #[test_case(Position::from_fen("rnb1kbnr/2q2ppp/pp1p4/2p1p3/8/1P1PP1P1/PBPNNPBP/R2QK2R b KQkq - 0 1").unwrap(), vec![
        Move::new(B8, C6), Move::new(E1, G1), Move::new(C8, B7), Move::new(E2, C3),
        Move::new(G8, F6), Move::new(A1, C1), Move::new(E8, C8), Move::new(D2, E4),
        Move::new(C6, B4), Move::new(A2, A3), Move::new(B4, D5), Move::new(C3, D5),
        Move::new(B7, D5), Move::new(C2, C4), Move::new(D5, E4), Move::new(D3, E4),
        Move::new(F8, E7), Move::new(D1, F3), Move::new(H8, E8), Move::new(C1, D1),
        Move::new(C8, B8), Move::new(F1, E1), Move::new(B6, B5), Move::new(H2, H3),
        Move::new(H7, H6), Move::new(B2, C3), Move::new(E8, F8), Move::new(F3, F5),
        Move::new(F8, E8), Move::new(F5, F3), Move::new(E8, F8), Move::new(F3, E2),
        Move::new(C7, B6), Move::new(E2, D3), Move::new(B5, C4), Move::new(B3, C4),
        Move::new(B6, C6), Move::new(D1, B1), Move::new(B8, A7), Move::new(C3, A5),
        Move::new(D8, B8), Move::new(B1, D1), Move::new(B8, B2), Move::new(D3, C3),
        Move::new(B2, B8), Move::new(C3, C2), Move::new(B8, E8), Move::new(A5, C3),
        Move::new(A7, B8), Move::new(A3, A4), Move::new(E7, D8), Move::new(C2, B3),
        Move::new(B8, C8), Move::new(B3, C2), Move::new(C8, B8), Move::new(C2, B3),
        Move::new(B8, C8), Move::new(B3, C2),
    ], 2, Move::new(C8, B8), Eval::DRAW)]
    fn test_expected_search_result(
        mut position: Position,
        start_moves: Vec<Move>,
        max_depth: u8,
        mve: Move,
        eval_want: Eval,
    ) {
        for start_mve in start_moves {
            position.make_move(start_mve);
        }

        let (_, search_res) = search(
            &position,
            &SearchParams {
                max_depth: Some(max_depth),
                search_moves: Some(vec![mve]),
                ..Default::default()
            },
            MOVE_GEN,
            POSITION_EVALUATOR,
            &mut TranspositionTable::new(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();

        let eval_got = search_res.move_evals[&mve];

        assert_eq!(eval_got, eval_want);
    }
}
