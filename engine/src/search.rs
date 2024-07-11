use core::panic;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::evaluation::EvaluatePosition;
use crate::move_gen::GenerateMoves;
use crate::position::{Move, Position};
use crate::Side;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
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
    pub debug: bool,
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
        if self.debug != default.debug {
            parts.push(format!("debug: {:?}", self.debug));
        }
        write!(f, "SearchParams: {}", parts.join(", "))
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

fn calc_time_per_move(time_left: Duration, time_inc: Duration, moves_to_go: u16) -> Duration {
    time_left / moves_to_go.into()
}

pub fn search(
    position: &Position,
    params: &SearchParams,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    info_writer: Arc<Mutex<impl Write>>,
    terminate: Arc<AtomicBool>,
    search_logs_path: Option<PathBuf>,
) -> Result<(Option<Move>, SearchResultInfo), SearchError> {
    if params.debug {
        writeln!(info_writer.lock().unwrap(), "info string {}", params).unwrap();
        log::debug!("{}", params);
    }
    let mut best_move: Option<Move> = None;
    let mut best_val: Option<Move> = None;

    let mut positions_processed: u64 = 0;
    let start = Instant::now();
    let mut latest_score = 0.0;

    let max_depth: usize = match (params.max_depth, params.mate) {
        (Some(max_depth), None) => max_depth.try_into().unwrap(),
        (None, Some(mate)) => mate.try_into().unwrap(),
        (Some(max_depth), Some(mate)) => {
            return Err(SearchError::DepthAndMatePassed(max_depth, mate))
        }
        (None, None) => 300,
    };

    let time_to_use = if position.state.to_move == Side::White {
        calc_time_per_move(
            params.white_time.unwrap_or(Duration::from_secs(60)),
            params.white_inc.unwrap_or(Duration::from_secs(0)),
            params.moves_to_go.unwrap_or(1),
        )
    } else {
        calc_time_per_move(
            params.black_time.unwrap_or(Duration::from_secs(60)),
            params.black_inc.unwrap_or(Duration::from_secs(0)),
            params.moves_to_go.unwrap_or(1),
        )
    };
    if params.debug {
        writeln!(
            info_writer.lock().unwrap(),
            "info string time_for_this_move {:?}",
            time_to_use
        )
        .unwrap();
        log::debug!("time for this move: {:?}", time_to_use);
    }

    let mut moves = move_gen.gen_moves(position);

    // Filter out moves not in search moves
    if let Some(search_moves) = &params.search_moves {
        moves.retain(|mve| search_moves.contains(mve));
    }

    let move_positions: HashMap<Move, Position> = moves
        .iter()
        .map(|mve| {
            let mut move_position = position.clone();
            move_position.make_move(mve).unwrap();
            (*mve, move_position)
        })
        .collect();

    let mut maybe_search_logs_file = if let Some(search_logs_path) = &search_logs_path {
        Some(
            File::create(search_logs_path)
                .map_err(|_| SearchError::OpenSearchLogsFile(search_logs_path.to_path_buf()))?,
        )
    } else {
        None
    };

    if let Some(search_logs_file) = &mut maybe_search_logs_file {
        writeln!(search_logs_file, "NEW SEARCH").unwrap();
        writeln!(search_logs_file, "{}", position.to_fen()).unwrap();
        writeln!(search_logs_file, "{}", params).unwrap();
    }

    'outer: for iterative_deepening_max_depth in 1..=max_depth {
        let iterative_deepening_max_depth: u64 = iterative_deepening_max_depth.try_into().unwrap();

        // Find value of each move up to current depth
        let mut move_vals = HashMap::with_capacity(moves.len());
        for mve in moves.clone() {
            println!("move: {}", mve);
            let move_position = &move_positions[&mve];
            let (mut move_val, search_complete) = search_helper(
                move_position,
                params,
                1,
                iterative_deepening_max_depth,
                &mut positions_processed,
                &start,
                &mut latest_score,
                f64::MIN,
                f64::MAX,
                move_gen,
                position_eval,
                Arc::clone(&info_writer),
                Arc::clone(&terminate),
            );
            // Since this is after making a move, flip the value to get the value
            // relative to the side of `position`
            move_val = -move_val;
            move_vals.insert(mve, move_val);

            if search_complete {
                write_search_info(
                    iterative_deepening_max_depth,
                    positions_processed,
                    iterative_deepening_max_depth,
                    &start,
                    &latest_score,
                    info_writer,
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

        latest_score = move_vals[&best_move.unwrap()];

        write_search_info(
            iterative_deepening_max_depth,
            positions_processed,
            iterative_deepening_max_depth,
            &start,
            &latest_score,
            Arc::clone(&info_writer),
        );

        if params.debug {
            log::debug!("best move: {}, score: {}", best_move.unwrap(), latest_score);
        }

        if let Some(search_logs_file) = &mut maybe_search_logs_file {
            for mve in &moves {
                writeln!(search_logs_file, "{}: {}", mve, move_vals[mve])
                    .expect("Write move to search logs failed");
            }
            writeln!(search_logs_file, "==================================")
                .expect("Write to search logs failed");
        }

        // Break search if:
        // - Engine has gone past alotted time
        if start.elapsed() > time_to_use {
            break;
        }
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
    iterative_deepening_max_depth: u64,
    positions_processed: &mut u64,
    start_time: &Instant,
    latest_score: &mut f64,
    mut alpha: f64,
    beta: f64,
    move_gen: impl GenerateMoves + std::marker::Copy,
    position_eval: impl EvaluatePosition + std::marker::Copy,
    info_writer: Arc<Mutex<impl Write>>,
    terminate: Arc<AtomicBool>,
) -> (f64, bool) {
    // If this search has been terminated, return early
    if terminate.load(std::sync::atomic::Ordering::Relaxed) {
        return (0.0, true);
    }
    // If this search is at the max number of nodes, return early
    if let Some(max_nodes) = params.max_nodes {
        debug_assert!(*positions_processed <= max_nodes);
        if *positions_processed == max_nodes {
            return (0.0, true);
        }
    }
    // If search has exceeded total time, return early
    if let Some(move_time) = params.move_time {
        if start_time.elapsed() >= move_time {
            return (0.0, true);
        }
    }
    *positions_processed += 1;

    if *positions_processed % 250_000 == 0 {
        write_search_info(
            iterative_deepening_max_depth,
            *positions_processed,
            curr_depth,
            start_time,
            latest_score,
            Arc::clone(&info_writer),
        );
    }

    if curr_depth == iterative_deepening_max_depth {
        let curr_evaluation = position_eval.evaluate(position, move_gen);
        println!("got eval: {}", curr_evaluation);
        return (curr_evaluation, false);
    }

    let moves = move_gen.gen_moves(position);

    let mut best_val = f64::MIN;
    for mve in moves {
        let mut move_position = position.clone();
        let move_res = move_position.make_move(&mve);
        if let Err(err) = move_res {
            write_search_info(
                iterative_deepening_max_depth,
                *positions_processed,
                curr_depth,
                start_time,
                latest_score,
                Arc::clone(&info_writer),
            );
            let mut info_writer = info_writer.lock().unwrap();
            writeln!(info_writer, "Error for move {}: {}", mve, err).unwrap();
            panic!("Err encountered searching, exiting");
        }

        let (got_val, search_complete) = search_helper(
            &move_position,
            params,
            curr_depth + 1,
            iterative_deepening_max_depth,
            positions_processed,
            start_time,
            latest_score,
            -beta,
            -alpha,
            move_gen,
            position_eval,
            Arc::clone(&info_writer),
            Arc::clone(&terminate),
        );

        // If child node is signaling search is terminated, better terminate self
        if search_complete {
            return (best_val, true);
        }

        // Then, flip value because it was relative to the other side
        let got_val = -got_val;

        if got_val >= best_val {
            best_val = got_val;
            alpha = f64::max(alpha, got_val);
        }

        if alpha >= beta {
            break;
        }
    }

    (best_val, false)
}

fn write_search_info(
    iterative_deepening_max_depth: u64,
    nodes_processed: u64,
    curr_depth: u64,
    start_time: &Instant,
    latest_score: &f64,
    info_writer: Arc<Mutex<impl Write>>,
) {
    // info depth 10 seldepth 6 multipv 1 score mate 3 nodes 971 nps 121375 hashfull 0 tbhits 0 time 8 pv f4g3 e6d6 d2d6 h1g1 d6d1
    let nps = nodes_processed as f32 / start_time.elapsed().as_secs_f32();
    let info = format!("info depth {} seldepth {} multipv {} score {} nodes {} nps {:.0} hashfull {} tbhits {} time {} pv {}", iterative_deepening_max_depth, curr_depth, 1, latest_score, nodes_processed, nps, 0, 0, start_time.elapsed().as_millis(), "");
    let mut info_writer = info_writer.lock().unwrap();
    writeln!(info_writer, "{}", info).unwrap();
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

    #[derive(Clone, Copy)]
    struct DummyInfoWriter;

    impl Write for DummyInfoWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let string = String::from_utf8(buf.to_vec()).unwrap();
            println!("{}", string);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

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
            Arc::new(Mutex::new(DummyInfoWriter)),
            Arc::new(AtomicBool::new(false)),
            None,
        )?;
        Ok(())
    }
}
