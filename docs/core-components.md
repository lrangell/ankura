# Core Components

This document provides detailed information about the core components of Karabiner-Pkl.

## CLI Module (`src/cli.rs`)

The CLI module defines the command-line interface using the `clap` crate.

### Structure

```rust
struct Cli {
    command: Commands,
    config: PathBuf,  // Path to Pkl config (default: ~/.config/karabiner.pkl)
    debug: bool,      // Enable debug logging
}
```

### Available Commands

| Command | Description | Options |
|---------|-------------|---------|
| `start` | Start the daemon | `--foreground`: Run in foreground |
| `stop` | Stop the daemon | - |
| `compile` | Compile configuration once | - |
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

# Watch logs in real-time
karabiner-pkl logs --follow

# Import a module from URL
karabiner-pkl add https://example.com/layers.pkl --name my-layers
```

## Compiler Module (`src/compiler/mod.rs`)

The Compiler handles Pkl to JSON compilation by invoking the external Pkl CLI.

### Key Components

```rust
struct Compiler {
    pkl_path: PathBuf,              // Path to pkl binary
    karabiner_config_dir: PathBuf,  // Output directory
    _embedded_temp_dir: TempDir,    // Keeps temp dir alive
    embedded_lib_path: PathBuf,     // Extracted pkl-lib path
}
```

### Compilation Process

1. **Initialization**
   - Locates Pkl CLI binary using `which`
   - Creates Karabiner config directory if missing
   - Extracts embedded Pkl library to temp directory

2. **Compilation**
   - Constructs module paths (embedded + user lib)
   - Invokes Pkl CLI with proper arguments
   - Parses JSON output and extracts `config` field
   - Validates configuration structure

3. **Validation**
   - Ensures `profiles` array exists
   - Verifies "Default" profile is present
   - Checks JSON structure matches expectations

4. **Error Handling**
   - Parses Pkl error output for location info
   - Creates rich diagnostics with source display
   - Provides helpful error messages

### Key Methods

- `new() -> Result<Self>`: Creates compiler instance
- `compile(&self, config_path: &Path) -> Result<()>`: Main compilation function
- `validate_config(config: &Value) -> Result<()>`: Validates JSON structure

## Daemon Module (`src/daemon/mod.rs`)

The Daemon provides file watching and auto-compilation functionality.

### Architecture

```rust
struct Daemon {
    config_path: PathBuf,
    compiler: Arc<Compiler>,
    notification_manager: NotificationManager,
    is_running: Arc<RwLock<bool>>,
}
```

### Features

1. **File Watching**
   - Uses `notify` crate with debouncer (300ms delay)
   - Watches specific config file, not entire directory
   - Filters events to only respond to modifications

2. **Compilation Loop**
   - Compiles on startup
   - Recompiles on every file change
   - Continues running on compilation errors

3. **Notifications**
   - Success: "✅ Configuration compiled successfully"
   - Error: "❌ Configuration error: [details]"

4. **Async Operation**
   - Runs on tokio runtime
   - Non-blocking file watching
   - Graceful shutdown support

### Key Methods

- `new(config_path: PathBuf) -> Self`: Creates daemon instance
- `start() -> Result<()>`: Starts file watching
- `stop()`: Stops the daemon
- `compile_once() -> Result<()>`: One-time compilation

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

- `new() -> Result<Self>`: Creates importer
- `import(&self, source: &str, name: Option<&str>) -> Result<()>`: Import module
- `list_imports() -> Result<Vec<PathBuf>>`: List imported modules

## Notifications Module (`src/notifications/mod.rs`)

Provides macOS native notifications for user feedback.

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

## Embedded Module (`src/embedded.rs`)

Embeds Pkl library files in the binary for distribution.

### Process

1. **Build Time**
   - `rust_embed` includes pkl-lib/*.pkl files
   - Files become part of binary

2. **Runtime**
   - Extracts to temp directory
   - Creates proper module structure
   - Returns path for Pkl imports

### Embedded Files

- `karabiner.pkl`: Core type definitions
- `helpers.pkl`: Helper functions and constants

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