use crate::compiler::Compiler;
use crate::daemon::Daemon;
use crate::error::{KarabinerPklError, Result};
use crate::import;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::convert::TryInto;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(unix)]
use libc::{self, c_int, pid_t, EPERM, ESRCH, SIGTERM};

#[cfg(unix)]
type ProcessId = pid_t;

#[cfg(not(unix))]
type ProcessId = u32;

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
        #[arg(long, hide = true)]
        daemon_mode: bool,
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

pub async fn start_daemon(config_path: PathBuf, daemon_mode: bool) -> Result<()> {
    if daemon_mode {
        run_daemon(config_path).await
    } else {
        spawn_daemon(config_path).await
    }
}

async fn spawn_daemon(config_path: PathBuf) -> Result<()> {
    let pid_path = daemon_pid_file()?;

    if let Some(existing_pid) = read_pid(&pid_path)? {
        if process_is_running(existing_pid) {
            info!("Existing ankura daemon detected (pid {existing_pid}), attempting to stop it");
            terminate_process(existing_pid).await?;
        } else {
            warn!("Removing stale ankura daemon pid file referencing pid {existing_pid}");
        }

        if let Err(e) = fs::remove_file(&pid_path) {
            if e.kind() != io::ErrorKind::NotFound {
                warn!("Failed to remove old pid file {}: {e}", pid_path.display());
            }
        }
    }

    let exe_path = std::env::current_exe().map_err(|e| KarabinerPklError::DaemonError {
        message: format!("Failed to resolve ankura executable: {e}"),
    })?;

    let config_arg = config_path.to_str().map(|s| s.to_string()).ok_or_else(|| {
        KarabinerPklError::DaemonError {
            message: format!(
                "Config path contains invalid UTF-8: {}",
                config_path.display()
            ),
        }
    })?;

    let mut command = Command::new(exe_path);
    command
        .arg("--config")
        .arg(&config_arg)
        .arg("start")
        .arg("--daemon-mode")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = command
        .spawn()
        .map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to spawn ankura daemon: {e}"),
        })?;

    info!("Started ankura daemon with pid {}", child.id());

    // Give the daemon a brief moment to write the PID file so that status commands can read it.
    for _ in 0..20 {
        if pid_path.exists() {
            info!("Daemon pid file created at {}", pid_path.display());
            return Ok(());
        }
        sleep(Duration::from_millis(50)).await;
    }

    warn!(
        "Daemon pid file was not created within the expected time at {}",
        pid_path.display()
    );

    Ok(())
}

struct PidFileGuard {
    path: PathBuf,
}

impl PidFileGuard {
    fn claim(path: &Path) -> Result<Self> {
        fs::write(path, format!("{}", std::process::id())).map_err(|e| {
            KarabinerPklError::DaemonError {
                message: format!("Failed to write daemon pid file {}: {e}", path.display()),
            }
        })?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            if e.kind() == io::ErrorKind::NotFound {
                return;
            }
            warn!(
                "Failed to remove daemon pid file {}: {e}",
                self.path.display()
            );
        }
    }
}

fn daemon_pid_file() -> Result<PathBuf> {
    let runtime_dir = homebrew_var_dir()?.join("run");
    fs::create_dir_all(&runtime_dir).map_err(|e| KarabinerPklError::DaemonError {
        message: format!(
            "Failed to create runtime directory {}: {e}",
            runtime_dir.display()
        ),
    })?;

    Ok(runtime_dir.join("ankura.pid"))
}

fn homebrew_var_dir() -> Result<PathBuf> {
    if let Some(prefix) = std::env::var_os("HOMEBREW_PREFIX") {
        let path = PathBuf::from(prefix).join("var");
        return Ok(path);
    }

    for candidate in ["/opt/homebrew", "/usr/local"] {
        let path = PathBuf::from(candidate).join("var");
        if path.exists() {
            return Ok(path);
        }
    }

    Ok(PathBuf::from("/opt/homebrew/var"))
}

fn read_pid(path: &Path) -> Result<Option<ProcessId>> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|e| KarabinerPklError::DaemonError {
        message: format!("Failed to read daemon pid file {}: {e}", path.display()),
    })?;

    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let raw_value: i64 = trimmed
        .parse()
        .map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Invalid pid value '{trimmed}' in {}: {e}", path.display()),
        })?;

    let pid = raw_value
        .try_into()
        .map_err(|_| KarabinerPklError::DaemonError {
            message: format!("Pid value {raw_value} does not fit in platform pid type"),
        })?;

    Ok(Some(pid))
}

