#!/bin/sh
# mdhavers uninstaller script
# Usage: ./uninstall.sh [--yes]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MDHAVERS_HOME="${MDHAVERS_HOME:-$HOME/.mdhavers}"

# Parse arguments
AUTO_YES=false

for arg in "$@"; do
    case $arg in
        --yes|-y)
            AUTO_YES=true
            ;;
        --help|-h)
            echo "mdhavers uninstaller"
            echo ""
            echo "Usage: uninstall.sh [OPTIONS]"
            echo ""
            echo "Options:"
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

# Remove mdhavers from shell rc files
remove_from_shell_rc() {
    local rc_file="$1"

    if [ -f "$rc_file" ] && grep -q "mdhavers" "$rc_file"; then
        say "removing mdhavers from $rc_file..."

        # Create backup
        cp "$rc_file" "$rc_file.backup.$(date +%Y%m%d%H%M%S)"

        # Remove mdhavers-related lines
        # This removes lines containing 'mdhavers' and the comment line before it
        sed -i.tmp '/# mdhavers/d; /\.mdhavers/d' "$rc_file"
        rm -f "$rc_file.tmp"

        say "removed mdhavers configuration from $rc_file"
    fi
}

# Remove shell completions
remove_completions() {
    say "removing shell completions..."

    # System bash completions
    if [ -f "/etc/bash_completion.d/mdhavers.bash" ]; then
        rm -f "/etc/bash_completion.d/mdhavers.bash" 2>/dev/null || true
    fi

    # User bash completions
    rm -f "$HOME/.local/share/bash-completion/completions/mdhavers" 2>/dev/null || true

    # Zsh completions
    rm -f "$HOME/.zsh/completions/_mdhavers" 2>/dev/null || true

    # Fish completions
    rm -f "$HOME/.config/fish/completions/mdhavers.fish" 2>/dev/null || true
}

# Show what will be removed
show_removal_plan() {
    echo ""
    printf "${BLUE}The following will be removed:${NC}\n"
    echo ""

    if [ -d "$MDHAVERS_HOME" ]; then
        echo "  Directory: $MDHAVERS_HOME"
        if [ -d "$MDHAVERS_HOME/bin" ]; then
            echo "    - bin/mdhavers"
            [ -f "$MDHAVERS_HOME/bin/mdhavers-lsp" ] && echo "    - bin/mdhavers-lsp"
        fi
        [ -d "$MDHAVERS_HOME/lib" ] && echo "    - lib/ (standard library)"
        [ -d "$MDHAVERS_HOME/completions" ] && echo "    - completions/"
        [ -d "$MDHAVERS_HOME/examples" ] && echo "    - examples/"
        [ -f "$MDHAVERS_HOME/env" ] && echo "    - env"
    else
        warn "no installation found at $MDHAVERS_HOME"
    fi

    echo ""
    echo "  Shell configuration will be cleaned from:"

    for rc in "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.config/fish/config.fish"; do
        if [ -f "$rc" ] && grep -q "mdhavers" "$rc"; then
            echo "    - $rc"
        fi
    done

    echo ""
}

# Main uninstallation
main() {
    echo ""
    printf "${BLUE}"
    echo "=================================="
    echo "    mdhavers Uninstaller"
    echo "=================================="
    printf "${NC}"
    echo ""

    if [ ! -d "$MDHAVERS_HOME" ]; then
        warn "mdhavers does not appear to be installed at $MDHAVERS_HOME"

        # Still check for shell rc entries
        local found_rc=false
        for rc in "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.profile"; do
            if [ -f "$rc" ] && grep -q "mdhavers" "$rc"; then
                found_rc=true
                break
            fi
        done

        if [ "$found_rc" = false ]; then
            say "nothing to uninstall"
            exit 0
        fi
    fi

    show_removal_plan

    if ! confirm "Proceed with uninstallation?"; then
        say "uninstallation cancelled"
        exit 0
    fi

    echo ""

    # Remove shell rc configurations
    for rc in "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.profile"; do
        remove_from_shell_rc "$rc"
    done

    # Handle fish config separately (different path)
    if [ -f "$HOME/.config/fish/config.fish" ]; then
        remove_from_shell_rc "$HOME/.config/fish/config.fish"
    fi

    # Remove completions from system locations
    remove_completions

    # Remove installation directory
    if [ -d "$MDHAVERS_HOME" ]; then
        say "removing $MDHAVERS_HOME..."
        rm -rf "$MDHAVERS_HOME"
        say "removed installation directory"
    fi

    echo ""
    printf "${GREEN}"
    echo "=================================="
    echo "    Uninstallation Complete"
    echo "=================================="
    printf "${NC}"
    echo ""
    say "mdhavers has been uninstalled"
    echo ""
    say "restart your shell to complete the process"
    echo ""

    # Check if mdhavers is still in PATH (from other sources)
    if command -v mdhavers > /dev/null 2>&1; then
        warn "mdhavers is still available in PATH from another location:"
        warn "  $(command -v mdhavers)"
    fi
}

main "$@"
