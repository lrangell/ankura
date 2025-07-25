use crate::compiler::Compiler;
use crate::daemon::Daemon;
use crate::error::{KarabinerPklError, Result};
use crate::import;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Parser)]
#[command(name = "karabiner-pkl")]
#[command(author, version, about = "Karabiner configuration using Apple Pkl", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true, default_value = "~/.config/karabiner.pkl")]
    pub config: String,

    #[arg(short, long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Start {
        #[arg(short, long)]
        foreground: bool,
    },

    Stop,

    Compile {
        #[arg(
            short,
            long,
            help = "Override the profile name (default: uses config value or 'pkl')"
        )]
        profile_name: Option<String>,

        #[arg(
            short,
            long,
            help = "Output file path (default: ~/.config/karabiner/karabiner.json)"
        )]
        output: Option<String>,
    },

    Check,

    Logs {
        #[arg(short, long, default_value = "50")]
        lines: usize,

        #[arg(short, long)]
        follow: bool,
    },

    Status,

    Init {
        #[arg(short, long)]
        force: bool,
    },

    Add {
        #[arg(help = "Path to a local .pkl file or URL to import")]
        source: String,

        #[arg(
            short,
            long,
            help = "Name for the imported file (defaults to source filename)"
        )]
        name: Option<String>,
    },
}

// CLI command implementations

pub async fn start_daemon(config_path: PathBuf, foreground: bool) -> Result<()> {
    let daemon = Daemon::new(config_path)?;
    daemon.start().await?;

    if foreground {
        // Keep the process running
        tokio::signal::ctrl_c().await.unwrap();
        daemon.stop().await?;
    }

    Ok(())
}

pub async fn stop_daemon() -> Result<()> {
    info!("Stopping karabiner-pkl daemon");
    println!("Daemon stopped");
    Ok(())
}

pub async fn compile_once(
    config_path: PathBuf,
    profile_name: Option<&str>,
    output: Option<String>,
) -> Result<()> {
    let compiler = Compiler::new()?;
    let compiled_config = compiler.compile(&config_path, profile_name).await?;

    // Determine output path
    let output_path = if let Some(path) = output {
        PathBuf::from(path)
    } else {
        let home = dirs::home_dir().ok_or_else(|| KarabinerPklError::DaemonError {
            message: "Could not find home directory".to_string(),
        })?;
        home.join(".config/karabiner/karabiner.json")
    };

    // Merge with existing configuration if needed
    let final_config = if output_path.exists() {
        merge_configurations(&output_path, compiled_config)?
    } else {
        // No existing config, just use the compiled one
        compiled_config
    };

    // Write the configuration
    write_karabiner_config(&output_path, &final_config)?;

    info!(
        "Successfully wrote configuration to {}",
        output_path.display()
    );
    Ok(())
}

