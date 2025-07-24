use crate::error::{KarabinerPklError, Result};
use rust_embed::RustEmbed;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use tracing::info;
use which::which;

#[derive(RustEmbed)]
#[folder = "pkl-lib/"]
#[include = "*.pkl"]
struct PklLib;

pub struct Compiler {
    pkl_path: PathBuf,
    _embedded_temp_dir: Option<TempDir>,
    embedded_lib_path: Option<PathBuf>,
}

impl Compiler {
    pub fn new() -> Result<Self> {
        let pkl_path = which("pkl").map_err(|_| KarabinerPklError::PklNotFound)?;

        // Extract embedded pkl-lib files once when creating the compiler
        let (temp_dir, embedded_lib_path) = Self::extract_pkl_lib()?;

        Ok(Self {
            pkl_path,
            _embedded_temp_dir: Some(temp_dir),
            embedded_lib_path: Some(embedded_lib_path),
        })
    }

    /// Compile a Pkl configuration file and return the resulting JSON
    /// Returns the compiled configuration with optional profile name override
    pub async fn compile(&self, config_path: &Path, profile_name: Option<&str>) -> Result<Value> {
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
        let config = pkl_output
            .get("config")
            .ok_or_else(|| KarabinerPklError::ValidationError {
                message: "Pkl output must contain a 'config' field".to_string(),
            })?
            .clone();

        self.validate_config(&config)?;

        // Apply profile name override if provided
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

        // No need to check for specific profile name anymore
        // Just ensure we have at least one profile which we already checked above

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

    /// Extract embedded pkl-lib files to a temporary directory with the expected structure
    /// Returns the path to the directory containing the module structure
    fn extract_pkl_lib() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new().map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to create temporary directory: {e}"),
        })?;

        // Create the expected directory structure: karabiner_pkl/lib/
        let module_base = temp_dir.path().join("karabiner_pkl").join("lib");
        std::fs::create_dir_all(&module_base).map_err(|e| KarabinerPklError::DaemonError {
            message: format!("Failed to create module directory structure: {e}"),
        })?;

        // Extract all embedded pkl files
        for file in PklLib::iter() {
            let file_path = module_base.join(file.as_ref());

            if let Some(content) = PklLib::get(&file) {
                // Extract the file
                std::fs::write(&file_path, content.data).map_err(|e| {
                    KarabinerPklError::DaemonError {
                        message: format!("Failed to write embedded file {}: {}", file.as_ref(), e),
                    }
                })?;
            }
        }

        // Return both the TempDir (to keep it alive) and the base path for module resolution
        let base_path = temp_dir.path().to_path_buf();
        // Module structure created successfully
        Ok((temp_dir, base_path))
    }
}
