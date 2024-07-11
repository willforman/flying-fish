use std::{path::PathBuf, sync::OnceLock};

pub static LOGS_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