pub async fn check_config(config_path: PathBuf) -> Result<()> {
    println!("Checking configuration: {}", config_path.display());

    let compiler = Compiler::new()?;
    match compiler.compile(&config_path, None).await {
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

pub fn show_logs(log_file: PathBuf, lines: usize, follow: bool) -> Result<()> {
    use std::process::Command;

    if follow {
        Command::new("tail")
            .args(["-f", "-n", &lines.to_string()])
            .arg(&log_file)
            .status()
            .map_err(|e| KarabinerPklError::DaemonError {
                message: format!("Failed to tail logs: {e}"),
            })?;
    } else {
        Command::new("tail")
            .args(["-n", &lines.to_string()])
            .arg(&log_file)
            .status()
            .map_err(|e| KarabinerPklError::DaemonError {
                message: format!("Failed to show logs: {e}"),
            })?;
    }
    Ok(())
}

pub async fn show_status() -> Result<()> {
    println!("karabiner-pkl status:");
    println!("  Daemon: stopped");
    println!("  Config: ~/.config/karabiner.pkl");
    Ok(())
}

pub async fn init_config(config_path: PathBuf, force: bool) -> Result<()> {
    if config_path.exists() && !force {
        println!("Configuration already exists at {}", config_path.display());
        println!("Use --force to overwrite");
        return Ok(());
    }

    // Ensure pkl files are materialized by creating a compiler instance
    let _compiler = crate::compiler::Compiler::new()?;

    // Get the actual data directory path
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| crate::error::KarabinerPklError::DaemonError {
            message: "Could not find local data directory".to_string(),
        })?
        .join("karabiner-pkl");

    println!("✅ Pkl library files ready at {}", data_dir.display());

    let example_config = r#"// Karabiner configuration using Pkl
// Import the karabiner library from the embedded module path
import "modulepath:/karabiner.pkl"
import "modulepath:/helpers.pkl"

// Create a simple configuration
config = new karabiner.SimpleConfig {
  // Example: Map Caps Lock to Escape when tapped, Control when held
  simple_modifications = List()
  
  complex_modifications = new karabiner.ComplexModifications {
    rules = List(
      new karabiner.Rule {
        description = "Caps Lock to Control/Escape"
        manipulators = List(
          helpers.dualFunction(
            new karabiner.KeyCode { key_code = "caps_lock" }, 
            new karabiner.ToEvent.KeyEvent { key_code = "left_control" }, 
            new karabiner.ToEvent.KeyEvent { key_code = "escape" }
          )
        )
      }
    )
  }
}.toConfig()
"#;

    // Create PklProject file content with the actual data directory path
    let pkl_project = format!(
        r#"amends "pkl:Project"

evaluatorSettings {{
  modulePath {{
    "{}"
  }}
}}
"#,
        data_dir.display()
    );

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| KarabinerPklError::ConfigWriteError {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    std::fs::write(&config_path, example_config).map_err(|e| {
        KarabinerPklError::ConfigWriteError {
            path: config_path.clone(),
            source: e,
        }
    })?;

    // Write PklProject file in the same directory
    if let Some(parent) = config_path.parent() {
        let pkl_project_path = parent.join("PklProject");
        std::fs::write(&pkl_project_path, pkl_project).map_err(|e| {
            KarabinerPklError::ConfigWriteError {
                path: pkl_project_path.clone(),
                source: e,
            }
        })?;
        println!("Created PklProject file at {}", pkl_project_path.display());
    }

    println!("Created example configuration at {}", config_path.display());
    println!("Edit this file and run 'karabiner-pkl compile' to apply changes");
    Ok(())
}

pub async fn add_import(source: String, name: Option<String>) -> Result<()> {
    let importer = import::Importer::new()?;
    let import_name = name.clone();
    importer.import(&source, name).await?;

    println!("✅ Successfully imported {source}");
    println!("You can now use it in your configuration with:");
    if let Some(ref name) = import_name {
        println!("  import \"modulepath:/{name}\"");
    } else {
        let filename = source.split('/').next_back().unwrap_or("imported.pkl");
        println!("  import \"modulepath:/{filename}\"");
    }

    Ok(())
}

// Helper functions for configuration management

pub fn merge_configurations(existing_path: &Path, new_config: Value) -> Result<Value> {
    // Read existing configuration
    let existing_content =
        std::fs::read_to_string(existing_path).map_err(|e| KarabinerPklError::ConfigReadError {
            path: existing_path.to_path_buf(),
            source: e,
        })?;

    let mut existing_config: Value = serde_json::from_str(&existing_content)
        .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

    // Extract the profile from the new config
    let new_profile = new_config["profiles"][0].clone();
    let target_profile_name = new_profile["name"].as_str().unwrap_or("pkl");

    // Ensure we have a profiles array
    if !existing_config
        .get("profiles")
        .map(|p| p.is_array())
        .unwrap_or(false)
    {
        existing_config["profiles"] = serde_json::json!([]);
    }

    // Update or add the profile
    let profiles = existing_config["profiles"].as_array_mut().unwrap();

    // Find existing profile with the same name
    let existing_profile_index = profiles
        .iter()
        .position(|p| p["name"].as_str() == Some(target_profile_name));

    if let Some(index) = existing_profile_index {
        // Update existing profile
        profiles[index] = new_profile;
    } else {
        // Add new profile - should not be selected by default
        let mut profile_to_add = new_profile;
        profile_to_add["selected"] = serde_json::json!(false);
        profiles.push(profile_to_add);
    }

    // Set title if not present
    if existing_config.get("title").is_none() {
        existing_config["title"] = new_config
            .get("title")
            .cloned()
            .unwrap_or_else(|| serde_json::json!("Karabiner-Pkl Configuration"));
    }

    Ok(existing_config)
}

pub fn write_karabiner_config(path: &Path, config: &Value) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| KarabinerPklError::KarabinerWriteError {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let pretty_json = serde_json::to_string_pretty(config)
        .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

    std::fs::write(path, pretty_json).map_err(|e| KarabinerPklError::KarabinerWriteError {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}
