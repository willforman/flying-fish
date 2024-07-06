use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use engine::HYPERBOLA_QUINTESSENCE_MOVE_GEN;

use log::LevelFilter;
use uci::UCI;

struct UCIStdoutDebugFileWriter {}

impl Write for UCIStdoutDebugFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written_str = std::str::from_utf8(buf).unwrap().to_string();
        let written_str = written_str.trim();

        if !written_str.is_empty() {
            if written_str.starts_with("info string") {
                log::info!("{}", written_str);
            } else {
                log::debug!("{}", written_str);
            }
        }
        io::stdout().lock().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().lock().flush()
    }
}

fn main() -> Result<()> {
    let log_path = dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("state")
        .join("chess")
        .join("chess.log");

    simplelog::WriteLogger::init(
        LevelFilter::Trace,
        simplelog::Config::default(),
        File::create(&log_path).context(format!("Failed to open file at: {:?}", log_path))?,
    )
    .context("Couldn't create WriteLogger")?;

    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;
    let response_writer = Arc::new(Mutex::new(UCIStdoutDebugFileWriter {}));

    let mut uci = UCI::new(move_gen, Arc::clone(&response_writer));

    for line in io::stdin().lock().lines().map(|r| r.unwrap()) {
        log::trace!("{}", line);
        let cmd_res = uci.handle_command(&line);

        if let Err(err) = cmd_res {
            println!("{}", err);
        }
    }
    Ok(())
}
