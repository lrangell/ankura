# Development Guide

This guide covers development workflows, testing strategies, and contribution guidelines for Karabiner-Pkl.

## Development Setup

### Prerequisites

1. **Rust Toolchain**
   ```bash
   # Install via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Pkl CLI**
   ```bash
   # Install via Homebrew
   brew install pkl
   
   # Verify installation
   pkl --version
   ```

3. **Development Tools**
   ```bash
   # Install additional tools
   cargo install cargo-nextest  # Better test runner
   cargo install cargo-watch    # File watcher for development
   ```

### Repository Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/karabiner-pkl.git
cd karabiner-pkl

# Install git hooks (optional)
./scripts/install-hooks.sh

# Build the project
cargo build

# Run tests
cargo test
```

## Development Workflow

### Building

```bash
# Debug build (fast compilation, slower runtime)
cargo build

# Release build (optimized)
cargo build --release

# Install locally for testing
cargo install --path .
```

### Running During Development

```bash
# Run without installing
cargo run -- compile

# Run with debug logging
cargo run -- --debug compile

# Watch for changes and rebuild
cargo watch -x build
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with all targets
cargo clippy --all-targets --all-features

# Check for security vulnerabilities
cargo audit
```

## Testing

### Test Organization

```
tests/
├── fixtures/          # Pkl test files
├── helpers/          # Test utilities
│   └── mod.rs       # TestContext and helpers
└── integration/      # Integration tests
    ├── caps_lock_test.rs
    ├── simlayers_test.rs
    ├── shell_commands_test.rs
    └── ...
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_caps_lock

# Run tests in parallel with nextest
cargo nextest run

# Run tests matching pattern
cargo nextest run simlayer
```

### Writing Tests

#### Integration Test Pattern

```rust
use crate::helpers::TestContext;

#[test]
fn test_my_feature() {
    // Setup
    let ctx = TestContext::new();
    
    // Write test Pkl file
    let pkl_content = r#"
        import "modulepath:/karabiner.pkl"
        import "modulepath:/helpers.pkl"
        
        simpleConfig: karabiner.SimpleConfig = new {
            // Your test configuration
        }
        config: karabiner.Config = simpleConfig.toConfig()
    "#;
    
    let pkl_file = ctx.write_pkl_file("test.pkl", pkl_content);
    
    // Compile and verify
    let result = ctx.compile_pkl_to_json(&pkl_file)
        .expect("Failed to compile");
    
    // Assert JSON structure
    assert_eq!(
        result["config"]["profiles"][0]["complex_modifications"]["rules"][0]["description"],
        "Expected description"
    );
}
```

#### Using Fixtures

```rust
#[test]
fn test_with_fixture() {
    let ctx = TestContext::new();
    let (pkl_file, _content) = ctx.load_fixture("advanced_generators.pkl");
    
    let result = ctx.compile_pkl_to_json(&pkl_file)
        .expect("Failed to compile");
    
    // Verify results
}
```

### Test Helper Functions

The `TestContext` provides:

- `new()` - Create isolated test environment
- `write_pkl_file(name, content)` - Write test Pkl file
- `load_fixture(name)` - Load fixture from tests/fixtures/
- `compile_pkl_to_json(path)` - Compile and parse JSON
- Automatic cleanup via Drop trait

## Adding New Features

### 1. Adding a New CLI Command

1. Update `Commands` enum in `src/cli.rs`:
   ```rust
   pub enum Commands {
       // ... existing commands ...
       MyCommand {
           #[arg(short, long)]
           option: String,
       },
   }
   ```

2. Handle the command in `src/main.rs`:
   ```rust
   Commands::MyCommand { option } => {
       // Implementation
   }
   ```

3. Add tests in `tests/integration/`

### 2. Adding Pkl Helper Functions

1. Add function to `pkl/helpers.pkl`:
   ```pkl
   function myHelper(param: String) -> Rule = new Rule {
       description = "My helper function"
       manipulators = List(
           // Implementation
       )
   }
   ```

2. Add test fixture in `tests/fixtures/`
3. Add integration test

### 3. Adding Error Types

1. Add variant to `KarabinerPklError` in `src/error.rs`:
   ```rust
   #[error("My error: {message}")]
   #[diagnostic(
       code(karabiner_pkl::my_error),
       help("Helpful suggestion")
   )]
   MyError { message: String },
   ```

2. Use the error in your code:
   ```rust
   return Err(KarabinerPklError::MyError {
       message: "Something went wrong".to_string(),
   });
   ```

## Debugging

### Debug Logging

```rust
use tracing::{debug, info, warn, error};

// Add debug logs
debug!("Processing file: {:?}", path);
info!("Compilation started");
warn!("Deprecated feature used");
error!("Failed to compile: {}", err);
```

Run with debug logging:
```bash
cargo run -- --debug compile
```

### Common Issues

1. **Pkl CLI Not Found**
   - Ensure `pkl` is in PATH
   - Install with `brew install pkl`

2. **Module Import Errors**
   - Check module paths in Pkl files
   - Verify pkl files are embedded correctly

3. **Permission Errors**
   - Ensure write access to `~/.config/karabiner/`
   - Check log directory permissions

## Performance Considerations

### Compilation Performance

- Pkl compilation is the bottleneck
- Cache results when possible
- Use debouncing for file watching (300ms default)

### Binary Size

- Release builds are ~10MB
- Embedded Pkl files add minimal overhead
- Use `cargo bloat` to analyze size

### Memory Usage

- Compiler creates temporary directories
- Clean up resources with Drop trait
- Use Arc for shared ownership in async code

## Release Process

### Version Bumping

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Commit with message: `chore: bump version to X.Y.Z`

### Building Releases

```bash
# Build for current platform
cargo build --release

# Strip debug symbols (macOS)
strip target/release/karabiner-pkl

# Verify binary
./target/release/karabiner-pkl --version
```

### Cross-Platform Builds

```bash
# Install cross-compilation tools
cargo install cross

# Build for different targets
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin
```

## Best Practices

### Code Style

1. **Error Handling**
   - Use `Result<T, KarabinerPklError>` for fallible operations
   - Provide context with error messages
   - Use `?` operator for propagation

2. **Async Code**
   - Use `tokio` for async runtime
   - Avoid blocking operations in async contexts
   - Use `Arc` for shared state

3. **Testing**
   - Write integration tests for new features
   - Use fixtures for complex test cases
   - Test error paths, not just happy paths

4. **Documentation**
   - Document public APIs with rustdoc
   - Update user documentation for new features
   - Include examples in doc comments

### Git Workflow

1. **Branch Naming**
   - `feature/description` for new features
   - `fix/description` for bug fixes
   - `docs/description` for documentation

2. **Commit Messages**
   - Use conventional commits format
   - Include context in commit body
   - Reference issues when applicable

3. **Pull Requests**
   - Include tests for new features
   - Update documentation
   - Ensure CI passes

## Troubleshooting Development Issues

### Rust Toolchain Issues

```bash
# Update Rust
rustup update

# Clean build artifacts
cargo clean

# Rebuild from scratch
cargo build
```

### Test Failures

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Check test logs
cat ~/.local/share/karabiner-pkl/logs/karabiner-pkl.log
```

### IDE Setup

#### VS Code

Install extensions:
- rust-analyzer
- CodeLLDB (for debugging)

Settings:
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

#### IntelliJ IDEA

Install Rust plugin and configure:
- Enable "Expand declarative macros"
- Set up cargo commands as run configurations