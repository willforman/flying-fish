use std::{
    io::{self, Write},
    str::FromStr,
    sync::{Arc, Mutex},
};

use engine::GenerateMoves;
use statig::prelude::{InitializedStateMachine, IntoStateMachineExt};

use crate::{
    messages::{UCICommand, UCICommandParseError},
    state::UCIState,
};

#[allow(clippy::upper_case_acronyms)]
pub struct UCI<G>
where
    G: GenerateMoves + Copy + Send + Sync + 'static,
{
    state_machine: InitializedStateMachine<UCIState<G>>,
}

impl<G> UCI<G>
where
    G: GenerateMoves + Copy + Send + Sync + 'static,
{
    pub fn new(move_gen: G) -> Self {
        let uci_state = UCIState::new(move_gen);
        let uci_state_machine = uci_state.uninitialized_state_machine().init();
        Self {
            state_machine: uci_state_machine,
        }
    }

    pub fn handle_command(&mut self, command: &str) -> Result<(), UCICommandParseError> {
        let command = UCICommand::from_str(command)?;
        self.state_machine.handle(&command);
        Ok(())
    }
}
