use std::{
    fs::{self, File},
    io::{Stdout, Write},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};

use crate::LOGS_DIRECTORY;

struct EngineWriter<W>
where
    W: Write,
{
    main_writer: W,
    maybe_debug_writer: Option<fs::File>,
    start_time: DateTime<Local>,
}

impl<W> EngineWriter<W>
where
    W: Write,
{
    pub(crate) fn new(main_writer: W, start_time: DateTime<Local>) -> Self {
        Self {
            main_writer,
            maybe_debug_writer: None,
            start_time,
        }
    }

    pub(crate) fn set_debug(&mut self, enabled: bool) -> Result<()> {
        match (&self.maybe_debug_writer, enabled) {
            (None, true) => {
                let logs_directory = LOGS_DIRECTORY
                    .get()
                    .expect("LOGS_DIRECTORY should be set")
                    .clone();

                let mut debug_logs_path = logs_directory.clone();
                debug_logs_path.push("in_out");
                debug_logs_path.push(format!("in_out-{}.txt", self.start_time));
                self.maybe_debug_writer = Some(
                    File::create(&debug_logs_path)
                        .context(format!("Couldn't open: {:?}", &debug_logs_path))?,
                );
            }
            (Some(_), false) => self.maybe_debug_writer = None,
            _ => {} // Pass
        }
        Ok(())
    }
}

impl<W> Write for EngineWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.main_writer.write(buf)?;

        if let Some(debug_writer) = self.maybe_debug_writer.as_mut() {
            debug_writer.write_all(&buf[..bytes_written])?;
        }
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.main_writer.flush()?;
        if let Some(debug_writer) = self.maybe_debug_writer.as_mut() {
            debug_writer.flush()?;
        }
        Ok(())
    }
}
