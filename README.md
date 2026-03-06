# npm_packager

A high-performance CLI tool for downloading and packaging npm dependencies in parallel with SHA512 integrity verification.

## Features

- ⚡ **Parallel Downloads**: Configurable concurrent downloads (default 100)
- 🔒 **Integrity Verification**: SHA512 checksum validation for all packages
- 🔄 **Automatic Retry**: Exponential backoff retry strategy (default 4 retries)
- 📦 **Auto Compression**: Automatic ZIP compression of downloaded packages
- 📊 **Progress Tracking**: Real-time progress bars and spinners
- 🏗️ **Well-Architected**: Clean separation of concerns with proper error handling
- 📋 **Structured Logging**: Production-ready logging with `tracing`

## Installation

### Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/Meubli/npm_packager/main/install.sh | bash
```

**Manual Download:**
Download the pre-built binary for your platform from [Releases](https://github.com/Meubli/npm_packager/releases):

- **Linux x86_64 (GNU)**: `npm_packager-x86_64-unknown-linux-gnu.tar.gz`
- **Linux x86_64 (musl/Alpine)**: `npm_packager-x86_64-unknown-linux-musl.tar.gz`
- **Linux ARM64 (GNU)**: `npm_packager-aarch64-unknown-linux-gnu.tar.gz`
- **Linux ARM64 (musl)**: `npm_packager-aarch64-unknown-linux-musl.tar.gz`
- **macOS Intel**: `npm_packager-x86_64-apple-darwin.tar.gz`
- **macOS Apple Silicon (M1/M2)**: `npm_packager-aarch64-apple-darwin.tar.gz`
- **Windows x86_64**: `npm_packager-x86_64-pc-windows-msvc.zip`

Extract and run:
```bash
# Linux/macOS
tar xzf npm_packager-x86_64-unknown-linux-gnu.tar.gz
./npm_packager-x86_64-unknown-linux-gnu/npm_packager --help

# Windows
# Unzip and run npm_packager.exe
```

### From Source

```bash
git clone https://github.com/Meubli/npm_packager.git
cd npm_packager
cargo build --release
./target/release/npm_packager --help
```

## Usage

### Basic Usage

```bash
# Download packages from package-lock.json
npm_packager --package-lock package-lock.json
```

### Advanced Usage

```bash
# Custom concurrency (200 parallel downloads)
npm_packager --concurrent 200 --package-lock package-lock.json

# More retries for unreliable networks
npm_packager --max-retries 8 --package-lock package-lock.json

# Custom output directory
npm_packager --output-dir ./my_packages --package-lock package-lock.json

# Combine options
npm_packager \
  --package-lock package-lock.json \
  --concurrent 150 \
  --max-retries 6 \
  --output-dir ./offline_packages
```

### Command Line Options

```
Options:
  -p, --package-lock <PACKAGE_LOCK>
      Path to package-lock.json [default: package-lock.json]

  -c, --concurrent <CONCURRENT>
      Number of concurrent downloads [default: 100]

  -m, --max-retries <MAX_RETRIES>
      Maximum retry attempts per package [default: 4]

  -o, --output-dir <OUTPUT_DIR>
      Output directory (optional, uses timestamped dir by default)

  -h, --help
      Print help information

  -V, --version
      Print version
```

## Logging

Control verbosity with `RUST_LOG`:

```bash
# Normal (no internal logs)
npm_packager

# Debug mode (all details)
RUST_LOG=debug npm_packager

# Production (errors only)
RUST_LOG=error npm_packager

# Specific module
RUST_LOG=npm_packager::download=debug npm_packager
```

See [LOGGING.md](LOGGING.md) for detailed logging guide.

## Architecture

The project follows clean architecture principles with well-separated modules:

- **`main.rs`** - CLI entry point
- **`config.rs`** - Configuration and argument parsing
- **`packager.rs`** - Main orchestration logic
- **`download.rs`** - Package downloading with retry logic
- **`system.rs`** - Filesystem operations
- **`error.rs`** - Unified error handling

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Development

### Building

```bash
cargo build              # Debug build
cargo build --release   # Optimized release build
```

### Testing

```bash
cargo test              # Run all tests
cargo test -- --nocapture   # Show output
cargo test --release    # Test with optimizations
```

### Code Quality

```bash
cargo clippy            # Lint checks
cargo fmt               # Format code
cargo fmt --check       # Check formatting
```

## Performance Tips

- Increase `--concurrent` for better parallelism (e.g., 200-300 on modern systems)
- Use `--max-retries 1` for reliable networks to fail fast
- Run in release mode for ~10x faster execution: `cargo build --release`

## Troubleshooting

### Timeout Issues

If downloads timeout frequently:

```bash
RUST_LOG=debug npm_packager  # See timeout details
npm_packager --max-retries 8  # Increase retries
```

### Integrity Check Failures

If packages fail integrity checks, the file is likely corrupted during download:

```bash
npm_packager  # Try again, it will retry automatically
```

If it persists, check the failed_packages.txt in the output directory.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture and design decisions
- [LOGGING.md](LOGGING.md) - Complete logging guide and usage examples
- [TRACING_EXPLAINED.md](TRACING_EXPLAINED.md) - Why tracing is useful for CLI tools
- [IMPROVEMENTS.md](IMPROVEMENTS.md) - Summary of architectural improvements
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contributing guidelines

## License

Licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Author

Created with ❤️ for better npm package management
