# API Reference

This document provides a detailed reference for the public APIs in Karabiner-Pkl.

## Compiler API

### `karabiner_pkl::compiler::Compiler`

The main compiler struct responsible for converting Pkl configurations to Karabiner JSON.

#### Constructor

```rust
pub fn new() -> Result<Self>
```

Creates a new Compiler instance.

**Returns:** 
- `Ok(Compiler)` on success
- `Err(KarabinerPklError::PklNotFound)` if pkl CLI is not found in PATH

**Side Effects:**
- Extracts embedded Pkl library to temporary directory

#### Methods

##### `compile`

```rust
pub async fn compile(&self, config_path: &Path, profile_name: Option<&str>) -> Result<Value>
```

Compiles a Pkl configuration file to JSON.

**Parameters:**
- `config_path`: Path to the .pkl configuration file
- `profile_name`: Optional profile name override

**Returns:**
- `Ok(Value)` containing the compiled JSON on success
- `Err(KarabinerPklError)` on failure

**Process:**
1. Reads the Pkl file
2. Invokes pkl CLI with proper module paths
3. Applies profile name override if provided
4. Returns JSON (no file I/O)

## CLI API

### `karabiner_pkl::cli`

CLI module functions for file operations and configuration management.

#### Functions

##### `merge_configurations`

```rust
pub fn merge_configurations(existing_path: &Path, new_config: Value) -> Result<Value>
```

Merges a new profile configuration into an existing Karabiner JSON file.

**Parameters:**
- `existing_path`: Path to existing karabiner.json file
- `new_config`: New configuration to merge

**Returns:**
- `Ok(Value)` containing merged configuration
- `Err(KarabinerPklError)` on failure

**Behavior:**
- Preserves all existing profiles
- Updates profile if name matches
- Adds new profile if name doesn't exist
- New profiles are added with `selected: false`

##### `write_karabiner_config`

```rust
pub fn write_karabiner_config(path: &Path, config: &Value) -> Result<()>
```

Writes a Karabiner configuration to file.

**Parameters:**
- `path`: Output file path
- `config`: Configuration JSON to write

**Returns:**
- `Ok(())` on success
- `Err(KarabinerPklError)` on failure

**Side Effects:**
- Creates parent directories if missing
- Writes formatted JSON to file

## Daemon API

### `karabiner_pkl::daemon::Daemon`

File watching daemon for automatic recompilation.

#### Constructor

```rust
pub fn new(config_path: PathBuf) -> Self
```

Creates a new Daemon instance.

**Parameters:**
- `config_path`: Path to the Pkl configuration file to watch

#### Methods

##### `start`

```rust
pub async fn start(&self) -> Result<()>
```

Starts the file watching daemon.

**Returns:**
- `Ok(())` when daemon starts successfully
- `Err(KarabinerPklError::WatchError)` on file watching errors

**Behavior:**
- Compiles configuration on startup
- Watches for file changes with 300ms debounce
- Sends notifications on success/failure
- Continues running on compilation errors

##### `stop`

```rust
pub async fn stop(&self) -> Result<()>
```

Stops the daemon gracefully.

**Returns:**
- `Ok(())` on successful shutdown

##### `compile_once`

```rust
pub async fn compile_once(&self, profile_name: Option<&str>, output_path: Option<&str>) -> Result<()>
```

Performs a one-time compilation without starting the watcher.

**Parameters:**
- `profile_name`: Optional profile name override (unused)
- `output_path`: Optional output path (unused)

**Returns:**
- `Ok(())` on successful compilation
- `Err(KarabinerPklError)` on failure

**Note:** This method is currently unused and may be removed in future versions.

## Import API

### `karabiner_pkl::import::Importer`

Manages importing external Pkl modules.

#### Constructor

```rust
pub fn new() -> Result<Self>
```

Creates a new Importer instance.

**Side Effects:**
- Creates `~/.config/karabiner_pkl/lib/` directory if missing

#### Methods

##### `import`

```rust
pub async fn import(&self, source: &str, name: Option<String>) -> Result<()>
```

Imports a Pkl module from a file path or URL.

**Parameters:**
- `source`: File path or HTTP(S) URL to .pkl file
- `name`: Optional custom name for the imported file

**Returns:**
- `Ok(())` on successful import
- `Err(KarabinerPklError)` on failure

**Behavior:**
- Downloads from URL or copies from file path
- Validates .pkl extension
- Warns on overwrites

