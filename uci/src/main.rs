use std::io::{self, Write};

use anyhow::Result;
use engine::HYPERBOLA_QUINTESSENCE_MOVE_GEN;
use simple_logger::SimpleLogger;

use uci::{ReadUCICommand, UCICommandStdinReader, UCI};

#[derive(Clone, Copy, Debug)]
pub struct StdoutWriter;

impl Write for StdoutWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let command_reader = UCICommandStdinReader;
    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

    let mut uci = UCI::new(move_gen, StdoutWriter);
    loop {
        let command = command_reader.read_uci_command().unwrap();
        uci.handle_command(&command).unwrap();
    }
}
