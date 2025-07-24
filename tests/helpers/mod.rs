use karabiner_pkl::compiler::Compiler;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub compiler: Compiler,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let compiler = Compiler::new().expect("Failed to create compiler");

        Self { temp_dir, compiler }
    }

    pub fn write_pkl_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::write(&path, content).expect("Failed to write test file");
        path
    }

    pub async fn compile_pkl(
        &self,
        pkl_file: &Path,
        profile_name: Option<&str>,
    ) -> Result<Value, String> {
        self.compiler
            .compile(pkl_file, profile_name)
            .await
            .map_err(|e| format!("Compilation failed: {e}"))
    }

    // Synchronous wrapper for tests that don't use async
    pub fn compile_pkl_sync(
        &self,
        pkl_file: &Path,
        profile_name: Option<&str>,
    ) -> Result<Value, String> {
        tokio_test::block_on(self.compile_pkl(pkl_file, profile_name))
    }

    pub fn load_fixture(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to load fixture: {name}"))
    }
}
