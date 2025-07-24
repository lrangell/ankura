use crate::embedded;
use crate::error::{KarabinerPklError, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use tracing::info;
use which::which;

pub struct Compiler {
    pkl_path: PathBuf,
    karabiner_config_dir: PathBuf,
    _embedded_temp_dir: Option<TempDir>,
    embedded_lib_path: Option<PathBuf>,
}

impl Compiler {
    pub fn new() -> Result<Self> {
        let pkl_path = which("pkl").map_err(|_| KarabinerPklError::PklNotFound)?;

        let home = dirs::home_dir().ok_or_else(|| KarabinerPklError::DaemonError {
            message: "Could not find home directory".to_string(),
        })?;
        let karabiner_config_dir = home.join(".config/karabiner");

        std::fs::create_dir_all(&karabiner_config_dir).map_err(|e| {
            KarabinerPklError::KarabinerWriteError {
                path: karabiner_config_dir.clone(),
                source: e,
            }
        })?;

        // Extract embedded pkl-lib files once when creating the compiler
        let (temp_dir, embedded_lib_path) = embedded::extract_pkl_lib()?;

        Ok(Self {
            pkl_path,
            karabiner_config_dir,
            _embedded_temp_dir: Some(temp_dir),
            embedded_lib_path: Some(embedded_lib_path),
        })
    }

    pub async fn compile(&self, config_path: &Path) -> Result<()> {
        info!("Compiling {}", config_path.display());

        if !config_path.exists() {
            return Err(KarabinerPklError::ConfigReadError {
                path: config_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Configuration file not found",
                ),
            });
        }

        // Set up module paths for imports
        let home = dirs::home_dir().ok_or_else(|| KarabinerPklError::DaemonError {
            message: "Could not find home directory".to_string(),
        })?;
        let lib_dir = home.join(".config/karabiner_pkl/lib");

        let mut pkl_command = Command::new(&self.pkl_path);
        pkl_command.args(["eval", "--format=json"]);

        // Add module paths
        let mut module_paths = vec![];

        // Always add the embedded library path first
        if let Some(embedded_path) = &self.embedded_lib_path {
            module_paths.push(embedded_path.to_string_lossy().to_string());
        }

        // Add user library directory if it exists
        if lib_dir.exists() {
            module_paths.push(lib_dir.to_string_lossy().to_string());
        }

        // Add the module path argument for modulepath: imports
        pkl_command.arg("--module-path");
        pkl_command.arg(module_paths.join(":"));
        let output =
            pkl_command
                .arg(config_path)
                .output()
                .map_err(|e| KarabinerPklError::DaemonError {
                    message: format!("Failed to execute pkl: {e}"),
                })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let source = std::fs::read_to_string(config_path).map_err(|e| {
                KarabinerPklError::ConfigReadError {
                    path: config_path.to_path_buf(),
                    source: e,
                }
            })?;

            let span = Self::parse_pkl_error_location(&stderr, &source);

            // Extract just the error message without the full traceback
            let error_msg = if let Some(start) = stderr.find("–– Pkl Error ––") {
                let msg_start = start + "–– Pkl Error ––".len();
                if let Some(end) = stderr[msg_start..].find("\n\n") {
                    stderr[msg_start..msg_start + end].trim().to_string()
                } else {
                    stderr[msg_start..].trim().to_string()
                }
            } else {
                stderr.to_string()
            };

            return Err(KarabinerPklError::PklCompileError {
                help: error_msg,
                source_code: source,
                span,
            });
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let pkl_output: Value = serde_json::from_str(&json_str)
            .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

        // Extract the 'config' field from the pkl output
        let mut config = pkl_output
            .get("config")
            .ok_or_else(|| KarabinerPklError::ValidationError {
                message: "Pkl output must contain a 'config' field".to_string(),
            })?
            .clone();

        self.validate_config(&config)?;

        if config.get("title").is_none() {
            config["title"] = serde_json::json!("Karabiner-Pkl Generated Configuration");
        }

        let karabiner_json_path = self.karabiner_config_dir.join("karabiner.json");
        let pretty_json = serde_json::to_string_pretty(&config)
            .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

        std::fs::write(&karabiner_json_path, pretty_json).map_err(|e| {
            KarabinerPklError::KarabinerWriteError {
                path: karabiner_json_path.clone(),
                source: e,
            }
        })?;

        info!(
            "Successfully wrote configuration to {}",
            karabiner_json_path.display()
        );
        Ok(())
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
        if !profiles.is_array() || profiles.as_array().unwrap().is_empty() {
            return Err(KarabinerPklError::ValidationError {
                message: "Configuration must contain at least one profile".to_string(),
            });
        }

        // Ensure we have a Default profile
        let profiles_array = profiles.as_array().unwrap();
        let has_default = profiles_array.iter().any(|p| {
            p.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n == "Default")
                .unwrap_or(false)
        });

        if !has_default {
            return Err(KarabinerPklError::ValidationError {
                message: "Configuration must contain a profile named 'Default'".to_string(),
            });
        }

        Ok(())
    }

    fn parse_pkl_error_location(error_str: &str, source_code: &str) -> Option<miette::SourceSpan> {
        // Look for the error location in the format "line X)"
        if let Some(line_match) = error_str.rfind("line ") {
            let rest = &error_str[line_match + 5..];
            if let Some(paren) = rest.find(')') {
                if let Ok(line_num) = rest[..paren].trim().parse::<usize>() {
                    // Find the column by looking for the caret (^) in the error output
                    let mut col = 1;
                    let lines: Vec<&str> = error_str.lines().collect();

                    // Look for the line with the caret
                    for line in lines.iter() {
                        if line.contains('^') {
                            col = line.find('^').unwrap_or(0) + 1;
                            break;
                        }
                    }

                    // Calculate the byte offset in the source
                    let mut offset = 0;
                    for (idx, line) in source_code.lines().enumerate() {
                        if idx + 1 == line_num {
                            offset += col.saturating_sub(1);
                            break;
                        }
                        offset += line.len() + 1; // +1 for newline
                    }

                    return Some(miette::SourceSpan::new(offset.into(), 1));
                }
            }
        }
        None
    }
}
