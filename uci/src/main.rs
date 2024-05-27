use std::sync::Arc;

use anyhow::Result;
use engine::HYPERBOLA_QUINTESSENCE_MOVE_GEN;
use simple_logger::SimpleLogger;

use uci::{ReadUCICommand, UCICommandStdinReader, UCIResponseStdoutWriter, UCI};

fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let command_reader = UCICommandStdinReader;
    let response_writer = Arc::new(UCIResponseStdoutWriter);
    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

    let mut uci = UCI::new(move_gen, Arc::clone(&response_writer));
    loop {
        let command = command_reader.read_uci_command().unwrap();
        uci.handle_command(&command).unwrap();
    }
}
