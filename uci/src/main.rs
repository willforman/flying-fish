use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use engine::HYPERBOLA_QUINTESSENCE_MOVE_GEN;
use simple_logger::SimpleLogger;

use uci::UCI;

fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;
    let response_writer = Arc::new(Mutex::new(io::stdout()));

    let mut uci = UCI::new(move_gen, Arc::clone(&response_writer));

    let mut command_str = String::new();
    loop {
        io::stdin().read_line(&mut command_str)?;

        let cmd_res = uci.handle_command(&command_str);

        if let Err(err) = cmd_res {
            println!("HERE!");
            response_writer
                .lock()
                .unwrap()
                .write_all(err.to_string().as_bytes())?;
        }
    }
}
