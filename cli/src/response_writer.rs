use core::str;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use chrono::Local;

#[derive(Debug)]
pub(crate) struct ResponseWriter<W1, W2>
where
    W1: Write,
    W2: Write,
{
    main_writer: W1,
    maybe_debug_writer: Arc<Mutex<Option<W2>>>,
}

impl<W1, W2> ResponseWriter<W1, W2>
where
    W1: Write,
    W2: Write,
{
    pub(crate) fn new(main_writer: W1, maybe_debug_writer: Arc<Mutex<Option<W2>>>) -> Self {
        Self {
            main_writer,
            maybe_debug_writer,
        }
    }
}

impl<W1, W2> Write for ResponseWriter<W1, W2>
where
    W1: Write,
    W2: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.main_writer.write(buf)?;

        if let Some(debug_writer) = self.maybe_debug_writer.lock().unwrap().as_mut() {
            let mut debug_str = Local::now().format("%I:%M:%m%.3f").to_string();
            debug_str.push_str(": ");
            let buf_str = str::from_utf8(&buf[..bytes_written]).expect("Couldn't parse buf");
            debug_str.push_str(buf_str);

            debug_writer.write_all(debug_str.as_bytes())?;
        }
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.main_writer.flush()?;
        if let Some(debug_writer) = self.maybe_debug_writer.lock().unwrap().as_mut() {
            debug_writer.flush()?;
        }
        Ok(())
    }
}
