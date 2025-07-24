use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() -> std::result::Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("karabiner-pkl/logs");

    fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("karabiner-pkl.log");
    let file_appender = tracing_subscriber::fmt::layer()
        .with_writer(std::sync::Mutex::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)?,
        ))
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    let console_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("karabiner_pkl=info,warn")),
        )
        .with(console_layer)
        .with(file_appender)
        .init();

    Ok(log_file)
}
