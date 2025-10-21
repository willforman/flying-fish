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
use crate::transposition_table::{
    EvalType, TranspositionTable, clear_transpostion_table_hitrate, get_transposition_table_hitrate,
};
use crate::{Side, Square};

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

    'outer: for iterative_deepening_max_depth in 1..=max_depth {
        let iteration_start_time = Instant::now();
        debug_span!(
            "search_iterative_deepening_iteration",
            depth = iterative_deepening_max_depth
        );
        debug!(
            "Iterative deepening iteration: {} of {}",
            iterative_deepening_max_depth, max_depth
        );
        let iterative_deepening_max_depth: u8 = iterative_deepening_max_depth.try_into().unwrap();
        let mut max_depth_reached: u8 = 1;

        // Find value of each move up to current depth
        let mut move_vals = HashMap::with_capacity(moves.len());
        for mve in moves.clone() {
            let mut move_position = move_positions[&mve].clone();
            let maybe_move_eval = search_helper(
                &mut move_position,
                &params,
                1,
                iterative_deepening_max_depth,
                &mut max_depth_reached,
                &mut positions_processed,
                &start,
                pv_eval,
                Eval::MIN,
                Eval::MAX,
                move_gen,
                position_eval,
                transposition_table,
                Arc::clone(&terminate),
            );
            if let Some(move_eval) = maybe_move_eval {
                // Since this is after making a move, flip the value to get the value
                // relative to the side of `position`
                move_vals.insert(mve, move_eval.flip());
            } else {
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
        }

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

        debug!("best move: {}, eval: {}", best_move.unwrap(), pv_eval);

        if tracing::enabled!(tracing::Level::DEBUG) {
            for mve in &moves {
                debug!("{}: {}", mve, move_vals[mve]);
            }
            debug!("==================================");
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
            let (soft, hard) = calc_time_to_use(white_time, params.black_inc, params.moves_to_go);
            (Some(soft), Some(hard))
        }
        (Side::Black, Some(black_time), _) => {
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

    let maybe_tt_best_move = if let Some(tt_entry) = transposition_table.get(position) {
        if tt_entry.depth() >= (max_depth - curr_depth) {
            if tt_entry.eval_type() == EvalType::Exact {
                return Some(tt_entry.eval);
            }
        }
        Some(tt_entry.best_move)
    } else {
        None
    };

    let mut moves = move_gen.gen_moves(position);
    if moves.is_empty() {
        if !move_gen.gen_checkers(position).is_empty() {
            return Some(Eval::MIN);
        } else {
            return Some(Eval::DRAW);
        }
    }
    order_moves(&mut moves, position, maybe_tt_best_move);

    let mut best_eval = Eval::MIN;
    let mut best_move = Move::new(Square::A1, Square::A1);
    let original_alpha = alpha;
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
        let got_eval = search_helper(
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
            Arc::clone(&terminate),
        )?;

        // Flip value because it was relative to the other side
        let got_eval = got_eval.flip();
        position.unmake_move(unmake_move_state);

        if got_eval >= best_eval {
            best_eval = got_eval;
            best_move = mve;
            if got_eval >= alpha {
                alpha = got_eval;
            }
        }

        if alpha >= beta {
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

    let mut best_eval = standing_pat;
    let moves: ArrayVec<Move, 218> = move_gen.gen_moves(position);
    if moves.is_empty() {
        if !move_gen.gen_checkers(position).is_empty() {
            return Some(Eval::MIN);
        } else {
            return Some(Eval::DRAW);
        }
    }

    let mut capture_moves = moves
        .into_iter()
        .filter(|mve| position.is_capture(mve))
        .collect();

    order_moves(&mut capture_moves, position, None);

    for mve in capture_moves {
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
        } else if move_eval > alpha {
            alpha = move_eval;
        }
    }

    Some(best_eval)
}

fn order_moves(
    moves: &mut ArrayVec<Move, 218>,
    position: &Position,
    maybe_tt_best_move: Option<Move>,
) {
    moves.sort_by_key(|mve| -(get_mvv_lva_value(mve, position) as i64));
    if let Some(tt_best_move) = maybe_tt_best_move {
        let tt_best_move_idx = moves.iter().position(|&m| m == tt_best_move);
        // .expect("Should have found tt_best_move in moves list");
        if tt_best_move_idx.is_none() {
            return;
        }
        moves.remove(tt_best_move_idx.unwrap());
        moves.insert(0, tt_best_move);
    }
}

fn get_mvv_lva_value(mve: &Move, position: &Position) -> usize {
    if position.is_capture(mve) {
        let attacker = position
            .is_piece_at(mve.src, position.state.to_move)
            .unwrap();
        let victim = position
            .is_piece_at(mve.dest, position.state.to_move.opposite_side())
            .unwrap();
        victim.index() * 10 + (5 - attacker.index())
    } else {
        0
    }
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
    use crate::Square::*;
    use test_case::test_case;

    #[test_case(Position::from_fen("7k/8/8/8/5q1b/3q1pP1/2r3b1/K3N3 w - - 0 1").unwrap(),
        [
            Move::new(E1, C2), Move::new(E1, D3), Move::new(E1, F3), Move::new(E1, G2),
            Move::new(G3, F4), Move::new(G3, G4), Move::new(G3, H4)
        ].into_iter().collect(),
        [
            Move::new(G3, F4), Move::new(E1, D3), Move::new(E1, C2), Move::new(G3, H4),
            Move::new(E1, G2), Move::new(E1, F3), Move::new(G3, G4),
        ].into_iter().collect() ; "simple"
    )]
    fn test_mvv_lva(
        position: Position,
        mut moves_input: ArrayVec<Move, 218>,
        moves_want: ArrayVec<Move, 218>,
    ) {
        moves_input.sort_by_key(|mve| -(get_mvv_lva_value(mve, &position) as i64));

        assert_eq!(moves_input, moves_want);
    }
}
