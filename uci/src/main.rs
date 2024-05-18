use anyhow::Result;
use engine::move_gen::HYPERBOLA_QUINTESSENCE_MOVE_GEN;
use simple_logger::SimpleLogger;
use statig::prelude::{InitializedStateMachine, IntoStateMachineExt};

use uci::{ReadUCICommand, UCICommandStdinReader, UCIResponseStdoutWriter, UCI};

fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let uci_reader = UCICommandStdinReader;
    let uci_writer = UCIResponseStdoutWriter;
    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

    let mut uci = UCI::new(move_gen, uci_writer);
    loop {
        let command = uci_reader.read_uci_command().unwrap();
        uci.handle_command(command).unwrap();
    }

    Ok(())
}
