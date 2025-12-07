#!/bin/sh
# mdhavers installer script
# Usage: curl -sSf https://raw.githubusercontent.com/.../install.sh | sh
#    or: ./install.sh [--local] [--yes]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
MDHAVERS_HOME="${MDHAVERS_HOME:-$HOME/.mdhavers}"
INSTALL_DIR="$MDHAVERS_HOME"
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib"
COMPLETIONS_DIR="$INSTALL_DIR/completions"

# Parse arguments
LOCAL_INSTALL=false
AUTO_YES=false
SCRIPT_DIR=""

for arg in "$@"; do
    case $arg in
        --local)
            LOCAL_INSTALL=true
            SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
            ;;
        --yes|-y)
            AUTO_YES=true
            ;;
        --help|-h)
            echo "mdhavers installer"
            echo ""
            echo "Usage: install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --local    Install from local build (for development)"
            echo "  --yes, -y  Skip confirmation prompts"
            echo "  --help, -h Show this help message"
            exit 0
            ;;
    esac
done

# Helper functions
say() {
    printf "${GREEN}mdhavers:${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}warning:${NC} %s\n" "$1"
}

err() {
    printf "${RED}error:${NC} %s\n" "$1" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found)"
    fi
}

confirm() {
    if [ "$AUTO_YES" = true ]; then
        return 0
    fi
    printf "%s [y/N] " "$1"
    read -r response
    case "$response" in
        [yY][eE][sS]|[yY]) return 0 ;;
        *) return 1 ;;
    esac
}

# Detect platform
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            OS="linux"
            ;;
        Darwin)
            OS="macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            ;;
        *)
            err "unsupported operating system: $os"
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            err "unsupported architecture: $arch"
            ;;
    esac

    say "detected platform: $OS-$ARCH"
}

# Check for existing installation
check_existing() {
    if [ -d "$INSTALL_DIR" ]; then
        warn "existing installation found at $INSTALL_DIR"
        if ! confirm "Do you want to overwrite it?"; then
            say "installation cancelled"
            exit 0
        fi
    fi
}

# Create directory structure
create_dirs() {
    say "creating installation directories..."
    mkdir -p "$BIN_DIR"
    mkdir -p "$LIB_DIR"
    mkdir -p "$COMPLETIONS_DIR"
}

# Install binaries
install_binaries() {
    say "installing binaries..."

    if [ "$LOCAL_INSTALL" = true ]; then
        # Local installation - copy from build directory
        local project_root="$SCRIPT_DIR/.."

        # Check for release binary first, then debug
        if [ -f "$project_root/target/release/mdhavers" ]; then
            cp "$project_root/target/release/mdhavers" "$BIN_DIR/"
            say "installed mdhavers (release build)"
        elif [ -f "$project_root/target/debug/mdhavers" ]; then
            cp "$project_root/target/debug/mdhavers" "$BIN_DIR/"
            warn "installed mdhavers (debug build - consider running 'cargo build --release')"
        else
            err "no mdhavers binary found. Run 'cargo build --release' first."
        fi

        # Install LSP if available
        if [ -f "$project_root/target/release/mdhavers-lsp" ]; then
            cp "$project_root/target/release/mdhavers-lsp" "$BIN_DIR/"
            say "installed mdhavers-lsp"
        elif [ -f "$project_root/target/debug/mdhavers-lsp" ]; then
            cp "$project_root/target/debug/mdhavers-lsp" "$BIN_DIR/"
        fi
    else
        # Remote installation - download from releases
        # TODO: Implement download from GitHub releases
        err "remote installation not yet implemented. Use --local for now."
    fi

    chmod +x "$BIN_DIR/mdhavers"
    [ -f "$BIN_DIR/mdhavers-lsp" ] && chmod +x "$BIN_DIR/mdhavers-lsp"
}

