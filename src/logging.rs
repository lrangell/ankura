use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub const LOG_DIR: &str = "/opt/homebrew/var/log/ankura";
pub const LOG_FILE_NAME: &str = "ankura.log";

pub fn log_dir() -> PathBuf {
    PathBuf::from(LOG_DIR)
}

pub fn log_file_path() -> PathBuf {
    log_dir().join(LOG_FILE_NAME)
}

pub fn init_logging(
    debug_log: bool,
) -> std::result::Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let log_dir = log_dir();

    fs::create_dir_all(&log_dir)?;

    let log_file = log_file_path();
    let file_appender = tracing_subscriber::fmt::layer()
        .with_writer(std::sync::Mutex::new(
            OpenOptions::new()
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
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            let default_level = if debug_log { "debug" } else { "info" };
            let directives = format!(
                "ankura={default_level},notify=warn,notify_debouncer_mini=warn"
            );
            EnvFilter::new(directives)
        }))
        .with(console_layer)
        .with(file_appender)
        .init();

    Ok(log_file)
}
