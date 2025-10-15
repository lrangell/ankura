# Agent Guidelines for Ankura

## Build/Test Commands
- **Build**: `cargo build` / `cargo build --release`
- **Format/Lint**: `cargo fmt` / `cargo clippy -- -D warnings`
- **Run single Rust test**: `cargo nextest run <test_name>` (uses nextest config)
- **Run all Rust tests**: `cargo nextest run`
- **Run Pkl tests**: `cd pkl-tests && pkl test`
- **Run CLI**: `cargo run -- <args>`

## Code Style & Conventions
- **Rust 2021** with 4-space indentation
- **Naming**: snake_case for functions/vars, CamelCase for types, snake_case for modules
- **Imports**: Group std, external crates, then local modules; use explicit imports
- **Error handling**: Use `KarabinerPklError` with `Result<T, KarabinerPklError>`; propagate with `?`
- **Logging**: Use `tracing` macros (`info!`, `warn!`, `error!`); prefer structured logging
- **Paths**: Use `dirs` crate for user paths; avoid hardcoded paths
- **Types**: Use strong typing; derive common traits (`Debug`, `Clone`, etc.) appropriately

## Testing Guidelines
- **Rust tests**: Integration-style in CLI/compiler layers; use `#[test]` and test modules
- **Pkl tests**: Place in `pkl-tests/*Test.pkl`; follow `*Test.pkl` pattern
- **Test data**: Use fixtures in `pkl-tests/` for Pkl tests
- **CI**: Tests run with nextest; ensure both Rust and Pkl tests pass

## Commit Standards
- Use conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, etc.
- Reference `.gitmessage` template for commit format
- Run `cargo fmt` and `cargo clippy` before committing
- Ensure Pkl tests pass for changes to `pkl/` directory

## Project Structure
- **Binary**: `src/main.rs` (CLI entry)
- **Library**: `src/lib.rs` (core logic)
- **Modules**: `cli.rs`, `compiler/`, `daemon/`, `import/`, `logging.rs`, `error.rs`
- **Pkl sources**: `pkl/` (embedded at build time)
- **Tests**: `pkl-tests/` for Pkl, inline `#[test]` for Rust

## Security Notes
- No secrets in code; use config/environment discovery
- Requires Pkl CLI in PATH (`brew install pkl`)
- Avoid logging sensitive data
- Use secure defaults for file permissions
