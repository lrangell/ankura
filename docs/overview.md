# Karabiner-Pkl Overview

Karabiner-Pkl is a configuration tool for [Karabiner-Elements](https://karabiner-elements.pqrs.org/) (macOS keyboard customization) using Apple's [Pkl configuration language](https://pkl-lang.org/). It provides type-safe configuration with a modern factory functions API, rich standard library, and live-reload daemon functionality.

## Quick Start

```pkl
import "modulepath:/karabiner.pkl" as k
import "modulepath:/helpers.pkl" as h
import "modulepath:/keys.pkl"

// Simple, readable configuration
config = k.config("MyProfile", List(
  // Basic key remapping  
  k.rule("Caps to Escape", k.map(keys.caps_lock, keys.escape)),
  
  // Dual-use keys
  k.rule("Space dual-use", k.dualUse(keys.spacebar, keys.left_option, keys.spacebar)),
  
  // Navigation layer
  k.layer(keys.left_option, k.mapping(new Mapping {
    ["h"] = keys.left_arrow
    ["j"] = keys.down_arrow
    ["k"] = keys.up_arrow
    ["l"] = keys.right_arrow
  })),
  
  // Helper shortcuts
  h.capsToCtrlEsc(),
  h.spaceToOpt(),
  h.vimNavigation()
))
```

## Architecture Overview

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Main Entry    │────▶│     CLI      │────▶│    Compiler     │
│  (src/main.rs)  │     │ (src/cli.rs) │     │(src/compiler)   │
└─────────────────┘     └──────────────┘     └─────────────────┘
                               │                      │
                               │                      ▼
                               │              ┌──────────────┐
                               │              │  Pkl CLI     │
                               │              │ (external)   │
                               │              └──────────────┘
                               │                      │
                               ▼                      ▼
                        ┌──────────────┐     ┌─────────────────┐
                        │    Daemon    │     │ Embedded Pkl    │
                        │ (src/daemon) │     │   Library       │
                        └──────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌──────────────┐
                        │ Notifications│
                        │  (in daemon) │
                        └──────────────┘
```

### Key Architectural Changes

1. **Separation of Concerns**:
   - `main.rs`: Simple initialization and command dispatch only
   - `cli.rs`: All CLI command implementations and file I/O operations
   - `compiler/`: Pure compilation logic - takes Pkl files, returns JSON
   - `daemon/`: File watching and notifications (merged)

2. **No File I/O in Compiler**: The compiler module only handles Pkl-to-JSON transformation
3. **Embedded Resources**: Pkl library files are embedded directly in the compiler module
4. **Consolidated Modules**: Notifications are now part of the daemon module

## Key Features

1. **Factory Functions API**: Clean, readable syntax eliminating verbose `new` constructions
2. **Type-Safe Configuration**: All Karabiner concepts are strongly typed in Pkl
3. **Keys Module**: Single source of truth for key constants with helpful aliases
4. **Live Reload**: Daemon watches configuration files and auto-recompiles on changes
5. **Rich Helper Library**: High-level functions for common patterns (layers, vim navigation, app switching)
6. **Module System**: Import additional Pkl modules from local paths or URLs
7. **Error Recovery**: Daemon continues running even if compilation fails

### Factory Functions API

The new factory functions API provides clean, readable syntax:

- **`k.map(from, to)`** - Basic key remapping
- **`k.withMods(modifiers, from, to)`** - Keys with modifiers
- **`k.dualUse(key, hold, tap)`** - Dual-use behavior
- **`k.simul(keys, to)`** - Simultaneous key combinations
- **`k.layer(modifier, mappings)`** - Modifier layers
- **`k.simlayer(trigger, mappings)`** - Simultaneous layers
- **`k.rule(description, manipulator)`** - Rule creation
- **`k.config(profileName, rules)`** - Configuration creation

See the [Factory Functions Guide](factory-functions-guide.md) for complete documentation.
6. **Native Notifications**: macOS notifications for compilation status
7. **Embedded Resources**: Pkl library files are embedded in the binary for easy distribution

## Project Structure

```
karabiner-pkl/
├── src/                    # Rust source code
│   ├── main.rs            # Simple entry point (init & dispatch)
│   ├── cli.rs             # CLI commands and file I/O operations
│   ├── compiler/          # Pure Pkl compilation logic
│   │   └── mod.rs         # Compiler + embedded resources
│   ├── daemon/            # File watching + notifications
│   │   └── mod.rs         # Daemon + NotificationManager
│   ├── error.rs           # Error types and handling
│   ├── import/            # Module import functionality
│   ├── logging.rs         # Structured logging
│   └── lib.rs             # Library exports
├── pkl/                   # Pkl type definitions
│   ├── karabiner.pkl      # Core types and SimpleConfig API
│   └── helpers.pkl        # Helper functions and constants
├── tests/                 # Test suite
│   ├── fixtures/          # Pkl test files
│   ├── helpers/           # Test utilities
│   └── integration/       # Integration tests
├── docs/                  # Documentation
└── Cargo.toml            # Rust project configuration
```

## Configuration Flow

1. User writes configuration in `~/.config/karabiner.pkl`
2. Karabiner-Pkl compiles Pkl to JSON using embedded type definitions
3. Generated JSON is written to `~/.config/karabiner/karabiner.json`
4. Karabiner-Elements automatically loads the new configuration
5. Daemon continues watching for changes (if running)

## Design Principles

- **Simplicity**: Default "pkl" profile approach with customizable profile names
- **Type Safety**: Leverage Pkl's type system to catch errors at compile time
- **Composability**: Build complex behaviors from simple, reusable functions
- **User Feedback**: Clear error messages with source locations and recovery suggestions
- **Reliability**: Graceful error handling ensures the daemon stays running
- **Separation of Concerns**: Compiler returns JSON, CLI handles files, daemon manages watching

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
4. **Usage**: Import in Pkl files with `import "modulepath:/module.pkl"`

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