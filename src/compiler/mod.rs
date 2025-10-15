use crate::error::{KarabinerPklError, Result};
use regex::Regex;
use rust_embed::RustEmbed;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tracing::debug;
use which::which;

const ANKURA_LIB_DIR: &str = "/opt/homebrew/var/lib/ankura";

#[derive(RustEmbed)]
#[folder = "pkl/"]
#[include = "*.pkl"]
struct PklLib;

pub struct Compiler {
    pkl_path: PathBuf,
    embedded_lib_path: PathBuf,
}

impl Compiler {
    pub fn new() -> Result<Self> {
        let pkl_path = which("pkl").map_err(|_| KarabinerPklError::PklNotFound)?;

        let embedded_lib_path = Self::materialize_pkl_lib()?;

        Ok(Self {
            pkl_path,
            embedded_lib_path,
        })
    }

    pub async fn compile(&self, config_path: &Path, profile_name: Option<&str>) -> Result<Value> {
        debug!("Compiling {}", config_path.display());

        if !config_path.exists() {
            return Err(KarabinerPklError::ConfigReadError {
                path: config_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Configuration file not found",
                ),
            });
        }

        let home = dirs::home_dir().ok_or_else(|| KarabinerPklError::DaemonError {
            message: "Could not find home directory".to_string(),
        })?;
        let lib_dir = home.join(".config/karabiner_pkl/lib");

        let mut pkl_command = Command::new(&self.pkl_path);
        pkl_command.args(["eval", "--format=json"]);

        let mut module_paths = vec![];

        module_paths.push(self.embedded_lib_path.to_string_lossy().to_string());

        if lib_dir.exists() {
            module_paths.push(lib_dir.to_string_lossy().to_string());
        }

        pkl_command.arg("--module-path");
        pkl_command.arg(module_paths.join(":"));
        pkl_command.arg(config_path);

        let output = pkl_command
            .output()
            .map_err(|e| KarabinerPklError::DaemonError {
                message: format!("Failed to execute pkl: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{stderr}");

            let (error_msg, line_number) = Self::parse_pkl_error(&stderr, config_path);
            return Err(KarabinerPklError::PklCompileError {
                message: error_msg,
                line: line_number,
            });
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let config: Value = serde_json::from_str(&json_str)
            .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

        self.validate_config(&config)?;

        let mut final_config = config;
        if let Some(name) = profile_name {
            if let Some(profiles) = final_config
                .get_mut("profiles")
                .and_then(|p| p.as_array_mut())
            {
                if let Some(first_profile) = profiles.get_mut(0) {
                    first_profile["name"] = serde_json::json!(name);
                }
            }
        }

        Ok(final_config)
    }

    fn validate_config(&self, config: &Value) -> Result<()> {
        if !config.is_object() {
            return Err(KarabinerPklError::ValidationError {
                message: "Configuration must be an object".to_string(),
            });
        }

        if config.get("profiles").is_none() {
            return Err(KarabinerPklError::ValidationError {
                message: "Configuration must contain 'profiles' field".to_string(),
            });
        }

        let profiles = config.get("profiles").unwrap();
        if !profiles.is_array() {
            return Err(KarabinerPklError::ValidationError {
                message: "'profiles' must be an array".to_string(),
            });
        }

        let profiles_array = profiles.as_array().unwrap();
        if profiles_array.is_empty() {
            return Err(KarabinerPklError::ValidationError {
                message: "Configuration must contain at least one profile".to_string(),
            });
        }

        Ok(())
    }

    fn parse_pkl_error(stderr: &str, config_path: &Path) -> (String, usize) {
        static LINE_REGEX: OnceLock<Regex> = OnceLock::new();
        let line_regex = LINE_REGEX.get_or_init(|| Regex::new(r"line (\d+)\)").unwrap());

        let error_message = stderr
            .lines()
            .nth(1)
            .map(|line| line.trim().to_string())
            .unwrap_or_else(|| "Compilation failed".to_string());

        let config_file_name = config_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ankura.pkl");

        let line_number = stderr
            .lines()
            .find(|line| line.contains(config_file_name))
            .and_then(|line| line_regex.captures(line))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);

        (error_message, line_number)
    }

    pub fn materialize_pkl_lib() -> Result<PathBuf> {
        let data_dir = PathBuf::from(ANKURA_LIB_DIR);

        debug!(
            "Attempting to materialize pkl files to {}",
            data_dir.display()
        );

        std::fs::create_dir_all(&data_dir).map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to create data directory: {e}"),
        })?;

        let embedded_hash = Self::calculate_embedded_hash();
        let hash_file = data_dir.join(".pkl-hash");

        let should_extract = if let Ok(stored_hash) = std::fs::read_to_string(&hash_file) {
            stored_hash.trim() != embedded_hash.to_string()
        } else {
            true
        };

        if should_extract {
            debug!("Extracting embedded pkl files to {}", data_dir.display());

            for file in PklLib::iter() {
                let file_path = data_dir.join(file.as_ref());

                if let Some(content) = PklLib::get(&file) {
                    std::fs::write(&file_path, content.data).map_err(|e| {
                        KarabinerPklError::DaemonError {
                            message: format!(
                                "Failed to write embedded file {}: {}",
                                file.as_ref(),
                                e
                            ),
                        }
                    })?;
                }
            }

            std::fs::write(&hash_file, embedded_hash.to_string()).map_err(|e| {
                KarabinerPklError::DaemonError {
                    message: format!("Failed to write hash file: {e}"),
                }
            })?;
        }

        Ok(data_dir)
    }

    fn calculate_embedded_hash() -> u64 {
        let mut hasher = DefaultHasher::new();

        let mut files: Vec<_> = PklLib::iter().collect();
        files.sort();

        for file in files {
            file.hash(&mut hasher);
            if let Some(content) = PklLib::get(&file) {
                content.data.hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    pub fn lib_dir() -> PathBuf {
        PathBuf::from(ANKURA_LIB_DIR)
    }
}
