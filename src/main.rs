use clap::Parser;
use karabiner_pkl::cli::{self, Cli, Commands};
use karabiner_pkl::error::Result;
use karabiner_pkl::logging;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = logging::init_logging();

    let cli = Cli::parse();

    // Expand tilde in config path
    let config_path = expand_tilde(&cli.config);

    match cli.command {
        Commands::Start { foreground } => cli::start_daemon(config_path, foreground).await,
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
    let data_dir = dirs::data_local_dir().ok_or_else(|| {
        karabiner_pkl::error::KarabinerPklError::DaemonError {
            message: "Could not find local data directory".to_string(),
        }
    })?;
    Ok(data_dir.join("karabiner-pkl").join("daemon.log"))
}
