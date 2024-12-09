use statig::prelude::*;
use std::fmt::format;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
use std::{sync::atomic::AtomicBool, thread};

use engine::{
    perft_full, search, EvaluatePosition, GenerateMoves, Position, AUTHOR,
    HYPERBOLA_QUINTESSENCE_MOVE_GEN, NAME, POSITION_EVALUATOR,
};

use crate::messages::{UCICommand, UCIResponse};
use crate::LOGS_DIRECTORY;

pub(crate) struct UCIState<T, U>
where
    T: GenerateMoves + Copy + Send + Sync,
    U: Write + Send + Sync,
{
    move_gen: T,
    response_writer: Arc<Mutex<U>>,
    debug: bool,
    // We need a way to terminate when running Go, but unfortunately don't seem
    // to be able store this as statig state local storage because that requires the
    // item to be a reference.
    maybe_terminate: Option<Arc<AtomicBool>>,
    maybe_search_logs_path: Option<PathBuf>,
}

impl<T, U> UCIState<T, U>
where
    T: GenerateMoves + Copy + Send + Sync,
    U: Write + Send + Sync + 'static,
{
    pub(crate) fn new(move_gen: T, response_writer: Arc<Mutex<U>>) -> Self {
        Self {
            move_gen,
            response_writer,
            debug: true,
            maybe_terminate: None,
            maybe_search_logs_path: None,
        }
    }
}

#[state_machine(initial = "State::initial()", state(derive(PartialEq, Eq, Debug)))]
impl<T, U> UCIState<T, U>
where
    T: GenerateMoves + Copy + Send + Sync + 'static,
    U: Write + Send + Sync + 'static,
{
    #[superstate]
    fn top_level(&mut self, event: &UCICommand) -> Response<State> {
        if *event == UCICommand::Quit {
            process::exit(0);
        }
        write_str_response(
            Arc::clone(&self.response_writer),
            &format!("Unexpected command for current state: {:?}", event),
        );
        Super
    }
    #[state(superstate = "top_level")]
    fn initial(event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::UCI => Transition(State::uci_enabled()),
            _ => Super,
        }
    }
    #[superstate(superstate = "top_level")]
    fn debug(&mut self, event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::Debug { on } => {
                self.debug = *on;
                Handled
            }
            _ => Super,
        }
    }

    #[superstate(superstate = "debug")]
    fn is_ready(&mut self, event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::IsReady => {
                write_response(Arc::clone(&self.response_writer), UCIResponse::ReadyOk);
                Handled
            }
            _ => Super,
        }
    }

    #[state(entry_action = "enter_uci_enabled", superstate = "is_ready")]
    fn uci_enabled(event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::UCINewGame => Transition(State::in_game(Position::start())),
            _ => Super,
        }
    }

    #[action]
    fn enter_uci_enabled(&mut self) {
        write_response(
            Arc::clone(&self.response_writer),
            UCIResponse::IDName {
                name: NAME.to_string(),
            },
        );
        write_response(
            Arc::clone(&self.response_writer),
            UCIResponse::IDAuthor {
                author: AUTHOR.to_string(),
            },
        );
        write_response(Arc::clone(&self.response_writer), UCIResponse::UCIOk);

        if self.debug {
            let curr_time_str = chrono::Local::now().format("%I_%M_%m_%d");
            let log_name = format!("search-{}.txt", curr_time_str);
            let mut search_logs_path = LOGS_DIRECTORY
                .get()
                .expect("LOGS_DIRECTORY should be set")
                .clone();
            search_logs_path.push("search");
            search_logs_path.push(log_name);
            self.maybe_search_logs_path = Some(search_logs_path);
        }
    }

    #[state(superstate = "is_ready")]
    fn in_game(&mut self, position: &mut Position, event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::UCINewGame => Transition(State::in_game(Position::start())),
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
                Transition(State::in_game(pos))
            }
            UCICommand::Go { params } => {
                if let Some(terminate) = &self.maybe_terminate {
                    if !terminate.load(std::sync::atomic::Ordering::Relaxed) {
                        write_str_response(
                            Arc::clone(&self.response_writer),
                            "Can't start new search until previous search completes",
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
                    debug: self.debug,
                    ..params.clone()
                };
                let maybe_search_logs_path = self.maybe_search_logs_path.clone();

                thread::spawn(move || {
                    let (best_move, _) = search(
                        &search_position,
                        &params,
                        move_gen,
                        POSITION_EVALUATOR,
                        Arc::clone(&response_writer),
                        Arc::clone(&terminate),
                        maybe_search_logs_path,
                    )
                    .unwrap();
                    write_response(
                        response_writer,
                        UCIResponse::BestMove {
                            mve: best_move.expect("Best move should have been found"),
                            ponder: None,
                        },
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
                    Arc::clone(&self.response_writer),
                    format!("info string {}", eval / 100.).as_str(),
                );
                Handled
            }
            UCICommand::Perft { depth } => {
                let perft_results = perft_full(position, *depth, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
                write_str_response(
                    Arc::clone(&self.response_writer),
                    format!("{}", perft_results).as_str(),
                );
                Handled
            }
            _ => Super,
        }
    }
}

fn write_response(response_writer: Arc<Mutex<impl Write>>, uci_response: UCIResponse) {
    // Helper method to reduce boilerplate for writing response
    let res_str: String = uci_response.into();
    write_str_response(response_writer, &res_str);
}

fn write_str_response(response_writer: Arc<Mutex<impl Write>>, str_response: &str) {
    let mut response_writer = response_writer.lock().unwrap();
    writeln!(response_writer, "{}", str_response).unwrap();
}
