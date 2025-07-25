# Testing Guide

This guide covers testing strategies, patterns, and best practices for Karabiner-Pkl.

## Test Structure

### Directory Layout

```
tests/
├── lib.rs              # Test module root
├── helpers/            # Test utilities and common code
│   └── mod.rs         # TestContext and helper functions
├── integration/        # Integration tests
│   ├── caps_lock_test.rs
│   ├── simlayers_test.rs
│   ├── shell_commands_test.rs
│   ├── shift_layers_test.rs
│   ├── advanced_pkl_test.rs
│   ├── import_test.rs
│   ├── generators_test.rs
│   └── space_mode_test.rs
└── fixtures/           # Pkl test files
    ├── advanced_generators.pkl
    ├── advanced_map_filter.pkl
    ├── caps_lock_complex.pkl
    ├── caps_lock_simple.pkl
    ├── shell_commands.pkl
    ├── shift_layers.pkl
    ├── simlayers.pkl
    └── space_mode.pkl
```

## Test Helper: TestContext

The `TestContext` struct provides a clean abstraction for test setup and execution.

### Key Features

```rust
pub struct TestContext {
    temp_dir: TempDir,
    compiler: Compiler,
}
```

### Available Methods

#### `new()`
Creates a new test context with isolated temporary directory and compiler instance.

```rust
let ctx = TestContext::new();
```

#### `write_pkl_file(name, content)`
Writes a Pkl file to the temp directory.

```rust
let pkl_file = ctx.write_pkl_file("test.pkl", r#"
    import "modulepath:/karabiner.pkl"
    // ... configuration
"#);
```

#### `load_fixture(name)`
Loads a fixture file from tests/fixtures/.

```rust
let content = TestContext::load_fixture("caps_lock_test.pkl");
```

#### `compile_pkl(&self, pkl_file, profile_name)`
Asynchronously compiles Pkl to JSON using the Compiler API.

```rust
let result = ctx.compile_pkl(&pkl_file, None).await?;
assert_eq!(result["profiles"][0]["name"], "pkl");
```

#### `compile_pkl_sync(&self, pkl_file, profile_name)`
Synchronous wrapper for tests that don't use async.

```rust
let result = ctx.compile_pkl_sync(&pkl_file, None)?;
assert_eq!(result["profiles"][0]["name"], "pkl");
```

## Writing Tests

### Basic Test Pattern

```rust
#[test]
fn test_basic_functionality() {
    // 1. Setup test context
    let ctx = TestContext::new();
    
    // 2. Define Pkl configuration
    let pkl_content = r#"
        import "modulepath:/karabiner.pkl"
        import "modulepath:/helpers.pkl"
        
        simpleConfig: karabiner.SimpleConfig = new {
            complex_modifications = new karabiner.ComplexModifications {
                rules = List(
                    helpers.capsLockToEscape()
                )
            }
        }
        config: karabiner.Config = simpleConfig.toConfig()
    "#;
    
    // 3. Write and compile
    let pkl_file = ctx.write_pkl_file("test.pkl", pkl_content);
    let result = ctx.compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");
    
    // 4. Verify JSON output (note: no ["config"] prefix anymore)
    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(rules[0]["description"], "Caps Lock to Escape");
}
```

### Testing with Fixtures

```rust
#[test]
fn test_complex_configuration() {
    let ctx = TestContext::new();
    let fixture_content = TestContext::load_fixture("caps_lock_test.pkl");
    let pkl_file = ctx.write_pkl_file("fixture_test.pkl", &fixture_content);
    
    let result = ctx.compile_pkl_sync(&pkl_file, None)
        .expect("Failed to compile");
    
    // Verify the generated configuration
    let rules = &result["profiles"][0]["complex_modifications"]["rules"];
    assert_eq!(rules.as_array().unwrap().len(), 1);
}
```

### Testing Error Cases

```rust
#[test]
fn test_invalid_pkl_syntax() {
    let ctx = TestContext::new();
    let pkl_content = r#"
        invalid syntax here {
    "#;
    
    let pkl_file = ctx.write_pkl_file("invalid.pkl", pkl_content);
    let result = ctx.compile_pkl_sync(&pkl_file, None);
    
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Compilation failed"));
    }
}
```

## Test Categories

### 1. Caps Lock Tests (`caps_lock_test.rs`)

Tests various Caps Lock remapping scenarios:
- Simple Escape mapping
- Control when held, Escape when tapped
- Custom modifier mappings

### 2. Simlayer Tests (`simlayers_test.rs`)

Tests simultaneous key layers:
- Basic simlayer functionality
- Vim-style navigation
- Custom threshold values

### 3. Shell Command Tests (`shell_commands_test.rs`)

Tests shell command execution:
- Single commands
- Commands with arguments
- Yabai window manager integration

### 4. Shift Layer Tests (`shift_layers_test.rs`)

Tests shift-based layers:
- Semicolon as shift
- Letter keys as shift
- Multiple shift layers

### 5. Advanced Pkl Tests (`advanced_pkl_test.rs`)

Tests advanced Pkl language features:
- Anonymous functions
- Map/filter operations
- Let expressions
- Custom classes
- Functional programming patterns

### 6. Import Tests (`import_test.rs`)

Tests module import functionality:
- Local file imports  
- Module path compilation
- Embedded library imports

