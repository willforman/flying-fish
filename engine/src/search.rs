use core::{f64, panic};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use strum::IntoEnumIterator;
use tracing::{debug, debug_span, enabled, error, info, warn};

use crate::evaluation::{Eval, EvaluatePosition};
use crate::move_gen::GenerateMoves;
use crate::position::{Move, Position};
use crate::{Piece, TRACING_TARGET_SEARCH};
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
    pub max_depth: Option<u64>,
    pub max_nodes: Option<u64>,
    pub mate: Option<u64>,
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

enum SearchInfo {
    Depth {
        plies: u32,
    },
    SelDepth {
        plies: u32,
    },
    Time {
        msec: u64,
    },
    Nodes {
        nodes: u64,
    },
    Pv {
        moves: Vec<Move>,
    },
    MultiPv {
        num: u8,
    },
    Score {
        centipawns: f32,
        mate_moves: Option<i32>,
        lower_bound: Option<bool>,
        upper_bound: Option<bool>,
    },
    CurrMove {
        mve: Move,
    },
    CurrMoveNumber {
        move_num: u32,
    },
    HashFull {
        per_mill_full: u16,
    },
    NodesPerSecond {
        nodes_per_sec: f32,
    },
    TableHits {
        hits: u64,
    },
    ShredderHits {
        hits: u64,
    },
    CPULoad {
        cpu_usage_per_mill: u16,
    },
    String {
        str: String,
    },
    Refutation {
        moves: Vec<Move>,
    },
    CurrLine {
        moves: Vec<Move>,
        cpu_num: Option<u8>,
    },
}

fn moves_to_string(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|mve| mve.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

impl Display for SearchInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info_str = match self {
            SearchInfo::Depth { plies } => format!("depth {}", plies),
            SearchInfo::SelDepth { plies } => format!("seldepth {}", plies),
            SearchInfo::Time { msec } => format!("time {}", msec),
            SearchInfo::Nodes { nodes } => format!("nodes {}", nodes),
            SearchInfo::Pv { moves } => format!("pv {}", moves_to_string(moves)),
            SearchInfo::MultiPv { num } => format!("multipv {}", num),
            SearchInfo::Score {
                centipawns,
                mate_moves,
                lower_bound,
                upper_bound,
            } => {
                let mut score_str = format!("score cp {}", centipawns);

                if let Some(mate_moves) = mate_moves {
                    score_str.push_str(format!(" mate {}", mate_moves).as_str());
                };

                if lower_bound.is_some() {
                    score_str.push_str(" lowerbound");
                }
                if upper_bound.is_some() {
                    score_str.push_str(" upperbound");
                }

                score_str
            }
            SearchInfo::CurrMove { mve } => format!("currmove {}", mve),
            SearchInfo::CurrMoveNumber { move_num } => format!("currmovenumber {}", move_num),
            SearchInfo::HashFull { per_mill_full } => format!("hashfull {}", per_mill_full),
            SearchInfo::NodesPerSecond { nodes_per_sec } => format!("nps {}", nodes_per_sec),
            SearchInfo::TableHits { hits } => format!("tbhits {}", hits),
            SearchInfo::ShredderHits { hits } => format!("sbhits {}", hits),
            SearchInfo::CPULoad { cpu_usage_per_mill } => format!("cpuload {}", cpu_usage_per_mill),
            SearchInfo::String { str } => format!("string {}", str),
            SearchInfo::Refutation { moves } => format!("refutation {}", moves_to_string(moves)),
            SearchInfo::CurrLine { moves, cpu_num } => {
                if let Some(cpu_num) = cpu_num {
                    format!("currline {} {}", cpu_num, moves_to_string(moves))
                } else {
                    format!("currline {}", moves_to_string(moves))
                }
            }
        };
        write!(f, "{}", info_str)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("Parameters depth and mate are mutually exclusive, both passed: {0}, {1}")]
    DepthAndMatePassed(u64, u64),

    #[error("Couldn't open search file logs: {0}")]
    OpenSearchLogsFile(PathBuf),
}

/// Calculate the time to use during search.
/// Returns a soft and hard limit time.
fn calc_time_to_use(
    time_left: Duration,
    time_inc: Duration,
    maybe_moves_to_go: Option<u16>,
) -> (Duration, Duration) {
    let usable_time = time_left - (time_left / 20);
    let moves_to_go = maybe_moves_to_go.unwrap_or(40);
    let soft_limit = (usable_time / moves_to_go.into()) + time_inc;
    let hard_limit = soft_limit * 2;
    (soft_limit, hard_limit)
}

