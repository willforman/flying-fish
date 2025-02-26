use anyhow::{Context, Result};
use statig::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{sync::atomic::AtomicBool, thread};

use engine::{
    perft, perft_full, search, EvaluatePosition, GenerateMoves, Move, Position, AUTHOR,
    HYPERBOLA_QUINTESSENCE_MOVE_GEN, NAME, POSITION_EVALUATOR,
};

use crate::messages::{UCICommand, UCIResponse};
use crate::LOGS_DIRECTORY;

#[derive(Debug)]
struct DebugItems {
    search_logs_writer: Arc<Mutex<File>>,
    in_out_logs_writer: Arc<Mutex<File>>,
}

impl DebugItems {
    fn init() -> Result<Self> {
        let curr_time_str = chrono::Local::now().format("%I_%M_%m_%d");
        let logs_directory = LOGS_DIRECTORY
            .get()
            .expect("LOGS_DIRECTORY should be set")
            .clone();

        let mut search_logs_path = logs_directory.clone();
        search_logs_path.push("search");
        search_logs_path.push(format!("search-{}.txt", curr_time_str));
        let search_logs_writer = File::create(&search_logs_path)
            .context(format!("Couldn't open: {:?}", &search_logs_path))?;

        let mut in_out_logs_path = logs_directory.clone();
        in_out_logs_path.push("in_out");
        in_out_logs_path.push(format!("in_out-{}.txt", curr_time_str));
        let in_out_logs_writer = File::create(&in_out_logs_path)
            .context(format!("Couldn't open: {:?}", &in_out_logs_path))?;

        Ok(Self {
            search_logs_writer: Arc::new(Mutex::new(search_logs_writer)),
            in_out_logs_writer: Arc::new(Mutex::new(in_out_logs_writer)),
        })
    }
}

#[derive(Debug)]
pub(crate) struct UCIState<G, W>
where
    G: GenerateMoves + Copy + Send + Sync,
    W: Write + Send + Sync,
{
    move_gen: G,
    response_writer: Arc<Mutex<W>>,
    // We need a way to terminate when running Go, but unfortunately don't seem
    // to be able store this as statig state local storage because that requires the
    // item to be a reference.
    maybe_terminate: Option<Arc<AtomicBool>>,
    debug: Option<DebugItems>,
}

impl<G, W> UCIState<G, W>
where
    G: GenerateMoves + Copy + Send + Sync + 'static,
    W: Write + Send + Sync + 'static,
{
    pub(crate) fn new(move_gen: G, response_writer: Arc<Mutex<W>>) -> Self {
        Self {
            move_gen,
            response_writer,
            maybe_terminate: None,
            debug: None,
        }
    }

    fn on_dispatch(&mut self, _: StateOrSuperstate<UCIState<G, W>>, event: &UCICommand) {
        if let Some(dbg_items) = &mut self.debug {
            let mut in_out_logs_writer = dbg_items.in_out_logs_writer.lock().unwrap();
            writeln!(in_out_logs_writer, "> {}", event).unwrap();
        }
    }
}

