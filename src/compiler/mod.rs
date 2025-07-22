use crate::error::{KarabinerPklError, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;
use which::which;

pub struct Compiler {
    pkl_path: PathBuf,
    karabiner_config_dir: PathBuf,
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

        Ok(Self {
            pkl_path,
            karabiner_config_dir,
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

        let output = Command::new(&self.pkl_path)
            .args(["eval", "--format=json"])
            .arg(config_path)
            .output()
            .map_err(|e| KarabinerPklError::DaemonError {
                message: format!("Failed to execute pkl: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let source = std::fs::read_to_string(config_path).map_err(|e| {
                KarabinerPklError::ConfigReadError {
                    path: config_path.to_path_buf(),
                    source: e,
                }
            })?;

            let span = Self::parse_pkl_error_location(&stderr);

            return Err(KarabinerPklError::PklCompileError {
                help: stderr.to_string(),
                source_code: source,
                span,
            });
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let mut config: Value = serde_json::from_str(&json_str)
            .map_err(|e| KarabinerPklError::JsonParseError { source: e })?;

        self.validate_config(&config)?;

        if !config.get("title").is_some() {
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

        info!("Successfully wrote configuration to {}", karabiner_json_path.display());
        Ok(())
    }

    fn validate_config(&self, config: &Value) -> Result<()> {
        if !config.is_object() {
            return Err(KarabinerPklError::ValidationError {
                message: "Configuration must be an object".to_string(),
            });
        }

        if !config.get("profiles").is_some() {
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

        Ok(())
    }

    fn parse_pkl_error_location(error_str: &str) -> Option<miette::SourceSpan> {
        if let Some(line_match) = error_str.find("line ") {
            let rest = &error_str[line_match + 5..];
            if let Some(comma) = rest.find(',') {
                if let Ok(line) = rest[..comma].trim().parse::<usize>() {
                    if let Some(col_match) = rest.find("column ") {
                        let col_rest = &rest[col_match + 7..];
                        if let Some(end) = col_rest.find(|c: char| !c.is_numeric()) {
                            if let Ok(col) = col_rest[..end].parse::<usize>() {
                                let offset = (line - 1) * 80 + col;
                                return Some(miette::SourceSpan::new(offset.into(), 1));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}