pub fn search(
    position: &Position,
    params: &SearchParams,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    terminate: Arc<AtomicBool>,
) -> Result<(Option<Move>, SearchResultInfo), SearchError> {
    debug_span!("search", position = position.to_fen(), params = ?params);
    let mut params = params.clone();
    let mut best_move: Option<Move> = None;
    let mut best_val: Option<Move> = None;

    let mut positions_processed: u64 = 0;
    let start = Instant::now();
    let mut latest_eval = Eval::Score(0.);

    let max_depth: usize = match (params.max_depth, params.mate) {
        (Some(max_depth), None) => max_depth.try_into().unwrap(),
        (None, Some(mate)) => mate.try_into().unwrap(),
        (Some(max_depth), Some(mate)) => {
            return Err(SearchError::DepthAndMatePassed(max_depth, mate))
        }
        (None, None) => 20,
    };

    let (time_soft_limit, time_hard_limit) = if position.state.to_move == Side::White {
        calc_time_to_use(
            params.white_time.unwrap_or(Duration::from_secs(60)),
            params.white_inc.unwrap_or(Duration::from_secs(0)),
            params.moves_to_go,
        )
    } else {
        calc_time_to_use(
            params.black_time.unwrap_or(Duration::from_secs(60)),
            params.black_inc.unwrap_or(Duration::from_secs(0)),
            params.moves_to_go,
        )
    };
    debug!(
        "Time for this move: soft limit={:?} hard limit={:?}",
        time_soft_limit, time_hard_limit
    );

    if params.move_time.is_none() {
        params.move_time = Some(time_hard_limit);
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
            move_position.make_move(mve).unwrap();
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
        let iterative_deepening_max_depth: u64 = iterative_deepening_max_depth.try_into().unwrap();
        let mut max_depth_reached: u64 = 1;

        // Find value of each move up to current depth
        let mut move_vals = HashMap::with_capacity(moves.len());
        for mve in moves.clone() {
            let move_position = &move_positions[&mve];
            let maybe_move_eval = search_helper(
                move_position,
                &params,
                1,
                iterative_deepening_max_depth,
                &mut max_depth_reached,
                &mut positions_processed,
                &start,
                &mut latest_eval,
                Eval::Mate(0), // Minimum `Eval` value
                Eval::Mate(1), // Maximum `Eval` value
                move_gen,
                position_eval,
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
                    &latest_eval,
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

        latest_eval = move_vals[&best_move.unwrap()];

        write_search_info(
            iterative_deepening_max_depth,
            positions_processed,
            max_depth_reached,
            &start,
            &latest_eval,
            best_move,
        );

        debug!("best move: {}, eval: {}", best_move.unwrap(), latest_eval);

        if tracing::enabled!(tracing::Level::DEBUG) {
            for mve in &moves {
                debug!("{}: {}", mve, move_vals[mve]);
            }
            debug!("==================================");
        }

        // Skip if we've elapsed the max amount of time or that we think the next iteration will
        // definitely go over on time
        let elapsed = start.elapsed();
        if (elapsed + iteration_start_time.elapsed()) > time_soft_limit {
            debug!(
                "Search time exceeded soft limit: {:?} > {:?}",
                elapsed, time_soft_limit
            );
            break 'outer;
        }
        debug!("Time: {:?} < {:?} to use", elapsed, time_soft_limit);
    }

    let search_info = SearchResultInfo {
        positions_processed,
        time_elapsed: start.elapsed(),
    };

    Ok((best_move, search_info))
}

#[allow(clippy::too_many_arguments)]
fn search_helper(
    position: &Position,
    params: &SearchParams,
    curr_depth: u64,
    max_depth: u64,
    max_depth_reached: &mut u64,
    positions_processed: &mut u64,
    start_time: &Instant,
    latest_eval: &mut Eval,
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
            latest_eval,
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
            latest_eval,
            alpha,
            beta,
            move_gen,
            position_eval,
            terminate,
        );
    }

    let moves = move_gen.gen_moves(position);

    let mut best_eval = Eval::Mate(0);
    for mve in moves {
        let mut move_position = position.clone();
        let move_res = move_position.make_move(mve);
        #[cfg(debug_assertions)]
        {
            if let Err(e) = move_position.validate_position(mve) {
                panic!("Validation failed: {}", e);
            }
        }
        if let Err(err) = move_res {
            write_search_info(
                max_depth,
                *positions_processed,
                *max_depth_reached,
                start_time,
                latest_eval,
                None,
            );
            error!("Error for move {}: {}", mve, err);
            panic!("Err encountered searching, exiting");
        }

        // Reason for `?`: if the child node is signaling search is terminated,
        // better terminate self.
        let got_eval = search_helper(
            &move_position,
            params,
            curr_depth + 1,
            max_depth,
            max_depth_reached,
            positions_processed,
            start_time,
            latest_eval,
            beta.flip(),
            alpha.flip(),
            move_gen,
            position_eval,
            Arc::clone(&terminate),
        )?;

        // Then, flip value because it was relative to the other side
        let got_eval = got_eval.flip();

        if got_eval >= best_eval {
            best_eval = got_eval;
            if got_eval >= alpha {
                alpha = got_eval;
            }
            *latest_eval = got_eval;
        }

        if alpha >= beta {
            break;
        }
    }

    Some(best_eval)
}

