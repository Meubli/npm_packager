#!/bin/bash
# npm_packager installer script
# Auto-detects platform and downloads the appropriate binary

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
REPO="Meubli/npm_packager"
VERSION="${1:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo -e "${BLUE}npm_packager Installer${NC}"
echo "Version: $VERSION"
echo "Install directory: $INSTALL_DIR"
echo ""

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Detect platform and architecture
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case "$PLATFORM" in
  Linux)
    # Detect libc type (glibc vs musl)
    if ldd /bin/ls | grep -q musl; then
      TARGET="x86_64-unknown-linux-musl"
    else
      TARGET="x86_64-unknown-linux-gnu"
    fi
    
    # Handle ARM64
    if [ "$ARCH" = "aarch64" ]; then
      if ldd /bin/ls | grep -q musl; then
        TARGET="aarch64-unknown-linux-musl"
      else
        TARGET="aarch64-unknown-linux-gnu"
      fi
    fi
    
    EXT="tar.gz"
    EXTRACT_CMD="tar xzf"
    ;;
    
  Darwin)
    # macOS
    if [ "$ARCH" = "arm64" ]; then
      TARGET="aarch64-apple-darwin"
    else
      TARGET="x86_64-apple-darwin"
    fi
    EXT="tar.gz"
    EXTRACT_CMD="tar xzf"
    ;;
    
  MINGW*|MSYS*|CYGWIN*)
    # Windows
    TARGET="x86_64-pc-windows-msvc"
    EXT="zip"
    EXTRACT_CMD="unzip -q"
    ;;
    
  *)
    echo -e "${RED}Error: Unsupported platform: $PLATFORM${NC}"
    exit 1
    ;;
esac

BINARY_URL="https://github.com/$REPO/releases/download/$VERSION/npm_packager-$TARGET.$EXT"
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo -e "${BLUE}Downloading npm_packager for $TARGET...${NC}"
echo "URL: $BINARY_URL"

# Download
if ! curl -fsSL -o "$TEMP_DIR/npm_packager.$EXT" "$BINARY_URL"; then
  echo -e "${RED}Error: Failed to download from $BINARY_URL${NC}"
  exit 1
fi

# Extract
cd "$TEMP_DIR"
$EXTRACT_CMD "npm_packager.$EXT"

# Find the binary in the extracted directory
if [ -f "npm_packager-$TARGET/npm_packager" ]; then
  BINARY_PATH="npm_packager-$TARGET/npm_packager"
elif [ -f "npm_packager-$TARGET/npm_packager.exe" ]; then
  BINARY_PATH="npm_packager-$TARGET/npm_packager.exe"
else
  echo -e "${RED}Error: Could not find binary in archive${NC}"
  exit 1
fi

# Install
echo -e "${BLUE}Installing to $INSTALL_DIR...${NC}"
install -m 755 "$BINARY_PATH" "$INSTALL_DIR/npm_packager"

# Add to PATH if necessary
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo ""
  echo -e "${BLUE}Add the following line to your shell config (~/.bashrc, ~/.zshrc, etc.):${NC}"
  echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
  echo ""
fi

echo -e "${GREEN}✓ npm_packager installed successfully!${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  npm_packager --package-lock package-lock.json"
echo ""
echo -e "${BLUE}Help:${NC}"
echo "  npm_packager --help"
