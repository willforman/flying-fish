use anyhow::Result;
use statig::prelude::*;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::panic;
use std::process;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{sync::atomic::AtomicBool, thread};
use tracing::{debug, error, warn};

use engine::{
    AUTHOR, EvaluatePosition, GenerateMoves, MOVE_GEN, Move, NAME, POSITION_EVALUATOR, Position,
    SearchError, SearchParams, TranspositionTable, perft, perft_full, search,
};

use crate::messages::{UCICommand, UCIResponse};
use crate::uci;

#[derive(Debug)]
pub(crate) struct UCIState<G>
where
    G: GenerateMoves + Copy + Send + Sync,
{
    move_gen: G,
    transposition_table: Arc<Mutex<TranspositionTable>>,
    // We need a way to terminate when running Go, but unfortunately don't seem
    // to be able store this as statig state local storage because that requires the
    // item to be a reference.
    maybe_terminate: Option<Arc<AtomicBool>>,
}

impl<G> UCIState<G>
where
    G: GenerateMoves + Copy + Send + Sync + 'static,
{
    pub(crate) fn new(move_gen: G) -> Self {
        Self {
            move_gen,
            transposition_table: Arc::new(Mutex::new(TranspositionTable::new())),
            maybe_terminate: None,
        }
    }
}

#[state_machine(
    initial = "State::initial()",
    state(derive(PartialEq, Eq, Debug)),
    superstate(derive(Debug))
)]
impl<G> UCIState<G>
where
    G: GenerateMoves + Copy + Send + Sync + 'static,
{
    #[superstate]
    fn top_level(&mut self, event: &UCICommand) -> Outcome<State> {
        if *event == UCICommand::Quit {
            process::exit(0);
        }

        warn!(target: "uci", "Unexpected command for current state: {}", event.to_string());

        Super
    }
    #[state(superstate = "top_level")]
    fn initial(&mut self, event: &UCICommand) -> Outcome<State> {
        match event {
            UCICommand::UCI => {
                uci!(
                    "{}",
                    &UCIResponse::IDName {
                        name: NAME.to_string()
                    }
                );
                uci!(
                    "{}",
                    &UCIResponse::IDAuthor {
                        author: AUTHOR.to_string(),
                    },
                );
                uci!("{}", UCIResponse::UCIOk);

                Transition(State::uci_enabled(Position::start()))
            }
            _ => Super,
        }
    }
    #[superstate(superstate = "top_level")]
    fn debug(&mut self, event: &UCICommand) -> Outcome<State> {
        match event {
            UCICommand::Debug { on: true } => Handled,
            UCICommand::Debug { on: false } => Handled,
            _ => Super,
        }
    }

    #[superstate(superstate = "debug")]
    fn is_ready(&mut self, event: &UCICommand) -> Outcome<State> {
        match event {
            UCICommand::IsReady => {
                uci!("{}", UCIResponse::ReadyOk);
                Handled
            }
            _ => Super,
        }
    }

    #[state(superstate = "is_ready")]
    fn uci_enabled(&mut self, position: &mut Position, event: &UCICommand) -> Outcome<State> {
        match event {
            UCICommand::UCINewGame => Transition(State::uci_enabled(Position::start())),
            UCICommand::Position { fen, moves } => {
                let mut pos = match fen {
                    Some(fen) => Position::from_fen(fen).unwrap(),
                    None => Position::start(),
                };
                if let Some(moves) = moves {
                    for mve in moves {
                        pos.make_move(*mve);
                    }
                }
                Transition(State::uci_enabled(pos))
            }
            UCICommand::Go { params } => {
                if let Some(terminate) = &self.maybe_terminate {
                    if !terminate.load(std::sync::atomic::Ordering::Relaxed) {
                        warn!(target: "uci", "Can't start new search until previous search completes");
                        return Handled;
                    }
                }
                let terminate = Arc::new(AtomicBool::new(false));

                spawn_search(
                    position.clone(),
                    params.clone(),
                    self.move_gen,
                    Arc::clone(&self.transposition_table),
                    terminate,
                );

                Handled
            }
            UCICommand::Stop => {
                if let Some(terminate) = &self.maybe_terminate {
                    terminate.store(true, std::sync::atomic::Ordering::Relaxed);
                    self.maybe_terminate = None;
                } else {
                    panic!("maybe_terminate should not be None when stop is received");
                }
                Handled
            }
            UCICommand::Quit => {
                debug!("Exiting with position fen: {}", position.to_fen());
                process::exit(0);
            }
            UCICommand::Eval => {
                let eval = POSITION_EVALUATOR.evaluate(position, MOVE_GEN);
                uci!("uci string {}", eval);
                Handled
            }
            UCICommand::Perft { depth } => {
                let start = Instant::now();
                let (move_counts, total_count) = perft(position, *depth, MOVE_GEN);
                let time_elapsed = start.elapsed();

                write_perft_results(move_counts, total_count, time_elapsed);
                Handled
            }
            UCICommand::PerftFull { depth } => {
                let perft_results = perft_full(position, *depth, MOVE_GEN);
                uci!("{}", perft_results);
                Handled
            }
            UCICommand::PerftBenchmark => {
                self.perft_benchmark().unwrap();
                Handled
            }
            _ => Super,
        }
    }

    fn perft_benchmark(&mut self) -> Result<()> {
        let mut nps_res = vec![];
        for (fen, depth) in PERFT_BENCHMARK_FENS_AND_DEPTHS {
            uci!("Position: [{}], depth {}", fen, depth);

            let position = Position::from_fen(fen)?;
            let position_start = Instant::now();
            let (position_move_nodes, position_total_nodes) = perft(&position, *depth, MOVE_GEN);
            let position_time_elapsed = position_start.elapsed();

            let nps = position_total_nodes as f64 / position_time_elapsed.as_secs_f64();
            nps_res.push(nps);

            write_perft_results(
                position_move_nodes,
                position_total_nodes,
                position_time_elapsed,
            );
        }
        let avg_nps = nps_res.iter().sum::<f64>() / nps_res.len() as f64;

        uci!("\n===========================");
        uci!("Average nodes/second: {:.0}", avg_nps);
        Ok(())
    }
}

