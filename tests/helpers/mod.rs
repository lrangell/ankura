use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use serde_json::Value;
use miette::SourceSpan;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub pkl_path: PathBuf,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pkl_path = which::which("pkl").expect("pkl not found in PATH");
        
        Self {
            temp_dir,
            pkl_path,
        }
    }
    
    pub fn write_pkl_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::write(&path, content).expect("Failed to write test file");
        
        // Copy pkl-lib files to temp directory for imports to work
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let pkl_lib_dir = PathBuf::from(manifest_dir).join("pkl-lib");
        
        // Copy karabiner.pkl and helpers.pkl
        if let Ok(karabiner_content) = std::fs::read_to_string(pkl_lib_dir.join("karabiner.pkl")) {
            std::fs::write(self.temp_dir.path().join("karabiner.pkl"), karabiner_content)
                .expect("Failed to copy karabiner.pkl");
        }
        if let Ok(helpers_content) = std::fs::read_to_string(pkl_lib_dir.join("helpers.pkl")) {
            std::fs::write(self.temp_dir.path().join("helpers.pkl"), helpers_content)
                .expect("Failed to copy helpers.pkl");
        }
        
        path
    }
    
    pub fn compile_pkl_to_json(&self, pkl_file: &Path) -> Result<Value, String> {
        // Find the pkl-lib directory relative to test execution
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let pkl_lib_dir = PathBuf::from(manifest_dir).join("pkl-lib");
        
        let output = Command::new(&self.pkl_path)
            .args(["eval", "--format=json"])
            .arg("--module-path")
            .arg(&pkl_lib_dir)
            .arg(pkl_file)
            .output()
            .map_err(|e| format!("Failed to execute pkl: {e}"))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Try to create a nice error message
            if let Ok(source) = std::fs::read_to_string(pkl_file) {
                use miette::{Diagnostic, NamedSource, SourceSpan};
                
                #[derive(Debug, Diagnostic, thiserror::Error)]
                #[error("Pkl compilation failed")]
                struct PklTestError {
                    #[source_code]
                    src: NamedSource<String>,
                    
                    #[label("error occurred here")]
                    span: Option<SourceSpan>,
                    
                    #[help]
                    help: String,
                }
                
                // Parse error location
                let span = Self::parse_pkl_error_span(&stderr, &source);
                let error_msg = Self::extract_pkl_error_message(&stderr);
                
                let err = PklTestError {
                    src: NamedSource::new(
                        pkl_file.file_name().unwrap().to_string_lossy(),
                        source,
                    ),
                    span,
                    help: error_msg,
                };
                
                // Use miette to format the error nicely
                return Err(format!("{:?}", miette::Report::new(err)));
            }
            
            return Err(format!("Pkl compilation failed: {stderr}"));
        }
        
        let json_str = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse JSON: {e}"))
    }
    
    fn parse_pkl_error_span(error_str: &str, source_code: &str) -> Option<SourceSpan> {
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
                    
                    return Some(SourceSpan::new(offset.into(), 1));
                }
            }
        }
        None
    }
    
    fn extract_pkl_error_message(error_str: &str) -> String {
        if let Some(start) = error_str.find("–– Pkl Error ––") {
            let msg_start = start + "–– Pkl Error ––".len();
            if let Some(end) = error_str[msg_start..].find("\n\n") {
                error_str[msg_start..msg_start + end].trim().to_string()
            } else {
                error_str[msg_start..].trim().to_string()
            }
        } else {
            error_str.to_string()
        }
    }
    
    pub fn load_fixture(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to load fixture: {name}"))
    }
}