# Core Components

This document provides detailed information about the core components of Karabiner-Pkl.

## CLI Module (`src/cli.rs`)

The CLI module defines the command-line interface and handles all file I/O operations.

### Structure

```rust
struct Cli {
    command: Commands,
    config: String,   // Path to Pkl config (default: ~/.config/karabiner.pkl)
    debug: bool,      // Enable debug logging
}
```

### Available Commands

| Command | Description | Options |
|---------|-------------|---------|
| `start` | Start the daemon | `--foreground`: Run in foreground |
| `stop` | Stop the daemon | - |
| `compile` | Compile configuration once | `--profile-name`: Override profile name<br>`--output`: Custom output path |
| `check` | Validate configuration | - |
| `logs` | View daemon logs | `--lines N`: Show last N lines<br>`--follow`: Follow log output |
| `status` | Check daemon status | - |
| `init` | Initialize example config | `--force`: Overwrite existing |
| `add` | Import Pkl module | `<source>`: File path or URL<br>`--name`: Custom name |

### Usage Examples

```bash
# Start daemon in background
karabiner-pkl start

# Compile configuration once
karabiner-pkl compile

# Compile with custom profile name
karabiner-pkl compile --profile-name "Work"

# Compile to specific output file
karabiner-pkl compile --output ~/test-karabiner.json

# Watch logs in real-time
karabiner-pkl logs --follow

# Import a module from URL
karabiner-pkl add https://example.com/layers.pkl --name my-layers
```

### Key Responsibilities

1. **Command Implementation**: All command logic is in this module
2. **File I/O Operations**: 
   - Reading existing Karabiner configurations
   - Writing compiled JSON output
   - Merging configurations to preserve other profiles
3. **Profile Management**: Handles profile preservation and merging

### Key Functions

- `merge_configurations(existing_path, new_config) -> Result<Value>`: Merges new profile into existing config
- `write_karabiner_config(path, config) -> Result<()>`: Writes JSON to file

## Compiler Module (`src/compiler/mod.rs`)

The Compiler is a pure transformation module that converts Pkl files to JSON without any file I/O.

### Key Components

```rust
struct Compiler {
    pkl_path: PathBuf,              // Path to pkl binary
    _embedded_temp_dir: TempDir,    // Keeps temp dir alive
    embedded_lib_path: PathBuf,     // Extracted pkl path
}
```

### Compilation Process

1. **Initialization**
   - Locates Pkl CLI binary using `which`
   - Extracts embedded Pkl library to temp directory
   - Sets up module paths

2. **Pure Compilation**
   - Takes a Pkl file path and optional profile name
   - Constructs module paths (embedded + user lib)
   - Invokes Pkl CLI with proper arguments
   - Returns JSON as `serde_json::Value`
   - No file writing or directory creation

3. **Profile Override**
   - Can override profile name via CLI parameter
   - Passes override to Pkl as external property

4. **Error Handling**
   - Parses Pkl error output for location info
   - Creates rich diagnostics with source display
   - Returns errors without side effects

### Key Methods

- `new() -> Result<Self>`: Creates compiler instance
- `compile(&self, config_path: &Path, profile_name: Option<&str>) -> Result<Value>`: Returns JSON

### Embedded Resources

The compiler module now includes embedded Pkl library files using `rust_embed`:
- Extracts `karabiner.pkl` and `helpers.pkl` at runtime
- Makes them available via `modulepath:/`

## Daemon Module (`src/daemon/mod.rs`)

The Daemon provides file watching, auto-compilation, and notification functionality.

### Architecture

```rust
struct Daemon {
    config_path: PathBuf,
    compiler: Arc<Compiler>,
    notification_manager: NotificationManager,
    is_running: Arc<RwLock<bool>>,
}

struct NotificationManager {
    app_name: String,
}
```

### Features

1. **File Watching**
   - Uses `notify` crate with debouncer (300ms delay)
   - Watches specific config file, not entire directory
   - Filters events to only respond to modifications

2. **Compilation Loop**
   - Gets JSON from compiler
   - Merges with existing configuration
   - Writes to Karabiner config file
   - Shows notifications for results