### 7. Generator Tests (`generators_test.rs`)

Tests code generation helpers:
- Character ranges
- Number ranges
- QWERTY sequences
- Vim home row navigation

### 8. Space Mode Tests (`space_mode_test.rs`)

Tests space-based modal configurations:
- Space as layer trigger
- Preserving space functionality
- Custom threshold values

### 9. Profile Preservation Tests (`profile_preservation_test.rs`)

Tests profile management:
- Preserving existing profiles
- Creating new profiles
- Profile name overrides
- Merging configurations

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test file
cargo test caps_lock

# Run specific test function
cargo test test_caps_lock_to_escape
```

### Using Nextest

```bash
# Install nextest
cargo install cargo-nextest

# Run all tests (parallel)
cargo nextest run

# Run with specific pattern
cargo nextest run simlayer

# Run with test output
cargo nextest run --no-capture
```

### Debugging Tests

```bash
# Run single test with full output
RUST_LOG=debug cargo test test_name -- --nocapture

# Check test artifacts
ls -la /tmp/karabiner_pkl_test_*

# View test logs
cat ~/.local/share/karabiner-pkl/logs/karabiner-pkl.log
```

## Best Practices

### 1. Test Organization

- Group related tests in the same file
- Use descriptive test names
- Keep tests focused on single functionality

### 2. Test Data

- Use inline Pkl content for simple tests
- Use fixtures for complex configurations
- Keep fixtures minimal and focused
- All imports should use `modulepath:/`

### 3. Assertions

```rust
// Use specific assertions
assert_eq!(actual, expected, "Custom message");

// Check JSON structure systematically
let config = &result["config"];
let profile = &config["profiles"][0];
let rules = &profile["complex_modifications"]["rules"];

// Use pretty_assertions for better output
use pretty_assertions::assert_eq;
```

### 4. Error Testing

Always test both success and failure cases:

```rust
// Success case
let result = ctx.compile_pkl_sync(&valid_pkl, None).unwrap();

// Failure case
let result = ctx.compile_pkl_sync(&invalid_pkl, None);
assert!(result.is_err());
```

### 5. Test Independence

Each test should:
- Create its own TestContext
- Not depend on other tests
- Clean up automatically (via Drop)

## Common Test Patterns

### Testing Helper Functions

```rust
#[test]
fn test_vim_navigation_helper() {
    let ctx = TestContext::new();
    let pkl_content = r#"
        import "modulepath:/karabiner.pkl"
        import "modulepath:/helpers.pkl"
        
        simpleConfig: karabiner.SimpleConfig = new {
            complex_modifications = new karabiner.ComplexModifications {
                rules = List(helpers.vimNavigation())
            }
        }
        config: karabiner.Config = simpleConfig.toConfig()
    "#;
    
    let pkl_file = ctx.write_pkl_file("vim_nav.pkl", pkl_content);
    let result = ctx.compile_pkl_sync(&pkl_file, None).unwrap();
    
    // Verify vim bindings exist
    let manipulators = &result["profiles"][0]
        ["complex_modifications"]["rules"][0]["manipulators"];
    
    // Check h → left_arrow mapping exists
    let h_mapping = manipulators.as_array().unwrap()
        .iter()
        .find(|m| m["from"]["key_code"] == "h")
        .expect("h mapping not found");
    
    assert_eq!(h_mapping["to"][0]["key_code"], "left_arrow");
}
```

### Testing Complex Configurations

```rust
#[test]
fn test_multiple_layers() {
    let ctx = TestContext::new();
    let pkl_content = r#"
        // Multiple layers configuration
        rules = List(
            helpers.layer("left_control", mappings1),
            helpers.simlayer("d", mappings2),
            helpers.spaceMode(mappings3)
        )
    "#;
    
    // Test that all layers coexist properly
}
```

### Testing Edge Cases

```rust
#[test]
fn test_empty_configuration() {
    // Test minimal valid configuration
}

#[test]
fn test_maximum_rules() {
    // Test with many rules
}

#[test]
fn test_special_characters_in_commands() {
    // Test shell commands with quotes, spaces, etc.
}
```

## Performance Testing

While not automated, consider:

1. **Compilation Speed**
   ```rust
   use std::time::Instant;
   
   let start = Instant::now();
   let _ = ctx.compile_pkl_sync(&pkl_file, None)?;
   let duration = start.elapsed();
   
   println!("Compilation took: {:?}", duration);
   ```

2. **Memory Usage**
   - Use tools like `valgrind` or `heaptrack`
   - Monitor temp directory cleanup

3. **Large Configurations**
   - Test with many rules
   - Test with deep nesting
   - Test with many imports

## Test Architecture

### Key Principles

1. **No File I/O in Tests**: Tests use the Compiler API directly instead of running CLI commands
2. **No System Modifications**: Tests don't write to actual Karabiner config files
3. **No OS Notifications**: Tests don't emit system notifications
4. **Direct API Usage**: Tests call `compiler.compile()` which returns JSON

### Separation of Concerns

The test architecture reflects the module separation:

- **Compiler Tests**: Test pure Pkl-to-JSON transformation
- **CLI Tests**: Test file operations and profile merging
- **Integration Tests**: Test the full flow using public APIs

## Continuous Integration

Tests run automatically on:
- Every push
- Every pull request
- Multiple OS versions (macOS)

Ensure all tests pass before merging!