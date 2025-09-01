use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum KarabinerPklError {
    #[error("Pkl CLI not found in PATH")]
    #[diagnostic(
        code(ankura::pkl_not_found),
        help("Install Pkl CLI from https://pkl-lang.org or via Homebrew: brew install pkl")
    )]
    PklNotFound,

    #[error("Failed to read configuration file")]
    #[diagnostic(code(ankura::read_error))]
    ConfigReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Pkl compilation failed")]
    #[diagnostic(code(ankura::pkl_compile_error))]
    PklCompileError {
        #[help]
        help: String,
        #[source_code]
        source_code: String,
        #[label("error occurred here")]
        span: Option<miette::SourceSpan>,
    },

    #[error("Invalid JSON output from Pkl")]
    #[diagnostic(
        code(ankura::json_parse_error),
        help("This is likely a bug in the Pkl configuration or ankura")
    )]
    JsonParseError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to write Karabiner configuration")]
    #[diagnostic(code(ankura::write_error))]
    KarabinerWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Configuration validation failed")]
    #[diagnostic(code(ankura::validation_error))]
    ValidationError {
        #[help]
        message: String,
    },

    #[error("File watching error")]
    #[diagnostic(code(ankura::watch_error))]
    WatchError {
        #[source]
        source: notify::Error,
    },

    #[error("Daemon error")]
    #[diagnostic(code(ankura::daemon_error))]
    DaemonError { message: String },

    #[error("Failed to write configuration file")]
    #[diagnostic(code(ankura::config_write_error))]
    ConfigWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, KarabinerPklError>;
