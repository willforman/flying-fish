use statig::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use engine::{search, GenerateMoves, Position, AUTHOR, NAME, POSITION_EVALUATOR};

use crate::messages::{UCICommand, UCIResponse, WriteUCIResponse};

pub(crate) struct UCIState<T, U>
where
    T: GenerateMoves,
    U: WriteUCIResponse,
{
    move_gen: T,
    response_writer: U,
    debug: bool,
    // We need a way to terminate when running Go, but unfortunately don't seem
    // to be able store this as state local storage because that requires the
    // item to be a reference.
    maybe_terminate: Option<Arc<AtomicBool>>,
}

impl<T, U> UCIState<T, U>
where
    T: GenerateMoves + Copy,
    U: WriteUCIResponse,
{
    pub(crate) fn new(move_gen: T, response_writer: U) -> Self {
        Self {
            move_gen,
            response_writer,
            debug: false,
            maybe_terminate: None,
        }
    }

    fn write_response(&self, uci_response: UCIResponse) {
        self.response_writer.write_uci_response(uci_response.into());
    }
}

#[state_machine(initial = "State::initial()", state(derive(PartialEq, Eq, Debug)))]
impl<T, U> UCIState<T, U>
where
    T: GenerateMoves + Copy,
    U: WriteUCIResponse,
{
    #[state]
    fn initial(event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::UCI => Transition(State::uci_enabled()),
            _ => Super,
        }
    }

    #[state(entry_action = "enter_uci_enabled")]
    fn uci_enabled(event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::UCINewGame => Transition(State::new_game()),
            UCICommand::Position { fen, moves } => {
                let mut pos = match fen {
                    Some(fen) => Position::from_fen(fen).unwrap(),
                    None => Position::start(),
                };
                for mve in moves {
                    pos.make_move(mve);
                }
                Transition(State::in_game(pos))
            }
            _ => Super,
        }
    }

    #[action]
    fn enter_uci_enabled(&self) {
        self.write_response(UCIResponse::ID {
            name: Some(NAME.to_string()),
            author: Some(AUTHOR.to_string()),
        });
        self.write_response(UCIResponse::UCIOk);
    }

    #[state]
    fn new_game(event: &UCICommand) -> Response<State> {
        match event {
            _ => Super,
        }
    }

    #[state]
    fn in_game(&mut self, position: &mut Position, event: &UCICommand) -> Response<State> {
        match event {
            UCICommand::Go { params: _ } => {
                assert!(self.maybe_terminate.is_none());
                let terminate = Arc::new(AtomicBool::new(false));
                self.maybe_terminate = Some(Arc::clone(&terminate));

                let best_move = search(
                    &position,
                    10,
                    self.move_gen,
                    POSITION_EVALUATOR,
                    Arc::clone(&terminate),
                );
                self.write_response(UCIResponse::BestMove {
                    mve: best_move.expect("Best move should have been found"),
                    ponder: None,
                });
                Super
            }
            _ => Super,
        }
    }
}
