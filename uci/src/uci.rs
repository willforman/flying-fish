use std::{str::FromStr, sync::Arc};

use engine::GenerateMoves;
use statig::prelude::{InitializedStateMachine, IntoStateMachineExt};

use crate::{
    messages::{UCICommand, UCICommandParseError},
    state::UCIState,
    WriteUCIResponse,
};

#[allow(clippy::upper_case_acronyms)]
pub struct UCI<T, U>
where
    T: GenerateMoves + Copy,
    U: WriteUCIResponse,
{
    state_machine: InitializedStateMachine<UCIState<T, U>>,
}

impl<T, U> UCI<T, U>
where
    T: GenerateMoves + Copy,
    U: WriteUCIResponse,
{
    pub fn new(move_gen: T, response_writer: Arc<U>) -> Self {
        let uci_state = UCIState::new(move_gen, response_writer);
        let uci_state_machine = uci_state.uninitialized_state_machine().init();
        Self {
            state_machine: uci_state_machine,
        }
    }

    pub fn handle_command(&mut self, command: String) -> Result<(), UCICommandParseError> {
        let command = UCICommand::from_str(&command)?;
        self.state_machine.handle(&command);
        Ok(())
    }
}
