# Creating Releases

This document explains how to create and publish releases for npm_packager.

## Quick Start

Creating a release is as simple as:

```bash
# Create a git tag
git tag v0.1.0

# Push the tag to GitHub
git push origin v0.1.0
```

That's it! GitHub Actions will automatically:
1. Compile for all 7 platforms
2. Create archives
3. Create a GitHub Release
4. Upload all binaries

## How It Works

### GitHub Actions Workflow

The file `.github/workflows/release.yml` defines the release process:

1. **Trigger**: When you push a tag matching `v*` pattern (e.g., `v0.1.0`, `v1.2.3`)
2. **Create Release**: Create an empty GitHub Release
3. **Build Matrix**: Compile in parallel for all platforms:
   - x86_64-unknown-linux-gnu
   - x86_64-unknown-linux-musl
   - aarch64-unknown-linux-gnu
   - aarch64-unknown-linux-musl
   - x86_64-apple-darwin
   - aarch64-apple-darwin
   - x86_64-pc-windows-msvc
4. **Archive**: Create compressed archives (.tar.gz for Unix, .zip for Windows)
5. **Upload**: Upload all artifacts to the GitHub Release

Total time: ~10-15 minutes

## Monitoring the Build

After pushing a tag:

1. Go to: https://github.com/Meubli/npm_packager/actions
2. You'll see the workflow running
3. Monitor the progress in real-time
4. Once complete, check https://github.com/Meubli/npm_packager/releases

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., `v1.2.3`)
- Increment MAJOR for breaking changes
- Increment MINOR for new features (backward compatible)
- Increment PATCH for bug fixes

Examples:
```bash
git tag v0.1.0    # Initial release
git tag v0.2.0    # New feature
git tag v0.2.1    # Bug fix
git tag v1.0.0    # Breaking changes (major release)
```

## Release Checklist

Before releasing:

- [ ] Code is committed and pushed
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code is formatted: `cargo fmt`
- [ ] Update CHANGELOG.md (optional but recommended)
- [ ] Decide on version number following SemVer

## Installation Scripts

Users can install via:

**Automatic (Linux/macOS):**
```bash
curl -sSL https://raw.githubusercontent.com/Meubli/npm_packager/main/install.sh | bash
```

**Manual:**
- Download from https://github.com/Meubli/npm_packager/releases
- Choose the right binary for their platform
- Extract and run

## Platform Selection Guide

### Which binary to choose?

**Linux x86_64 (GNU libc)** - Most common
```
npm_packager-x86_64-unknown-linux-gnu.tar.gz
```
- For: Debian, Ubuntu, Fedora, CentOS, RHEL
- Best for: Standard distributions

**Linux x86_64 (musl)** - Universal
```
npm_packager-x86_64-unknown-linux-musl.tar.gz
```
- For: Alpine Linux, Docker containers, statically-linked binaries
- Best for: Containerized environments

**Linux ARM64 (GNU)** - ARM servers
```
npm_packager-aarch64-unknown-linux-gnu.tar.gz
```

**Linux ARM64 (musl)** - ARM containers
```
npm_packager-aarch64-unknown-linux-musl.tar.gz
```

**macOS Intel**
```
npm_packager-x86_64-apple-darwin.tar.gz
```
- For: Intel-based Macs (pre-2020)

**macOS Apple Silicon**
```
npm_packager-aarch64-apple-darwin.tar.gz
```
- For: M1/M2 Macs

**Windows**
```
npm_packager-x86_64-pc-windows-msvc.zip
```
- Extract and run `npm_packager.exe`

### How to detect your libc (Linux only)

```bash
ldd /bin/ls | head -1
```

- If output contains "musl" → use musl version
- If output contains "glibc" → use gnu version
- If unsure → try gnu first, fallback to musl if needed

## Failed Build?

If a build fails in GitHub Actions:

1. Go to the Actions tab
2. Click on the failed workflow
3. Check the logs to see what went wrong
4. Common issues:
   - Network timeout (usually fixes itself on retry)
   - Dependency issues (update Cargo.lock locally and commit)
   - Platform-specific issues (may need to debug locally)

To retry a failed build, delete the tag and recreate it:
```bash
git tag -d v0.1.0
git push origin :v0.1.0
git tag v0.1.0
git push origin v0.1.0
```

## Future Enhancements

Optional improvements to the release process:

1. **Checksums**: Generate SHA256 checksums for verification
2. **Binary stripping**: Reduce binary size by 50-70%
3. **Changelog**: Auto-generate from git commits
4. **crates.io**: Publish to Rust registry: `cargo publish`
5. **Docker**: Build and push Docker images

## References

- [GitHub Actions Documentation](https://docs.github.com/actions)
- [Semantic Versioning](https://semver.org/)
- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo-cross](https://github.com/cross-rs/cross)
