# Contributing to npm_packager

Thank you for your interest in contributing! This document provides guidelines and instructions.

## Code of Conduct

Be respectful and constructive. Treat all contributors with courtesy.

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git

### Local Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/npm_packager.git
cd npm_packager

# Build the project
cargo build

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or for bugfixes:
git checkout -b fix/your-bug-name
```

### 2. Make Changes

- Write clear, idiomatic Rust code
- Follow existing code style
- Add tests for new features
- Update documentation as needed

### 3. Code Quality Checks

Before committing, ensure:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Build everything
cargo build --release
```

All checks must pass with zero warnings.

### 4. Commit with Clear Messages

Follow conventional commits format:

```bash
git commit -m "feat: add new feature description"
git commit -m "fix: resolve issue with downloads"
git commit -m "docs: update README"
git commit -m "refactor: improve error handling"
git commit -m "test: add tests for download retry"
```

Prefix types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `refactor:` - Code reorganization (no functionality change)
- `test:` - Test additions or changes
- `perf:` - Performance improvements
- `chore:` - Maintenance

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then open a Pull Request on GitHub with:
- Clear title describing the change
- Description of what changed and why
- Reference to any related issues
- Any breaking changes highlighted

## Style Guide

### Rust Code Style

- Use `cargo fmt` for formatting (non-negotiable)
- Run `cargo clippy` and fix all warnings
- Zero clippy warnings required for merge
- Max line length: ~100 characters (rustfmt default)

### Naming Conventions

- **Functions**: `snake_case` (e.g., `download_package`)
- **Types/Structs**: `PascalCase` (e.g., `PackageInfo`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- **Variables**: `snake_case` (e.g., `output_dir`)

### Module Organization

Keep modules focused on single responsibility:
- `error.rs` → Error types
- `config.rs` → Configuration
- `packager.rs` → Orchestration
- `download.rs` → Download logic
- `system.rs` → Filesystem operations

### Documentation

Add doc comments for public functions:

```rust
/// Downloads a single package with retry logic.
///
/// # Arguments
/// * `package` - Package to download
/// * `output_dir` - Directory to save the package
/// * `max_retry` - Maximum retry attempts
///
/// # Returns
/// Ok if successful, Err with PackagerError on failure
pub async fn download_package_with_retry(
    package: &Package,
    output_dir: &Path,
    max_retry: u16,
) -> PackagerResult<()>
```

### Error Handling

- Use custom `PackagerError` types
- Provide context in error messages
- Avoid generic `String` errors
- Use `Result<T, PackagerError>` for fallible operations

## Testing

### Writing Tests

Add tests for new features:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_feature() {
        // For async code
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Show output
cargo test -- --nocapture

# Run with debug output
RUST_LOG=debug cargo test -- --nocapture
```

All tests must pass before submitting a PR.

## Issues and Discussions

### Reporting Bugs

Create an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, Rust version, etc.)

### Suggesting Features

Open an issue with:
- Clear description of the feature
- Use cases and motivation
- Potential implementation approach

## Review Process

1. Automated checks run (CI/CD)
2. Code review by maintainers
3. Address feedback and update PR
4. Merge when approved

Merging requires:
- All checks passing
- At least one approval
- Zero warnings from clippy

## Release Process

Version numbering follows [SemVer](https://semver.org/):
- MAJOR: Breaking API changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

## Questions?

Feel free to open an issue for:
- Documentation clarifications
- Architecture questions
- Implementation approach discussions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing! 🎉
