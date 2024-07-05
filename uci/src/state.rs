use statig::prelude::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{sync::atomic::AtomicBool, thread};

use engine::{search, GenerateMoves, Position, AUTHOR, NAME, POSITION_EVALUATOR};

use crate::messages::{UCICommand, UCIResponse};

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
            debug: false,
            maybe_terminate: None,
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
    fn unexpected_command(&mut self, event: &UCICommand) -> Response<State> {
        write_str_response(
            Arc::clone(&self.response_writer),
            &format!("Unexpected command for current state: {:?}", event),
        );
        Super
    }
    #[state(superstate = "unexpected_command")]
    fn initial(event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::UCI => Transition(State::uci_enabled()),
            _ => Super,
        }
    }
    #[superstate(superstate = "unexpected_command")]
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
                if self.maybe_terminate.is_some() {
                    write_str_response(
                        Arc::clone(&self.response_writer),
                        "Can't start new search until previous search completes",
                    );
                }
                let terminate = Arc::new(AtomicBool::new(false));
                self.maybe_terminate = Some(Arc::clone(&terminate));
                let search_position = position.clone();
                let move_gen = self.move_gen;
                let response_writer = Arc::clone(&self.response_writer);
                let params = params.clone();

                thread::spawn(move || {
                    let (best_move, _) = search(
                        &search_position,
                        &params,
                        move_gen,
                        POSITION_EVALUATOR,
                        Arc::clone(&response_writer),
                        Arc::clone(&terminate),
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
