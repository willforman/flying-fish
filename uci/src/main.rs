use std::io::{self, BufRead};
use std::str::FromStr;

use anyhow::Result;
use messages::UCIMessageToServer;
use simple_logger::SimpleLogger;
use statig::prelude::{InitializedStateMachine, IntoStateMachineExt};

mod messages;
mod state;

use crate::messages::UCIMessageToClient;
use crate::state::{SendToUCIClient, UCIState};

#[derive(Default)]
struct MessagePrinter;

impl SendToUCIClient for MessagePrinter {
    fn send_client(&self, msgs: Vec<UCIMessageToClient>) {
        for msg in msgs {
            println!("{:?}", msg);
        }
    }
}

struct UCI {
    state_machine: InitializedStateMachine<UCIState>,
}

impl UCI {
    fn send_server(&mut self, msg: UCIMessageToServer) {
        self.state_machine.handle(&msg);
    }
}

fn receive_messages(mut uci: UCI) -> Result<()> {
    let stdin = io::stdin();
    loop {
        for line in stdin.lock().lines() {
            let msg = UCIMessageToServer::from_str(line?.as_str())?;
            uci.send_server(msg);
        }
    }
}

fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();
    let message_printer = MessagePrinter::default();
    let uci_state = UCIState {
        client_sender: Box::new(message_printer),
    };
    let uci_state_machine = uci_state.uninitialized_state_machine().init();
    let uci = UCI {
        state_machine: uci_state_machine,
    };

    receive_messages(uci).unwrap();

    Ok(())
}
