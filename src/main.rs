use ankura::cli::{self, Cli, Commands};
use ankura::error::Result;
use ankura::logging;
use clap::Parser;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let _ = logging::init_logging(cli.debug_log);

    let config_path = expand_tilde(&cli.config);

    match cli.command {
        Commands::Start { daemon_mode } => {
            cli::start_daemon(config_path, daemon_mode, cli.debug_log).await
        }
        Commands::Stop => cli::stop_daemon().await,
        Commands::Compile {
            profile_name,
            output,
        } => cli::compile_once(config_path, profile_name.as_deref(), output).await,
        Commands::Check => cli::check_config(config_path).await,
        Commands::Logs { lines, follow } => {
            let log_file = get_log_file()?;
            cli::show_logs(log_file, lines, follow)
        }
        Commands::Status => cli::show_status().await,
        Commands::Init { force } => cli::init_config(config_path, force).await,
        Commands::Add { source, name } => cli::add_import(source, name).await,
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn get_log_file() -> Result<PathBuf> {
    let log_file = logging::log_file_path();
    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ankura::error::KarabinerPklError::DaemonError {
                message: format!("Failed to create log directory: {e}"),
            }
        })?;
    }
    Ok(log_file)
}
