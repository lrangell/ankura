use crate::error::{KarabinerPklError, Result};
use rust_embed::RustEmbed;
use std::path::PathBuf;
use tempfile::TempDir;

#[derive(RustEmbed)]
#[folder = "pkl-lib/"]
#[include = "*.pkl"]
struct PklLib;

/// Extract embedded pkl-lib files to a temporary directory with the expected structure
/// Returns the path to the directory containing the module structure
pub fn extract_pkl_lib() -> Result<(TempDir, PathBuf)> {
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

/// Get the list of embedded pkl files (for debugging/info)
#[allow(dead_code)]
pub fn list_embedded_files() -> Vec<String> {
    PklLib::iter().map(|f| f.to_string()).collect()
}
