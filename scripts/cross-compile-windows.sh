#!/bin/bash

# Cross-compilation script for Woodstock Backup - Windows targets
# Author: Woodstock Development Team
# Date: June 2025

set -e

echo "🏗️  Woodstock Backup - Cross-compilation for Windows"
echo "=================================================="

# Configuration
TARGET="x86_64-pc-windows-gnu"
BUILD_TYPE="${1:-release}"
OUTPUT_DIR="dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
print_status "Checking prerequisites..."

# Check if Rust target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    print_status "Installing Rust target: $TARGET"
    rustup target add "$TARGET"
else
    print_success "Rust target $TARGET is already installed"
fi

# Check if MinGW is installed
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    print_error "MinGW cross-compiler not found!"
    print_status "Install it with: sudo apt install gcc-mingw-w64-x86-64"
    exit 1
else
    print_success "MinGW cross-compiler found"
fi

# Clean previous build if requested
if [[ "$2" == "--clean" ]]; then
    print_status "Cleaning previous build..."
    cargo clean --target "$TARGET"
fi

# Build for Windows
print_status "Building for Windows ($TARGET) in $BUILD_TYPE mode..."
cargo build --target "$TARGET" --profile "$BUILD_TYPE"
