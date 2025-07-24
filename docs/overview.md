# Karabiner-Pkl Overview

Karabiner-Pkl is a configuration tool for [Karabiner-Elements](https://karabiner-elements.pqrs.org/) (macOS keyboard customization) using Apple's [Pkl configuration language](https://pkl-lang.org/). It provides type-safe configuration with a rich standard library and live-reload daemon functionality.

## Architecture Overview

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   CLI Entry     │────▶│    Daemon    │────▶│   Compiler      │
│  (src/main.rs)  │     │ (src/daemon) │     │(src/compiler)   │
└─────────────────┘     └──────────────┘     └─────────────────┘
         │                      │                      │
         │                      │                      ▼
         │                      │              ┌──────────────┐
         │                      │              │  Pkl CLI     │
         │                      │              │ (external)   │
         │                      │              └──────────────┘
         │                      │                      │
         ▼                      ▼                      ▼
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│ Notifications   │     │File Watching │     │   Pkl Library   │
│(src/notifications)    │  (notify)    │     │  (pkl-lib/)     │
└─────────────────┘     └──────────────┘     └─────────────────┘
```

## Key Features

1. **Type-Safe Configuration**: All Karabiner concepts are strongly typed in Pkl
2. **Live Reload**: Daemon watches configuration files and auto-recompiles on changes
3. **Rich Helper Library**: High-level functions for common patterns (layers, vim navigation, app switching)
4. **Module System**: Import additional Pkl modules from local paths or URLs
5. **Error Recovery**: Daemon continues running even if compilation fails
6. **Native Notifications**: macOS notifications for compilation status
7. **Embedded Resources**: Pkl library files are embedded in the binary for easy distribution

## Project Structure

```
karabiner-pkl/
├── src/                    # Rust source code
│   ├── main.rs            # Entry point
│   ├── cli.rs             # Command-line interface
│   ├── compiler/          # Pkl compilation logic
│   ├── daemon/            # File watching daemon
│   ├── error.rs           # Error types and handling
│   ├── import/            # Module import functionality
│   ├── notifications/     # macOS notifications
│   ├── logging.rs         # Structured logging
│   └── embedded.rs        # Embedded Pkl resources
├── pkl-lib/               # Pkl type definitions
│   ├── karabiner.pkl      # Core types and SimpleConfig API
│   └── helpers.pkl        # Helper functions and constants
├── tests/                 # Test suite
│   ├── fixtures/          # Pkl test files
│   ├── helpers/           # Test utilities
│   └── integration/       # Integration tests
├── docs/                  # Documentation
├── scripts/               # Development scripts
└── Cargo.toml            # Rust project configuration
```

## Configuration Flow

1. User writes configuration in `~/.config/karabiner.pkl`
2. Karabiner-Pkl compiles Pkl to JSON using embedded type definitions
3. Generated JSON is written to `~/.config/karabiner/karabiner.json`
4. Karabiner-Elements automatically loads the new configuration
5. Daemon continues watching for changes (if running)

## Design Principles

- **Simplicity**: Single "Default" profile approach - no multi-profile complexity
- **Type Safety**: Leverage Pkl's type system to catch errors at compile time
- **Composability**: Build complex behaviors from simple, reusable functions
- **User Feedback**: Clear error messages with source locations and recovery suggestions
- **Reliability**: Graceful error handling ensures the daemon stays running

## Dependencies

### External Requirements
- **Pkl CLI**: Must be installed separately (`brew install pkl`)
- **Karabiner-Elements**: The actual keyboard customization software
- **macOS**: Currently only supports macOS due to Karabiner-Elements dependency

### Key Rust Dependencies
- **tokio**: Async runtime for daemon and file watching
- **clap**: Command-line argument parsing
- **miette**: Rich error diagnostics with source code display
- **notify**: Cross-platform file system notification
- **serde**: JSON serialization/deserialization
- **rust-embed**: Embed Pkl files in binary
- **notify-rust**: Native OS notifications

## Module System

Users can extend functionality by importing additional Pkl modules:

1. **Local Library**: `~/.config/karabiner_pkl/lib/`
2. **Import Command**: `karabiner-pkl add <source>` 
3. **Sources**: Local file paths or HTTP/HTTPS URLs
4. **Usage**: Import in Pkl files with `import "modulepath:/karabiner_pkl/lib/module.pkl"`

## Error Handling

The project uses `miette` for rich error diagnostics:

- Source code display with error location highlighting
- Helpful error messages with recovery suggestions
- Structured error types for different failure modes
- Graceful degradation (daemon continues on compilation errors)

## Logging

Structured logging with `tracing`:

- Console output with ANSI colors
- File logging to `~/.local/share/karabiner-pkl/logs/karabiner-pkl.log`
- Different log levels for different components
- Log viewing with `karabiner-pkl logs` command