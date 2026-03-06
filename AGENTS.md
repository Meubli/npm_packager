# Agent Guidelines for npm_packager

This document provides agentic coding agents with guidelines for working in the `npm_packager` codebase.

## Build, Test, and Lint Commands

### Build
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Running the application
```bash
# Run with default arguments
cargo run

# Run with custom arguments
cargo run -- --package-lock <path> --concurrent <num> --max-retries <num>

# Example: 200 concurrent downloads, 5 retries
cargo run -- --concurrent 200 --max-retries 5
```

### Testing
```bash
# Run all tests
cargo test

# Run single test (replace TEST_NAME with actual test name)
cargo test TEST_NAME -- --nocapture

# Run tests without capturing output
cargo test -- --nocapture

# Run tests with release optimizations
cargo test --release
```

### Linting and Formatting
```bash
# Check code with clippy
cargo clippy

# Check with all clippy features
cargo clippy -- -W clippy::all

# Format code (dry-run, shows what would change)
cargo fmt -- --check

# Format code (apply changes)
cargo fmt

# Check specific file formatting
cargo fmt -- --check src/main.rs
```

## Code Style Guidelines

### Imports
- Use `use` statements at the top of modules, not inline
- Group imports: standard library, external crates, internal modules
- Sort imports alphabetically within groups
- Example:
  ```rust
  use std::fs;
  use std::time::Duration;
  
  use serde::Deserialize;
  use tokio;
  
  use crate::download::Package;
  ```

### Formatting and Spacing
- Max line length: ~100 characters (follow rustfmt defaults)
- Use 4 spaces for indentation
- Add blank lines between logical sections
- Use consistent spacing around operators

### Types and Declarations
- Always specify return types for public functions
- Use `Result<T, E>` for fallible operations
- Prefer specific error types over generic `String` errors
- Use type aliases for complex types:
  ```rust
  type DynError = Box<dyn std::error::Error>;
  ```

### Naming Conventions
- **Functions**: `snake_case` (e.g., `download_package`, `verify_integrity`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- **Types/Structs**: `PascalCase` (e.g., `Package`, `PackageInfo`)
- **Variables**: `snake_case` (e.g., `concurrent_downloads`)
- **Module files**: `snake_case` (e.g., `download.rs`, `system.rs`)

### Error Handling
- Use custom enum types for domain-specific errors instead of plain `String`
- Example (from codebase):
  ```rust
  enum DownloadError {
      TryError(String),
      IntegrityError(String),
  }
  ```
- Use `?` operator to propagate errors in functions returning `Result`
- Use `.map_err()` to convert between error types when needed
- Provide context in error messages with file/URL information

### Documentation
- Add doc comments (`///`) for public functions and public types
- Include examples in doc comments where helpful
- Use `//` for inline comments explaining complex logic
- Example:
  ```rust
  /// Verifies file integrity by comparing SHA512 hash
  /// Format: "sha512-{hash_en_base64}"
  fn verify_integrity(bytes: &[u8], integrity_string: &str) -> Result<(), DownloadError>
  ```

### Async/Concurrency Patterns
- Use `tokio` for async runtime management
- Use `futures::stream::StreamExt` for stream operations
- Use `.buffered()` for managing concurrent operations:
  ```rust
  stream::iter(packages)
      .map(|pkg| async { /* download */ })
      .buffered(concurrent_limit)
  ```
- Wrap shared mutable state in `Arc<Mutex<T>>` for concurrent access

### Module Organization
- Keep modules focused on single responsibilities
- `main.rs`: Application entry point, CLI parsing, orchestration
- `download.rs`: Package download logic and integrity verification
- `system.rs`: File system operations (zipping, directory management)
- Use module declaration: `mod download;` in `main.rs`

### Struct and Impl Patterns
- Implement `new()` constructor for structs with parameters
- Derive common traits: `Debug`, `Clone`, `Serialize`, `Deserialize`
- Use `#[derive(...)]` attribute macro for auto-implementations:
  ```rust
  #[derive(Debug, Clone)]
  pub struct Package { ... }
  ```

## Project Structure

```
npm_packager/
├── src/
│   ├── main.rs       # CLI args, package parsing, orchestration
│   ├── download.rs   # Download and integrity verification logic
│   └── system.rs     # File system utilities (zip, directories)
├── Cargo.toml        # Dependencies and project metadata
└── AGENTS.md         # This file
```

## Key Dependencies
- **tokio**: Async runtime
- **serde/serde_json**: JSON serialization
- **reqwest**: HTTP requests
- **clap**: CLI argument parsing
- **sha2**: SHA512 hashing
- **base64**: Base64 encoding
- **zip**: ZIP file creation
- **walkdir**: Directory traversal
- **indicatif**: Progress bars

## Common Patterns in Codebase

### Retry Logic with Exponential Backoff
```rust
let mut delay = Duration::from_millis(500);
loop {
    match attempt().await {
        Ok(result) => return Ok(result),
        Err(e) if retry < max => {
            delay = Duration::from_millis(delay.as_millis() as u64 * 2);
            sleep(delay).await;
        }
    }
}
```

### Stream-Based Concurrency
```rust
stream::iter(items)
    .map(|item| async { process(item).await })
    .buffered(concurrent_limit)
    .collect::<()>()
    .await
```

### Error Conversion
```rust
map_err(|e| DownloadError::TryError(e.to_string()))
```
