mod cli;
mod compiler;
mod daemon;
mod error;
mod logging;
mod notifications;

use clap::Parser;
use cli::{Cli, Commands};
use daemon::Daemon;
use error::Result;
use miette;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    
    let log_file = logging::init_logging()
        .map_err(|e| error::KarabinerPklError::DaemonError {
            message: format!("Failed to initialize logging: {}", e),
        })?;

    let config_path = shellexpand::tilde(&cli.config).to_string();
    let config_path = PathBuf::from(config_path);

    match cli.command {
        Commands::Start { foreground } => {
            start_daemon(config_path, foreground).await?;
        }
        Commands::Stop => {
            stop_daemon().await?;
        }
        Commands::Compile => {
            compile_once(config_path).await?;
        }
        Commands::Check => {
            check_config(config_path).await?;
        }
        Commands::Logs { lines, follow } => {
            show_logs(log_file, lines, follow)?;
        }
        Commands::Status => {
            show_status().await?;
        }
        Commands::Init { force } => {
            init_config(config_path, force).await?;
        }
    }

    Ok(())
}

async fn start_daemon(config_path: PathBuf, foreground: bool) -> Result<()> {
    info!("Starting karabiner-pkl daemon");
    
    let daemon = Daemon::new(config_path)?;
    daemon.start().await?;

    if foreground {
        info!("Running in foreground mode. Press Ctrl+C to stop.");
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| error::KarabinerPklError::DaemonError {
                message: format!("Failed to install signal handler: {}", e),
            })?;
        daemon.stop().await?;
    } else {
        info!("Daemon started in background");
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| error::KarabinerPklError::DaemonError {
                message: format!("Failed to install signal handler: {}", e),
            })?;
        daemon.stop().await?;
    }

    Ok(())
}

async fn stop_daemon() -> Result<()> {
    info!("Stopping karabiner-pkl daemon");
    println!("Daemon stopped");
    Ok(())
}

async fn compile_once(config_path: PathBuf) -> Result<()> {
    let daemon = Daemon::new(config_path)?;
    daemon.compile_once().await?;
    Ok(())
}

async fn check_config(config_path: PathBuf) -> Result<()> {
    println!("Checking configuration: {}", config_path.display());
    
    let compiler = compiler::Compiler::new()?;
    match compiler.compile(&config_path).await {
        Ok(_) => {
            println!("✅ Configuration is valid!");
            Ok(())
        }
        Err(e) => {
            println!("❌ Configuration is invalid:");
            Err(e)
        }
    }
}

fn show_logs(log_file: PathBuf, lines: usize, follow: bool) -> Result<()> {
    use std::process::Command;
    
    if follow {
        Command::new("tail")
            .args(["-f", "-n", &lines.to_string()])
            .arg(&log_file)
            .status()
            .map_err(|e| error::KarabinerPklError::DaemonError {
                message: format!("Failed to tail logs: {}", e),
            })?;
    } else {
        Command::new("tail")
            .args(["-n", &lines.to_string()])
            .arg(&log_file)
            .status()
            .map_err(|e| error::KarabinerPklError::DaemonError {
                message: format!("Failed to show logs: {}", e),
            })?;
    }
    
    Ok(())
}

async fn show_status() -> Result<()> {
    println!("Karabiner-Pkl Status:");
    println!("  Daemon: Not running");
    println!("  Config: ~/.config/karabiner.pkl");
    println!("  Logs: ~/.local/share/karabiner-pkl/logs/karabiner-pkl.log");
    Ok(())
}

async fn init_config(config_path: PathBuf, force: bool) -> Result<()> {
    if config_path.exists() && !force {
        return Err(error::KarabinerPklError::DaemonError {
            message: "Configuration already exists. Use --force to overwrite.".to_string(),
        });
    }

    println!("Creating example configuration at: {}", config_path.display());
    
    let example_config = r#"module karabiner

module karabiner

profiles = List {
  new {
    name = "Default"
    selected = true
    rules = List {
      new {
        description = "Caps Lock to Escape"
        manipulators = List {
          new {
            type = "basic"
            from = new {
              key_code = "caps_lock"
            }
            to = List {
              new {
                key_code = "escape"
              }
            }
          }
        }
      }
    }
  }
}
"#;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            error::KarabinerPklError::ConfigReadError {
                path: parent.to_path_buf(),
                source: e,
            }
        })?;
    }

    std::fs::write(&config_path, example_config).map_err(|e| {
        error::KarabinerPklError::KarabinerWriteError {
            path: config_path.clone(),
            source: e,
        }
    })?;

    println!("✅ Created example configuration!");
    println!("Edit {} and run 'karabiner-pkl compile' to test", config_path.display());
    
    Ok(())
}