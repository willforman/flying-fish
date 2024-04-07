use statig::prelude::*;

use engine::{AUTHOR, NAME};

use crate::messages::{UCIMessageToClient, UCIMessageToServer};

pub(crate) trait SendToUCIClient {
    fn send_client(&self, msgs: Vec<UCIMessageToClient>);
}

pub(crate) struct UCIState {
    pub(crate) client_sender: Box<dyn SendToUCIClient>,
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
        println!("{:?}", event);
        Super
    }

    #[action]
    fn enter_uci_enabled(&self) {
        let msg = UCIMessageToClient::ID {
            name: Some(NAME.to_string()),
            author: Some(AUTHOR.to_string()),
        };
        self.client_sender.send_client(vec![msg]);
    }
}