##### `list_imports`

```rust
pub fn list_imports(&self) -> Result<Vec<String>>
```

Lists all imported .pkl files in the library directory.

**Returns:**
- `Ok(Vec<String>)` with filenames of all .pkl files
- `Err(KarabinerPklError)` on directory read errors

##### `get_lib_dir`

```rust
pub fn get_lib_dir(&self) -> &Path
```

Returns the path to the user's Pkl library directory.

**Returns:** Reference to the lib directory path (`~/.config/karabiner_pkl/lib/`)

## Error Types

### `karabiner_pkl::error::KarabinerPklError`

Main error type for all operations.

```rust
pub enum KarabinerPklError {
    PklNotFound,
    ConfigReadError { path: PathBuf, source: std::io::Error },
    PklCompileError { message: String, source_code: String, span: Option<(usize, usize)> },
    JsonParseError { source: serde_json::Error },
    KarabinerWriteError { path: PathBuf, source: std::io::Error },
    ValidationError { message: String },
    WatchError { source: notify::Error },
    DaemonError { message: String },
    ConfigWriteError { path: PathBuf, source: std::io::Error },
}
```

All errors implement:
- `std::error::Error`
- `miette::Diagnostic` for rich error display

### Type Alias

```rust
pub type Result<T> = std::result::Result<T, KarabinerPklError>;
```

## Notification API

### `karabiner_pkl::notifications::NotificationManager`

Handles system notifications for user feedback.

#### Constructor

```rust
pub fn new() -> Self
```

Creates a new NotificationManager instance.

#### Methods

##### `send_success`

```rust
pub fn send_success(&self, message: &str)
```

Sends a success notification with 3-second timeout.

##### `send_error`

```rust
pub fn send_error(&self, message: &str)
```

Sends an error notification that persists until dismissed.

##### `send_info`

```rust
pub fn send_info(&self, message: &str)
```

Sends an info notification with 3-second timeout.

## Embedded Resources API

### `karabiner_pkl::embedded::extract_pkl_lib`

```rust
pub fn extract_pkl_lib() -> Result<(TempDir, PathBuf)>
```

Extracts embedded Pkl library files to a temporary directory.

**Returns:**
- `Ok((TempDir, PathBuf))` where:
  - `TempDir`: Temporary directory handle (keeps directory alive)
  - `PathBuf`: Base path for Pkl imports
- `Err(KarabinerPklError)` on extraction failure

### `karabiner_pkl::embedded::list_embedded_files`

```rust
pub fn list_embedded_files() -> Vec<String>
```

Lists all embedded Pkl file names.

**Returns:** Vector of file names (e.g., `["karabiner.pkl", "helpers.pkl"]`)

## Logging API

### `karabiner_pkl::logging::setup_logging`

```rust
pub fn setup_logging(debug: bool) -> Result<PathBuf>
```

Initializes the logging system.

**Parameters:**
- `debug`: Enable debug-level logging

**Returns:**
- `Ok(PathBuf)` with path to log file
- `Err(KarabinerPklError)` on setup failure

**Side Effects:**
- Creates log directory at `~/.local/share/karabiner-pkl/logs/`
- Configures console and file output
- Sets up tracing subscriber

## Usage Examples

### Basic Compilation

```rust
use karabiner_pkl::compiler::Compiler;
use std::path::Path;

let compiler = Compiler::new()?;
compiler.compile(Path::new("/path/to/config.pkl"))?;
```

### Daemon Usage

```rust
use karabiner_pkl::daemon::Daemon;
use std::path::PathBuf;

let daemon = Daemon::new(PathBuf::from("/path/to/config.pkl"));

// Start daemon (in async context)
daemon.start().await?;

// Or compile once
daemon.compile_once().await?;
```

### Import Module

```rust
use karabiner_pkl::import::Importer;

let importer = Importer::new()?;

// Import from URL
importer.import("https://example.com/layers.pkl", None).await?;

// Import from file with custom name
importer.import("/path/to/local.pkl", Some("my-layers")).await?;

// List imports
let imports = importer.list_imports()?;
```

### Error Handling

```rust
use karabiner_pkl::error::{KarabinerPklError, Result};

fn compile_config() -> Result<()> {
    match Compiler::new()?.compile(Path::new("config.pkl")) {
        Ok(()) => println!("Success!"),
        Err(KarabinerPklError::PklCompileError { message, .. }) => {
            eprintln!("Compilation failed: {}", message);
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
```