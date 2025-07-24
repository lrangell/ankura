use crate::error::{KarabinerPklError, Result};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub struct Importer {
    lib_dir: PathBuf,
}

impl Importer {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| KarabinerPklError::DaemonError {
            message: "Could not find home directory".to_string(),
        })?;
        let lib_dir = home.join(".config/karabiner_pkl/lib");

        std::fs::create_dir_all(&lib_dir).map_err(|e| {
            KarabinerPklError::ConfigReadError {
                path: lib_dir.clone(),
                source: e,
            }
        })?;

        Ok(Self { lib_dir })
    }

    pub async fn import(&self, source: &str, name: Option<String>) -> Result<()> {
        if source.starts_with("http://") || source.starts_with("https://") {
            self.import_from_url(source, name).await
        } else {
            self.import_from_file(source, name)
        }
    }

    async fn import_from_url(&self, url: &str, name: Option<String>) -> Result<()> {
        info!("Importing from URL: {}", url);

        let response = reqwest::get(url).await.map_err(|e| {
            KarabinerPklError::DaemonError {
                message: format!("Failed to download file: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(KarabinerPklError::DaemonError {
                message: format!("Failed to download file: HTTP {}", response.status()),
            });
        }

        let content = response.text().await.map_err(|e| {
            KarabinerPklError::DaemonError {
                message: format!("Failed to read response: {}", e),
            }
        })?;

        let filename = name.unwrap_or_else(|| {
            url.split('/')
                .next_back()
                .unwrap_or("imported.pkl")
                .to_string()
        });

        if !filename.ends_with(".pkl") {
            return Err(KarabinerPklError::ValidationError {
                message: "Imported files must have .pkl extension".to_string(),
            });
        }

        let target_path = self.lib_dir.join(&filename);
        
        if target_path.exists() {
            warn!("File {} already exists in lib directory. Overwriting.", filename);
        }

        std::fs::write(&target_path, content).map_err(|e| {
            KarabinerPklError::ConfigWriteError {
                path: target_path.clone(),
                source: e,
            }
        })?;

        info!("Successfully imported {} to {}", url, target_path.display());
        Ok(())
    }

    fn import_from_file(&self, path: &str, name: Option<String>) -> Result<()> {
        let source_path = Path::new(path);
        
        if !source_path.exists() {
            return Err(KarabinerPklError::ConfigReadError {
                path: source_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Source file not found",
                ),
            });
        }

        if !path.ends_with(".pkl") {
            return Err(KarabinerPklError::ValidationError {
                message: "Source file must have .pkl extension".to_string(),
            });
        }

        let filename = name.unwrap_or_else(|| {
            source_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });

        let target_path = self.lib_dir.join(&filename);
        
        if target_path.exists() {
            warn!("File {} already exists in lib directory. Overwriting.", filename);
        }

        std::fs::copy(source_path, &target_path).map_err(|e| {
            KarabinerPklError::ConfigReadError {
                path: source_path.to_path_buf(),
                source: e,
            }
        })?;

        info!("Successfully imported {} to {}", path, target_path.display());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    #[allow(dead_code)]
    pub fn list_imports(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        
        let entries = std::fs::read_dir(&self.lib_dir).map_err(|e| {
            KarabinerPklError::ConfigReadError {
                path: self.lib_dir.clone(),
                source: e,
            }
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| KarabinerPklError::ConfigReadError {
                path: self.lib_dir.clone(),
                source: e,
            })?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("pkl") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    files.push(filename.to_string());
                }
            }
        }

        files.sort();
        Ok(files)
    }
}