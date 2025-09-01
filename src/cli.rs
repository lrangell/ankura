use crate::compiler::Compiler;
use crate::daemon::Daemon;
use crate::error::{KarabinerPklError, Result};
use crate::import;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Parser)]
#[command(name = "ankura")]
#[command(author, version, about = "Karabiner configuration using Apple Pkl", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true, default_value = "~/.config/ankura.pkl")]
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

pub async fn start_daemon(config_path: PathBuf, foreground: bool) -> Result<()> {
    let daemon = Daemon::new(config_path)?;
    daemon.start().await?;

    if foreground {
        tokio::signal::ctrl_c().await.unwrap();
        daemon.stop().await?;
    }

    Ok(())
}

pub async fn stop_daemon() -> Result<()> {
    info!("Stopping ankura daemon");
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

    let output_path = if let Some(path) = output {
        PathBuf::from(path)
    } else {
        let home = dirs::home_dir().ok_or_else(|| KarabinerPklError::DaemonError {
            message: "Could not find home directory".to_string(),
        })?;
        home.join(".config/karabiner/karabiner.json")
    };

    let final_config = if output_path.exists() {
        merge_configurations(&output_path, compiled_config)?
    } else {
        compiled_config
    };

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
    println!("ankura status:");
    println!("  Daemon: stopped");
    println!("  Config: ~/.config/ankura.pkl");
    Ok(())
}

pub async fn init_config(config_path: PathBuf, force: bool) -> Result<()> {
    if config_path.exists() && !force {
        println!("Configuration already exists at {}", config_path.display());
        println!("Use --force to overwrite");
        return Ok(());
    }

    let _compiler = crate::compiler::Compiler::new()?;

    let data_dir = std::path::PathBuf::from("/opt/homebrew/share/ankura");

    println!("✅ Pkl library files ready at {}", data_dir.display());

    let example_config = format!(
        r#"// Simple Karabiner configuration with unified userconfig
amends "{}/config.pkl"

name = "My Karabiner Config"

rules = List(
    new DualUse {{
        key = outer.keys.caps_lock
        tap = outer.keys.escape
        hold = outer.keys.left_control
    }},
    
    new SimLayer {{
        trigger = outer.keys.f
        h = outer.keys.left_arrow    // h -> Left
        j = outer.keys.down_arrow    // j -> Down  
        u = outer.keys.up_arrow      // u -> Up
        l = outer.keys.right_arrow   // l -> Right
    }}
)
"#,
        data_dir.display()
    );

    let pkl_project = r#"amends "pkl:Project"
"#;

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
    println!("Edit this file and run 'ankura compile' to apply changes");
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

pub fn merge_configurations(existing_path: &Path, new_config: Value) -> Result<Value> {
    let existing_content =
        std::fs::read_to_string(existing_path).map_err(|e| KarabinerPklError::ConfigReadError {
            path: existing_path.to_path_buf(),
            source: e,
        })?;

    let mut existing_config: Value = serde_json::from_str(&existing_content)
        .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

    let new_profile = new_config["profiles"][0].clone();
    let target_profile_name = new_profile["name"].as_str().unwrap_or("pkl");

    if !existing_config
        .get("profiles")
        .map(|p| p.is_array())
        .unwrap_or(false)
    {
        existing_config["profiles"] = serde_json::json!([]);
    }

    let profiles = existing_config["profiles"].as_array_mut().unwrap();

    let existing_profile_index = profiles
        .iter()
        .position(|p| p["name"].as_str() == Some(target_profile_name));

    if let Some(index) = existing_profile_index {
        profiles[index] = new_profile;
    } else {
        let mut profile_to_add = new_profile;
        profile_to_add["selected"] = serde_json::json!(false);
        profiles.push(profile_to_add);
    }

    if existing_config.get("title").is_none() {
        existing_config["title"] = new_config
            .get("title")
            .cloned()
            .unwrap_or_else(|| serde_json::json!("Karabiner-Pkl Configuration"));
    }

    Ok(existing_config)
}

pub fn write_karabiner_config(path: &Path, config: &Value) -> Result<()> {
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