# Install shell completions
install_completions() {
    say "installing shell completions..."

    if [ "$LOCAL_INSTALL" = true ]; then
        local completions_src="$SCRIPT_DIR/completions"
        if [ -d "$completions_src" ]; then
            cp "$completions_src"/* "$COMPLETIONS_DIR/" 2>/dev/null || true
        fi
    fi

    # Install to system completion directories if they exist
    # Bash
    if [ -d "/etc/bash_completion.d" ] && [ -w "/etc/bash_completion.d" ]; then
        cp "$COMPLETIONS_DIR/mdhavers.bash" "/etc/bash_completion.d/" 2>/dev/null || true
    elif [ -d "$HOME/.local/share/bash-completion/completions" ]; then
        mkdir -p "$HOME/.local/share/bash-completion/completions"
        cp "$COMPLETIONS_DIR/mdhavers.bash" "$HOME/.local/share/bash-completion/completions/mdhavers" 2>/dev/null || true
    fi

    # Zsh
    if [ -d "$HOME/.zsh/completions" ]; then
        cp "$COMPLETIONS_DIR/mdhavers.zsh" "$HOME/.zsh/completions/_mdhavers" 2>/dev/null || true
    fi

    # Fish
    if [ -d "$HOME/.config/fish/completions" ]; then
        cp "$COMPLETIONS_DIR/mdhavers.fish" "$HOME/.config/fish/completions/" 2>/dev/null || true
    fi
}

# Create env script
create_env_script() {
    say "creating environment script..."

    cat > "$INSTALL_DIR/env" << 'EOF'
#!/bin/sh
# mdhavers environment setup
# Source this file to add mdhavers to your PATH

export MDHAVERS_HOME="$HOME/.mdhavers"
export PATH="$MDHAVERS_HOME/bin:$PATH"

# Optional: Enable shell completions
if [ -n "$BASH_VERSION" ] && [ -f "$MDHAVERS_HOME/completions/mdhavers.bash" ]; then
    . "$MDHAVERS_HOME/completions/mdhavers.bash"
elif [ -n "$ZSH_VERSION" ] && [ -f "$MDHAVERS_HOME/completions/mdhavers.zsh" ]; then
    . "$MDHAVERS_HOME/completions/mdhavers.zsh"
fi
EOF
    chmod +x "$INSTALL_DIR/env"
}

# Configure shell
configure_shell() {
    say "configuring shell..."

    local shell_rc=""
    local shell_name=""
    local mdhavers_line='. "$HOME/.mdhavers/env"'
    local mdhavers_marker="# mdhavers"

    # Detect current shell
    case "$SHELL" in
        */bash)
            shell_name="bash"
            if [ -f "$HOME/.bashrc" ]; then
                shell_rc="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_rc="$HOME/.bash_profile"
            fi
            ;;
        */zsh)
            shell_name="zsh"
            shell_rc="$HOME/.zshrc"
            ;;
        */fish)
            shell_name="fish"
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            shell_name="unknown"
            shell_rc="$HOME/.profile"
            ;;
    esac

    if [ -z "$shell_rc" ]; then
        warn "could not detect shell configuration file"
        say "add this to your shell's rc file manually:"
        echo ""
        echo "  $mdhavers_marker"
        echo "  $mdhavers_line"
        echo ""
        return
    fi

    # Check if already configured
    if [ -f "$shell_rc" ] && grep -q "mdhavers" "$shell_rc"; then
        say "shell already configured in $shell_rc"
        return
    fi

    if confirm "Modify $shell_rc to add mdhavers to PATH?"; then
        # Create backup
        if [ -f "$shell_rc" ]; then
            cp "$shell_rc" "$shell_rc.backup.$(date +%Y%m%d%H%M%S)"
        fi

        # Add mdhavers configuration
        echo "" >> "$shell_rc"
        echo "$mdhavers_marker" >> "$shell_rc"
        echo "$mdhavers_line" >> "$shell_rc"

        say "added mdhavers to $shell_rc"
    else
        say "skipped shell configuration"
        echo ""
        say "to complete installation, add this to your $shell_rc:"
        echo ""
        echo "  $mdhavers_marker"
        echo "  $mdhavers_line"
        echo ""
    fi
}

# Copy standard library files
install_stdlib() {
    say "installing standard library..."

    if [ "$LOCAL_INSTALL" = true ]; then
        local stdlib_src="$SCRIPT_DIR/../stdlib"
        if [ -d "$stdlib_src" ]; then
            cp -r "$stdlib_src"/* "$LIB_DIR/" 2>/dev/null || true
            say "installed standard library"
        fi

        # Also copy examples
        local examples_src="$SCRIPT_DIR/../examples"
        if [ -d "$examples_src" ]; then
            mkdir -p "$INSTALL_DIR/examples"
            cp -r "$examples_src"/* "$INSTALL_DIR/examples/" 2>/dev/null || true
            say "installed examples"
        fi
    fi
}

# Print success message
print_success() {
    echo ""
    printf "${GREEN}"
    echo "  __  __     _ _"
    echo " |  \\/  | __| | |__   __ ___   _____ _ __ ___"
    echo " | |\\/| |/ _\` | '_ \\ / _\` \\ \\ / / _ \\ '__/ __|"
    echo " | |  | | (_| | | | | (_| |\\ V /  __/ |  \\__ \\"
    echo " |_|  |_|\\__,_|_| |_|\\__,_| \\_/ \\___|_|  |___/"
    printf "${NC}"
    echo ""
    say "mdhavers has been installed successfully!"
    echo ""
    say "installed to: $INSTALL_DIR"
    say "binary at: $BIN_DIR/mdhavers"
    echo ""

    if command -v "$BIN_DIR/mdhavers" > /dev/null 2>&1; then
        say "version: $($BIN_DIR/mdhavers --version 2>/dev/null || echo 'unknown')"
    fi

    echo ""
    printf "${CYAN}To get started:${NC}\n"
    echo ""
    echo "  1. Restart your shell or run:"
    echo "     source ~/.mdhavers/env"
    echo ""
    echo "  2. Verify installation:"
    echo "     mdhavers --version"
    echo ""
    echo "  3. Try the REPL:"
    echo "     mdhavers repl"
    echo ""
    echo "  4. Run a program:"
    echo "     mdhavers run examples/hello.braw"
    echo ""
    printf "${CYAN}For help:${NC}\n"
    echo "  mdhavers --help"
    echo ""
}

# Main installation flow
main() {
    echo ""
    printf "${BLUE}"
    echo "=================================="
    echo "    mdhavers Installer"
    echo "=================================="
    printf "${NC}"
    echo ""

    need_cmd chmod
    need_cmd mkdir
    need_cmd cp

    detect_platform
    check_existing
    create_dirs
    install_binaries
    install_stdlib
    install_completions
    create_env_script
    configure_shell
    print_success
}

main "$@"