#[state_machine(
    initial = "State::initial()",
    on_dispatch = "Self::on_dispatch",
    state(derive(PartialEq, Eq, Debug)),
    superstate(derive(Debug))
)]
impl<G, W> UCIState<G, W>
where
    G: GenerateMoves + Copy + Send + Sync + 'static,
    W: Write + Send + Sync + 'static,
{
    #[superstate]
    fn top_level(&mut self, event: &UCICommand) -> Response<State> {
        if *event == UCICommand::Quit {
            process::exit(0);
        }
        let maybe_in_out_logs_writer = self
            .debug
            .as_ref()
            .map(|debug| Arc::clone(&debug.in_out_logs_writer));

        write_str_response(
            &format!("Unexpected command for current state: {:?}", event),
            Arc::clone(&self.response_writer),
            maybe_in_out_logs_writer,
        );
        Super
    }
    #[state(superstate = "top_level")]
    fn initial(&mut self, event: &UCICommand) -> Response<State> {
        let mut maybe_in_out_logs_writer = self
            .debug
            .as_mut()
            .map(|debug| Arc::clone(&debug.in_out_logs_writer));

        match event {
            UCICommand::UCI => {
                write_response(
                    UCIResponse::IDName {
                        name: NAME.to_string(),
                    },
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
                );
                write_response(
                    UCIResponse::IDAuthor {
                        author: AUTHOR.to_string(),
                    },
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
                );
                write_response(
                    UCIResponse::UCIOk,
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer,
                );

                Transition(State::uci_enabled(Position::start()))
            }
            _ => Super,
        }
    }
    #[superstate(superstate = "top_level")]
    fn debug(&mut self, event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::Debug { on: true } => {
                self.debug = Some(DebugItems::init().unwrap());
                Handled
            }
            UCICommand::Debug { on: false } => {
                self.debug = None;
                Handled
            }
            _ => Super,
        }
    }

    #[superstate(superstate = "debug")]
    fn is_ready(&mut self, event: &UCICommand) -> Response<State> {
        let maybe_in_out_logs_writer = self
            .debug
            .as_mut()
            .map(|debug| Arc::clone(&debug.in_out_logs_writer));

        match event {
            UCICommand::IsReady => {
                write_response(
                    UCIResponse::ReadyOk,
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer,
                );
                Handled
            }
            _ => Super,
        }
    }

    #[state(superstate = "is_ready")]
    fn uci_enabled(&mut self, position: &mut Position, event: &UCICommand) -> Response<State> {
        let maybe_in_out_logs_writer = self
            .debug
            .as_mut()
            .map(|debug| Arc::clone(&debug.in_out_logs_writer));

        match event {
            UCICommand::UCINewGame => Transition(State::uci_enabled(Position::start())),
            UCICommand::Position { fen, moves } => {
                let mut pos = match fen {
                    Some(fen) => Position::from_fen(fen).unwrap(),
                    None => Position::start(),
                };
                if let Some(moves) = moves {
                    for mve in moves {
                        pos.make_move(mve).unwrap();
                    }
                }
                Transition(State::uci_enabled(pos))
            }
            UCICommand::Go { params } => {
                if let Some(terminate) = &self.maybe_terminate {
                    if !terminate.load(std::sync::atomic::Ordering::Relaxed) {
                        write_str_response(
                            "Can't start new search until previous search completes",
                            Arc::clone(&self.response_writer),
                            maybe_in_out_logs_writer,
                        );
                        return Handled;
                    }
                }
                let terminate = Arc::new(AtomicBool::new(false));
                self.maybe_terminate = Some(Arc::clone(&terminate));
                let search_position = position.clone();
                let move_gen = self.move_gen;
                let response_writer = Arc::clone(&self.response_writer);
                let params = engine::SearchParams {
                    debug: self.debug.is_some(),
                    ..params.clone()
                };

                let maybe_search_logs_writer = self
                    .debug
                    .as_ref()
                    .map(|debug| Arc::clone(&debug.search_logs_writer));

                thread::spawn(move || {
                    let (best_move, _) = search(
                        &search_position,
                        &params,
                        move_gen,
                        POSITION_EVALUATOR,
                        Arc::clone(&response_writer),
                        Arc::clone(&terminate),
                        maybe_search_logs_writer,
                    )
                    .unwrap();
                    write_response(
                        UCIResponse::BestMove {
                            mve: best_move.expect("Best move should have been found"),
                            ponder: None,
                        },
                        response_writer,
                        maybe_in_out_logs_writer,
                    );
                    terminate.store(true, std::sync::atomic::Ordering::Relaxed);
                });
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
                log::debug!("Exiting with position fen: {}", position.to_fen());
                process::exit(0);
            }
            UCICommand::Eval => {
                let eval = POSITION_EVALUATOR.evaluate(position, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
                write_str_response(
                    format!("info string {}", eval / 100.).as_str(),
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer,
                );
                Handled
            }
            UCICommand::Perft { depth } => {
                let start = Instant::now();
                let (move_counts, total_count) =
                    perft(position, *depth, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
                let time_elapsed = start.elapsed();

                write_perft_results(
                    move_counts,
                    total_count,
                    time_elapsed,
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer,
                );
                Handled
            }
            UCICommand::PerftFull { depth } => {
                let perft_results = perft_full(position, *depth, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
                write_str_response(
                    format!("{}", perft_results).as_str(),
                    Arc::clone(&self.response_writer),
                    maybe_in_out_logs_writer,
                );
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
        let mut maybe_in_out_logs_writer = self
            .debug
            .as_mut()
            .map(|debug| Arc::clone(&debug.in_out_logs_writer));

        let total_start = Instant::now();
        let mut total_nodes = 0;
        for (fen, depth) in PERFT_BENCHMARK_FENS_AND_DEPTHS {
            write_str_response(
                &format!("Position: [{}], depth {}", fen, depth),
                Arc::clone(&self.response_writer),
                maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
            );

            let position = Position::from_fen(fen)?;
            let position_start = Instant::now();
            let (position_move_nodes, position_total_nodes) =
                perft(&position, *depth, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
            let position_time_elapsed = position_start.elapsed();

            total_nodes += position_total_nodes;

            write_perft_results(
                position_move_nodes,
                position_total_nodes,
                position_time_elapsed,
                Arc::clone(&self.response_writer),
                maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
            );
        }
        let total_time_elapsed = total_start.elapsed();
        let total_nodes_per_second = total_nodes as f64 / total_time_elapsed.as_secs_f64();
        write_str_response(
            "\n===========================",
            Arc::clone(&self.response_writer),
            maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
        );
        write_str_response(
            &format!("Total time (ms): {}", total_time_elapsed.as_millis()),
            Arc::clone(&self.response_writer),
            maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
        );
        write_str_response(
            &format!("Nodes searched: {}", total_nodes),
            Arc::clone(&self.response_writer),
            maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
        );
        write_str_response(
            &format!("Nodes/second: {:.0}", total_nodes_per_second),
            Arc::clone(&self.response_writer),
            maybe_in_out_logs_writer,
        );
        Ok(())
    }
}

fn write_response(
    uci_response: UCIResponse,
    response_writer: Arc<Mutex<impl Write>>,
    maybe_in_out_logs_writer: Option<Arc<Mutex<impl Write>>>,
) {
    // Helper method to reduce boilerplate for writing response
    let res_str: String = uci_response.into();
    write_str_response(
        &res_str,
        Arc::clone(&response_writer),
        maybe_in_out_logs_writer,
    );
}

fn write_str_response(
    str_response: &str,
    response_writer: Arc<Mutex<impl Write>>,
    maybe_in_out_logs_writer: Option<Arc<Mutex<impl Write>>>,
) {
    let mut response_writer = response_writer.lock().unwrap();
    writeln!(response_writer, "{}", str_response).unwrap();
    if let Some(in_out_logs_writer) = maybe_in_out_logs_writer {
        let mut in_out_logs_writer_guard = in_out_logs_writer.lock().unwrap();
        writeln!(in_out_logs_writer_guard, "< {}", str_response).unwrap();
    }
}

fn write_perft_results(
    move_counts: HashMap<Move, usize>,
    total_count: usize,
    time_elapsed: Duration,
    response_writer: Arc<Mutex<impl Write>>,
    mut maybe_in_out_logs_writer: Option<Arc<Mutex<impl Write>>>,
) {
    let move_counts_str = move_counts
        .iter()
        .map(|(mve, count)| format!("{}: {}", mve, count))
        .collect::<Vec<_>>()
        .join("\n");
    let nodes_per_second = total_count as f64 / time_elapsed.as_secs_f64();

    write_str_response(
        &move_counts_str,
        Arc::clone(&response_writer),
        maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
    );
    write_str_response(
        "",
        Arc::clone(&response_writer),
        maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
    );
    write_str_response(
        &format!("Time (ms): {}", time_elapsed.as_millis()),
        Arc::clone(&response_writer),
        maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
    );
    write_str_response(
        &format!("Nodes searched: {}", total_count),
        Arc::clone(&response_writer),
        maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
    );
    write_str_response(
        &format!("Nodes/second: {:.0}", nodes_per_second),
        Arc::clone(&response_writer),
        maybe_in_out_logs_writer.as_mut().map(|itm| Arc::clone(itm)),
    );
}

const PERFT_BENCHMARK_FENS_AND_DEPTHS: &[(&str, usize)] = &[
    (
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        4,
    ),
    (
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        4,
    ),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5),
];
