use std::io::{self, BufRead};

use anyhow::Result;
use log;
use simple_logger::SimpleLogger;

struct UciInterface {}

impl UciInterface {
    pub fn accept_command(&self, command: String) -> Result<()> {
        log::debug!("{}", command);
        Ok(())
    }
}

fn main() -> Result<()> {
    SimpleLogger::new().env().init().unwrap();
    let stdin = io::stdin();
    let uci = UciInterface {};
    loop {
        for line in stdin.lock().lines() {
            uci.accept_command(line.unwrap())?;
        }
    }
}
