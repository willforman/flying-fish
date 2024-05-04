use engine::position::{Move, Side};
use engine::search::search;
use statig::prelude::*;

use engine::evaluation::POSITION_EVALUATOR;
use engine::move_gen::{
    GenerateMoves, HyperbolaQuintessenceMoveGen, HYPERBOLA_QUINTESSENCE_MOVE_GEN,
};
use engine::{position::Position, AUTHOR, NAME};

use crate::messages::{UCIMessageToClient, UCIMessageToServer};

const SEARCH_DEPTH: u32 = 3;
static MOVE_GEN: HyperbolaQuintessenceMoveGen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

pub(crate) trait SendToUCIClient {
    fn send_client(&self, msgs: Vec<UCIMessageToClient>);
}

pub(crate) struct UCIState {
    pub(crate) client_sender: Box<dyn SendToUCIClient>,
    debug: bool,
}

#[state_machine(initial = "State::initial()", state(derive(PartialEq, Eq, Debug)))]
impl UCIState {
    #[state]
    fn initial(event: &UCIMessageToServer) -> Response<State> {
        match event {
            UCIMessageToServer::UCI => Transition(State::uci_enabled()),
            _ => Super,
        }
    }

    #[state(entry_action = "enter_uci_enabled")]
    fn uci_enabled(event: &UCIMessageToServer) -> Response<State> {
        match event {
            UCIMessageToServer::UCINewGame => Transition(State::new_game()),
            UCIMessageToServer::Position { fen, moves } => {
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
        self.client_sender.send_client(vec![
            UCIMessageToClient::ID {
                name: Some(NAME.to_string()),
                author: Some(AUTHOR.to_string()),
            },
            UCIMessageToClient::UCIOk,
        ]);
    }

    #[state]
    fn new_game(event: &UCIMessageToServer) -> Response<State> {
        match event {
            _ => Super,
        }
    }

    #[state]
    fn in_game(&self, position: &mut Position, event: &UCIMessageToServer) -> Response<State> {
        match event {
            UCIMessageToServer::Go { params: _ } => {
                let moves = MOVE_GEN.gen_moves(&position);

                let mut best_val = if position.state.to_move == Side::White {
                    f64::MIN
                } else {
                    f64::MAX
                };

                let mut best_move: Option<Move> = None;

                for mve in moves {
                    let mut move_position = position.clone();
                    move_position.make_move(&mve).unwrap();

                    let got_val =
                        search(&move_position, SEARCH_DEPTH, MOVE_GEN, POSITION_EVALUATOR);

                    if position.state.to_move == Side::White {
                        if got_val > best_val {
                            best_val = got_val;
                            best_move = Some(mve);
                        }
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        if got_val < best_val {
                            best_val = got_val;
                            best_move = Some(mve);
                        }
                    }
                }
                self.client_sender
                    .send_client(vec![UCIMessageToClient::BestMove {
                        mve: best_move.expect("Best move should have been found"),
                        ponder: None,
                    }]);
                Super
            }
            _ => Super,
        }
    }
}