#[cfg(unix)]
async fn terminate_process(pid: ProcessId) -> Result<()> {
    send_signal(pid, SIGTERM)?;

    for _ in 0..50 {
        if !process_is_running(pid) {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    Err(KarabinerPklError::DaemonError {
        message: format!("Timed out waiting for daemon (pid {pid}) to exit"),
    })
}

#[cfg(not(unix))]
async fn terminate_process(_pid: ProcessId) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn process_is_running(pid: ProcessId) -> bool {
    let result = unsafe { libc::kill(pid, 0) };
    if result == 0 {
        true
    } else {
        match io::Error::last_os_error().raw_os_error() {
            Some(EPERM) => true,
            Some(ESRCH) => false,
            _ => false,
        }
    }
}

#[cfg(not(unix))]
fn process_is_running(_pid: ProcessId) -> bool {
    false
}

#[cfg(unix)]
fn send_signal(pid: ProcessId, signal: c_int) -> Result<()> {
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 {
        Ok(())
    } else {
        Err(KarabinerPklError::DaemonError {
            message: format!(
                "Failed to send signal {signal} to pid {pid}: {}",
                io::Error::last_os_error()
            ),
        })
    }
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() -> Result<()> {
    let mut sigterm =
        signal(SignalKind::terminate()).map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to listen for SIGTERM: {e}"),
        })?;

    let mut sigint =
        signal(SignalKind::interrupt()).map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to listen for SIGINT: {e}"),
        })?;

    tokio::select! {
        res = tokio::signal::ctrl_c() => {
            res.map_err(|e| KarabinerPklError::DaemonError {
                message: format!("Failed to listen for Ctrl+C: {e}"),
            })?;
        }
        _ = sigterm.recv() => {}
        _ = sigint.recv() => {}
    }

    Ok(())
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() -> Result<()> {
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to listen for shutdown signal: {e}"),
        })?;
    Ok(())
}

async fn run_daemon(config_path: PathBuf) -> Result<()> {
    let pid_path = daemon_pid_file()?;
    let _pid_guard = PidFileGuard::claim(&pid_path)?;

    let daemon = Daemon::new(config_path)?;
    daemon.start().await?;

    info!("Ankura daemon is running (pid {})", std::process::id());

    wait_for_shutdown_signal().await?;

    info!("Shutdown signal received, stopping ankura daemon");
    daemon.stop().await?;

    Ok(())
}

pub async fn stop_daemon() -> Result<()> {
    let pid_path = daemon_pid_file()?;

    match read_pid(&pid_path)? {
        Some(pid) if process_is_running(pid) => {
            info!("Stopping ankura daemon (pid {pid})");
            terminate_process(pid).await?;
            if let Err(e) = fs::remove_file(&pid_path) {
                warn!("Failed to remove pid file {}: {e}", pid_path.display());
            }
            println!("Daemon stopped");
        }
        Some(pid) => {
            warn!("Found stale ankura pid file pointing to pid {pid}, removing it");
            if let Err(e) = fs::remove_file(&pid_path) {
                warn!(
                    "Failed to remove stale pid file {}: {e}",
                    pid_path.display()
                );
            }
            println!("Daemon is not running");
        }
        None => {
            println!("Daemon is not running");
        }
    }

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
    let data_dir = crate::compiler::Compiler::lib_dir();

    std::fs::create_dir_all(&data_dir).map_err(|e| KarabinerPklError::DaemonError {
        message: format!("Failed to create lib directory {}: {e}", data_dir.display()),
    })?;

    let _extracted_dir = crate::compiler::Compiler::materialize_pkl_lib()?;

    println!("✅ Pkl library files ready at {}", data_dir.display());

    if config_path.exists() && !force {
        println!("Configuration already exists at {}", config_path.display());
        println!("Use --force to overwrite");
        return Ok(());
    }

    // Read the blank config template
    let blank_config_path = PathBuf::from("pkl/blank_config.pkl");
    let example_config = std::fs::read_to_string(&blank_config_path).map_err(|e| {
        KarabinerPklError::ConfigReadError {
            path: blank_config_path.clone(),
            source: e,
        }
    })?;

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

    let mut new_profile = new_config["profiles"][0].clone();
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
        if let Some(selected) = profiles
            .get(index)
            .and_then(|profile| profile.get("selected"))
            .and_then(|value| value.as_bool())
        {
            new_profile["selected"] = serde_json::Value::Bool(selected);
        }
        profiles[index] = new_profile;
    } else {
        profiles.push(new_profile);
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
