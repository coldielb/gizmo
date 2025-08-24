#!/bin/bash
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Check dir
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/main.rs" ]]; then
    print_error "This script must be run from the gizmo project root directory"
    print_error "Please cd to the directory containing Cargo.toml and try again"
    exit 1
fi

# Make sure cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo (Rust) is not installed or not in PATH"
    print_error "Please install Rust from https://rustup.rs/ and try again"
    exit 1
fi

print_status "Starting Gizmo installation..."
print_status "Project: $(pwd)"

print_status "Building Gizmo in release mode..."
if cargo build --release; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

INSTALL_DIR=""
if [[ -d "$HOME/.local/bin" ]]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [[ -d "$HOME/bin" ]]; then
    INSTALL_DIR="$HOME/bin"
elif [[ -w "/usr/local/bin" ]]; then
    INSTALL_DIR="/usr/local/bin"
else
    # Create ~/.local/bin if it doesn't exist
    mkdir -p "$HOME/.local/bin"
    INSTALL_DIR="$HOME/.local/bin"
fi

print_status "Installing to: $INSTALL_DIR"

# Copy the binary
BINARY_SRC="target/release/gizmo"
BINARY_DEST="$INSTALL_DIR/gizmo"

if [[ -f "$BINARY_SRC" ]]; then
    if cp "$BINARY_SRC" "$BINARY_DEST"; then
        chmod +x "$BINARY_DEST"
        print_success "Gizmo binary installed to $BINARY_DEST"
    else
        print_error "Failed to copy binary to $BINARY_DEST"
        print_error "You may need to run with sudo or choose a different install location"
        exit 1
    fi
else
    print_error "Binary not found at $BINARY_SRC"
    print_error "Build may have failed"
    exit 1
fi

if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
    print_success "Installation directory is already in PATH"
else
    print_warning "Installation directory $INSTALL_DIR is not in your PATH"
    print_warning "You may need to add it to your shell profile:"
    echo ""
    echo "For bash/zsh, add this line to ~/.bashrc or ~/.zshrc:"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
    echo "Then restart your terminal or run: source ~/.bashrc (or ~/.zshrc)"
    echo ""
fi

# Print usage instructions
echo ""
echo -e "${GREEN}🎉 Gizmo has been installed successfully!${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  gizmo start <script.gzmo>    # Start animation with script"
echo "  gizmo restart                # Restart current animation"
echo "  gizmo stop                   # Stop animation"
echo ""
echo -e "${BLUE}Example scripts:${NC}"
echo "  gizmo start examples/spinner.gzmo"
echo "  gizmo start examples/digital_rain.gzmo"
echo "  gizmo start examples/psychedelic_tunnel.gzmo"
echo ""

# Check PATH one more time and give final instructions
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  Remember to add $INSTALL_DIR to your PATH to use 'gizmo' from anywhere${NC}"
    echo ""
fi

print_success "Installation complete! 🚀"
