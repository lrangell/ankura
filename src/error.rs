use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum KarabinerPklError {
    #[error("Pkl CLI not found in PATH")]
    #[diagnostic(
        code(karabiner_pkl::pkl_not_found),
        help("Install Pkl CLI from https://pkl-lang.org or via Homebrew: brew install pkl")
    )]
    PklNotFound,

    #[error("Failed to read configuration file")]
    #[diagnostic(code(karabiner_pkl::read_error))]
    ConfigReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Pkl compilation failed")]
    #[diagnostic(code(karabiner_pkl::pkl_compile_error))]
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
        code(karabiner_pkl::json_parse_error),
        help("This is likely a bug in the Pkl configuration or karabiner-pkl")
    )]
    JsonParseError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to write Karabiner configuration")]
    #[diagnostic(code(karabiner_pkl::write_error))]
    KarabinerWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Configuration validation failed")]
    #[diagnostic(code(karabiner_pkl::validation_error))]
    ValidationError { message: String },

    #[error("File watching error")]
    #[diagnostic(code(karabiner_pkl::watch_error))]
    WatchError {
        #[source]
        source: notify::Error,
    },

    #[error("Daemon error")]
    #[diagnostic(code(karabiner_pkl::daemon_error))]
    DaemonError { message: String },
}

pub type Result<T> = std::result::Result<T, KarabinerPklError>;