fn write_perft_results(
    move_counts: HashMap<Move, usize>,
    total_count: usize,
    time_elapsed: Duration,
) {
    let move_counts_str = move_counts
        .iter()
        .map(|(mve, count)| format!("{}: {}", mve, count))
        .collect::<Vec<_>>()
        .join("\n");
    let nodes_per_second = total_count as f64 / time_elapsed.as_secs_f64();

    uci!("{}", move_counts_str);
    uci!("");
    uci!("Time (ms): {}", time_elapsed.as_millis());
    uci!("Nodes searched: {}", total_count);
    uci!("Nodes/second: {:.0}", nodes_per_second);
}

const PERFT_BENCHMARK_FENS_AND_DEPTHS: &[(&str, usize)] = &[
    (
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        6,
    ),
    (
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        5,
    ),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 7),
];

fn spawn_search(
    search_position: Position,
    params: SearchParams,
    move_gen: impl GenerateMoves + Copy + Send + Sync + 'static,
    transposition_table: Arc<Mutex<TranspositionTable>>,
    terminate: Arc<AtomicBool>,
) {
    let panic_info = Arc::new(Mutex::new(None));
    let panic_info_clone = Arc::clone(&panic_info);

    let search_thread_handle = thread::spawn(move || -> Result<(), SearchError> {
        panic::set_hook(Box::new(move |info| {
            let location = if let Some(location) = info.location() {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            } else {
                "unknown location".to_string()
            };

            let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic message".to_string()
            };

            let backtrace = Backtrace::force_capture();

            *panic_info_clone.lock().unwrap() = Some((message, location, backtrace));
        }));

        let transposition_table_arc = Arc::clone(&transposition_table);
        let mut transposition_table = transposition_table_arc.lock().unwrap_or_else(|poisoned| {
            warn!("Transposition table was poisened, clearing it");
            let mut inner = poisoned.into_inner();
            *inner = TranspositionTable::new();
            inner
        });

        let (best_move, _) = search(
            &search_position,
            &params,
            move_gen,
            POSITION_EVALUATOR,
            &mut transposition_table,
            Arc::clone(&terminate),
        )?;
        uci!(
            "{}",
            &UCIResponse::BestMove {
                mve: best_move.expect("Best move should have been found"),
                ponder: None,
            }
        );
        terminate.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    });

    thread::spawn(move || {
        match search_thread_handle.join() {
            Ok(Ok(())) => {
                // Thread finished normally
            }
            Ok(Err(search_error)) => {
                error!(target: "uci", "Search thread error: {}", search_error);
                // Send invalid move so that client realizes we forfeited.
                uci!("bestmove 0000");
            }
            Err(_) => {
                if let Some((message, location, backtrace)) = panic_info.lock().unwrap().take() {
                    error!(target: "uci", "Search thread panicked at {}: {}\nBacktrace:\n{}", location, message, backtrace);
                } else {
                    error!(target: "uci", "Search thread panicked with unknown payload");
                }

                uci!("bestmove 0000");
            }
        }
    });
}