/// Source: https://www.chessprogramming.org/Quiescence_Search
#[allow(clippy::too_many_arguments)]
fn quiescence_search(
    position: &Position,
    params: &SearchParams,
    curr_depth: u64,
    max_depth: u64,
    max_depth_reached: &mut u64,
    positions_processed: &mut u64,
    start_time: &Instant,
    latest_eval: &mut Eval,
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
            latest_eval,
            None,
        );
    }

    let standing_pat = position_eval.evaluate(position, move_gen);
    // println!("Stading pat: {:?}", standing_pat);
    // If quiescence search exceeds 2x max depth, terminate.
    if curr_depth >= max_depth * 3 {
        return Some(standing_pat);
    }
    // if standing_pat >= beta {
    //     return Some(standing_pat);
    // }
    // if standing_pat > alpha {
    //     alpha = standing_pat;
    // }

    let mut best_eval = standing_pat;
    let capture_moves = move_gen
        .gen_moves(position)
        .into_iter()
        .filter(|mve| position.is_capture(mve));

    for mve in capture_moves {
        let mut move_position = position.clone();
        let move_res = move_position.make_move(mve);
        #[cfg(debug_assertions)]
        {
            if let Err(e) = move_position.validate_position(mve) {
                // println!("Curr depth: {curr_depth}");
                // println!("BEFORE======== {}\n{:?}", position.to_fen(), position);
                // for side in Side::iter() {
                //     for piece in Piece::iter() {
                //         println!(
                //             "{} {}:\n{:?}",
                //             side,
                //             piece,
                //             position.pieces.get(piece).get(side)
                //         );
                //     }
                // }
                // println!(
                //     "AFTER======== {}\n{:?}",
                //     move_position.to_fen(),
                //     move_position
                // );
                // for side in Side::iter() {
                //     for piece in Piece::iter() {
                //         println!(
                //             "{} {}:\n{:?}",
                //             side,
                //             piece,
                //             move_position.pieces.get(piece).get(side)
                //         );
                //     }
                // }
                // println!("Moves: {:?}", move_gen.gen_moves(position));
                // println!(
                //     "Checkers: {:?}",
                //     move_gen.gen_checkers(position).to_squares()
                // );
                panic!("Validation failed: {}", e);
            }
        }
        if let Err(err) = move_res {
            write_search_info(
                max_depth,
                *positions_processed,
                curr_depth,
                start_time,
                latest_eval,
                None,
            );
            error!("Error for move {}: {}", mve, err);
            panic!("Err encountered searching, exiting");
        }

        // Reason for `?`: if the child node is signaling search is terminated,
        // better terminate self.
        let move_eval = quiescence_search(
            &move_position,
            params,
            curr_depth + 1,
            max_depth,
            max_depth_reached,
            positions_processed,
            start_time,
            latest_eval,
            beta.flip(),
            alpha.flip(),
            move_gen,
            position_eval,
            Arc::clone(&terminate),
        )?;

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

fn write_search_info(
    iterative_deepening_max_depth: u64,
    nodes_processed: u64,
    max_depth_reached: u64,
    start_time: &Instant,
    latest_eval: &Eval,
    best_move: Option<Move>,
) {
    let nps = nodes_processed as f32 / start_time.elapsed().as_secs_f32();
    info!("info depth {} seldepth {} multipv {} score cp {} nodes {} nps {:.0} hashfull {} tbhits {} time {} pv {}", iterative_deepening_max_depth, max_depth_reached, 1, latest_eval, nodes_processed, nps, 0, 0, start_time.elapsed().as_millis(), best_move.map_or("".to_string(), |mve| mve.to_string().to_ascii_lowercase()));
    println!("info depth {} seldepth {} multipv {} score cp {} nodes {} nps {:.0} hashfull {} tbhits {} time {} pv {}", iterative_deepening_max_depth, max_depth_reached, 1, latest_eval, nodes_processed, nps, 0, 0, start_time.elapsed().as_millis(), best_move.map_or("".to_string(), |mve| mve.to_string().to_ascii_lowercase()));
}
