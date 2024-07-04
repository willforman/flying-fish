use std::{
    io::{self, BufRead, Write},
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

    for line in io::stdin().lock().lines().map(|r| r.unwrap()) {
        let cmd_res = uci.handle_command(&line);

        if let Err(err) = cmd_res {
            println!("{}", err);
        }
    }
    Ok(())
}