3. **Integrated Notifications**
   - Success: "✅ Configuration compiled successfully"
   - Error: "❌ Configuration error: [details]"
   - NotificationManager is now part of daemon module

4. **Async Operation**
   - Runs on tokio runtime
   - Non-blocking file watching
   - Graceful shutdown support

### Key Methods

- `new(config_path: PathBuf) -> Result<Self>`: Creates daemon with compiler and notifications
- `start() -> Result<()>`: Starts file watching
- `stop() -> Result<()>`: Stops the daemon
- `compile_once(profile_name, output_path) -> Result<()>`: One-time compilation (unused)

## Error Module (`src/error.rs`)

Centralized error handling using `miette` for rich diagnostics.

### Error Types

```rust
enum KarabinerPklError {
    PklNotFound,         // Pkl CLI not in PATH
    ConfigReadError,     // Can't read config file
    PklCompileError,     // Pkl compilation failed
    JsonParseError,      // Invalid JSON output
    KarabinerWriteError, // Can't write output
    ValidationError,     // Config validation failed
    WatchError,          // File watching error
    DaemonError,         // General daemon error
    ConfigWriteError,    // Can't write config
}
```

### Features

1. **Rich Diagnostics**
   - Source code display with error location
   - Syntax highlighting in terminal
   - Helpful error messages and suggestions

2. **Error Context**
   - File paths and line numbers
   - Relevant code snippets
   - Recovery suggestions

3. **Integration**
   - Works with `thiserror` for derivation
   - Implements `miette::Diagnostic`
   - Type alias: `Result<T> = std::result::Result<T, KarabinerPklError>`

## Import Module (`src/import/mod.rs`)

Manages importing external Pkl modules.

### Functionality

1. **Import Sources**
   - Local file paths
   - HTTP/HTTPS URLs
   - Validates .pkl extension

2. **Library Management**
   - Default location: `~/.config/karabiner_pkl/lib/`
   - Creates directory structure if missing
   - Warns on overwrites

3. **Import Process**
   - Downloads from URL or copies from path
   - Preserves original filename or uses custom name
   - Lists existing imports

### Key Methods

- `new() -> Result<Self>`: Creates importer with lib directory setup
- `import(&self, source: &str, name: Option<String>) -> Result<()>`: Import module
- `list_imports() -> Result<Vec<String>>`: List imported modules

## Notifications (in Daemon Module)

Notification functionality is now integrated into the daemon module.

### Notification Types

1. **Success** (✅)
   - 3-second timeout
   - Configuration compiled successfully

2. **Error** (❌)
   - No timeout (persistent)
   - Shows error details

3. **Info** (ℹ️)
   - 3-second timeout
   - General information

### Implementation

- Uses `notify-rust` crate
- Graceful fallback on failure
- App name: "Karabiner-Pkl"
- Managed by `NotificationManager` struct within daemon

## Embedded Resources (in Compiler Module)

Embedded Pkl library functionality is now integrated into the compiler module.

### Process

1. **Build Time**
   - `rust_embed` includes pkl/*.pkl files
   - Files become part of binary

2. **Runtime**
   - Extracts to temp directory during compiler initialization
   - Creates proper module structure
   - Available via `modulepath:/`

### Embedded Files

- `karabiner.pkl`: Core type definitions (default profile: "pkl")
- `helpers.pkl`: Helper functions and constants

## Main Entry Point (`src/main.rs`)

The main entry point is now simplified to only:
1. Parse CLI arguments with clap
2. Initialize logging
3. Expand config path (tilde expansion)
4. Dispatch to CLI module for execution

All command implementations have been moved to the CLI module for better separation of concerns.

## Logging Module (`src/logging.rs`)

Structured logging setup using `tracing`.

### Configuration

1. **Output Targets**
   - Console: ANSI colors, formatted output
   - File: `~/.local/share/karabiner-pkl/logs/karabiner-pkl.log`

2. **Log Levels**
   - karabiner_pkl: info
   - Other crates: warn

3. **Features**
   - Append mode for file logging
   - Structured fields
   - Performance metrics