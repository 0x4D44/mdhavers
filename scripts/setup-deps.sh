#!/bin/bash
# setup-deps.sh - Install mdhavers optional dependencies on Linux/WSL
#
# Usage: ./scripts/setup-deps.sh [OPTIONS]
#
# Options:
#   --all       Install all optional dependencies (default)
#   --llvm      Install LLVM 15 and related packages
#   --graphics  Install raylib dependencies
#   --audio     Install audio dependencies
#   --help      Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect package manager
detect_package_manager() {
    if command -v apt-get &>/dev/null; then
        PKG_MANAGER="apt"
        PKG_INSTALL="sudo apt-get install -y"
        PKG_UPDATE="sudo apt-get update"
    elif command -v dnf &>/dev/null; then
        PKG_MANAGER="dnf"
        PKG_INSTALL="sudo dnf install -y"
        PKG_UPDATE="sudo dnf check-update || true"
    elif command -v pacman &>/dev/null; then
        PKG_MANAGER="pacman"
        PKG_INSTALL="sudo pacman -S --noconfirm"
        PKG_UPDATE="sudo pacman -Sy"
    elif command -v zypper &>/dev/null; then
        PKG_MANAGER="zypper"
        PKG_INSTALL="sudo zypper install -y"
        PKG_UPDATE="sudo zypper refresh"
    else
        error "Unsupported package manager. Please install dependencies manually."
    fi
    info "Detected package manager: $PKG_MANAGER"
}

# Install LLVM 15 and dependencies
install_llvm() {
    info "Installing LLVM 15 and dependencies..."

    case $PKG_MANAGER in
        apt)
            # Add LLVM apt repository for Ubuntu/Debian
            if ! command -v llvm-config-15 &>/dev/null; then
                info "Adding LLVM apt repository..."
                wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | sudo tee /etc/apt/trusted.gpg.d/apt.llvm.org.asc >/dev/null 2>&1 || true

                # Detect Ubuntu/Debian version
                if [ -f /etc/os-release ]; then
                    . /etc/os-release
                    case $VERSION_CODENAME in
                        noble|mantic)
                            LLVM_REPO="deb http://apt.llvm.org/$VERSION_CODENAME/ llvm-toolchain-$VERSION_CODENAME-15 main"
                            ;;
                        jammy|kinetic|lunar)
                            LLVM_REPO="deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-15 main"
                            ;;
                        focal)
                            LLVM_REPO="deb http://apt.llvm.org/focal/ llvm-toolchain-focal-15 main"
                            ;;
                        bookworm|bullseye)
                            LLVM_REPO="deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-15 main"
                            ;;
                        *)
                            warn "Unknown distro version: $VERSION_CODENAME. Trying default LLVM packages."
                            ;;
                    esac

                    if [ -n "$LLVM_REPO" ]; then
                        echo "$LLVM_REPO" | sudo tee /etc/apt/sources.list.d/llvm-15.list >/dev/null
                        $PKG_UPDATE
                    fi
                fi
            fi

            $PKG_INSTALL llvm-15 llvm-15-dev libpolly-15-dev lld-15 libzstd-dev
            ;;
        dnf)
            $PKG_INSTALL llvm15 llvm15-devel lld libzstd-devel
            ;;
        pacman)
            $PKG_INSTALL llvm15 lld zstd
            ;;
        zypper)
            $PKG_INSTALL llvm15 llvm15-devel lld libzstd-devel
            ;;
    esac

    if command -v llvm-config-15 &>/dev/null || command -v llvm-config &>/dev/null; then
        success "LLVM installed successfully"
    else
        warn "LLVM installation may have failed. Check manually with: llvm-config-15 --version"
    fi
}

# Install graphics (raylib) dependencies
install_graphics() {
    info "Installing graphics (raylib) dependencies..."

    case $PKG_MANAGER in
        apt)
            $PKG_INSTALL cmake libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev libx11-dev
            ;;
        dnf)
            $PKG_INSTALL cmake libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel mesa-libGL-devel libX11-devel
            ;;
        pacman)
            $PKG_INSTALL cmake libxrandr libxinerama libxcursor libxi mesa libx11
            ;;
        zypper)
            $PKG_INSTALL cmake libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel Mesa-libGL-devel libX11-devel
            ;;
    esac

    success "Graphics dependencies installed"
}

# Install audio dependencies
install_audio() {
    info "Installing audio dependencies..."

    case $PKG_MANAGER in
        apt)
            $PKG_INSTALL libasound2-dev
            ;;
        dnf)
            $PKG_INSTALL alsa-lib-devel
            ;;
        pacman)
            $PKG_INSTALL alsa-lib
            ;;
        zypper)
            $PKG_INSTALL alsa-devel
            ;;
    esac

    success "Audio dependencies installed"
}

# Install Rust if not present
check_rust() {
    if ! command -v cargo &>/dev/null; then
        warn "Rust not found. Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        return 1
    fi
    success "Rust found: $(rustc --version)"
    return 0
}

# Install essential build tools
install_build_tools() {
    info "Installing essential build tools..."

    case $PKG_MANAGER in
        apt)
            $PKG_INSTALL build-essential pkg-config
            ;;
        dnf)
            $PKG_INSTALL gcc gcc-c++ make pkg-config
            ;;
        pacman)
            $PKG_INSTALL base-devel pkg-config
            ;;
        zypper)
            $PKG_INSTALL gcc gcc-c++ make pkg-config
            ;;
    esac

    success "Build tools installed"
}

# Show help
show_help() {
    echo "setup-deps.sh - Install mdhavers optional dependencies"
    echo ""
    echo "Usage: ./scripts/setup-deps.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --all       Install all optional dependencies (default)"
    echo "  --llvm      Install LLVM 15 and related packages"
    echo "  --graphics  Install raylib dependencies"
    echo "  --audio     Install audio dependencies"
    echo "  --tools     Install essential build tools only"
    echo "  --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./scripts/setup-deps.sh              # Install everything"
    echo "  ./scripts/setup-deps.sh --llvm       # Install only LLVM"
    echo "  ./scripts/setup-deps.sh --graphics   # Install only graphics deps"
}

# Main
main() {
    echo "==========================================="
    echo "  mdhavers Dependency Installer (Linux)   "
    echo "==========================================="
    echo ""

    # Parse arguments
    INSTALL_ALL=true
    INSTALL_LLVM=false
    INSTALL_GRAPHICS=false
    INSTALL_AUDIO=false
    INSTALL_TOOLS=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                INSTALL_ALL=true
                shift
                ;;
            --llvm)
                INSTALL_ALL=false
                INSTALL_LLVM=true
                shift
                ;;
            --graphics)
                INSTALL_ALL=false
                INSTALL_GRAPHICS=true
                shift
                ;;
            --audio)
                INSTALL_ALL=false
                INSTALL_AUDIO=true
                shift
                ;;
            --tools)
                INSTALL_ALL=false
                INSTALL_TOOLS=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                error "Unknown option: $1. Use --help for usage."
                ;;
        esac
    done

    detect_package_manager

    info "Updating package lists..."
    $PKG_UPDATE || true

    check_rust || true

    if $INSTALL_ALL; then
        install_build_tools
        install_llvm
        install_graphics
        install_audio
    else
        $INSTALL_TOOLS && install_build_tools
        $INSTALL_LLVM && install_llvm
        $INSTALL_GRAPHICS && install_graphics
        $INSTALL_AUDIO && install_audio
    fi

    echo ""
    echo "==========================================="
    success "Dependency installation complete!"
    echo "==========================================="
    echo ""
    info "Next steps:"
    echo "  1. Build with all features: cargo build"
    echo "  2. Or build minimal: cargo build --no-default-features --features minimal"
    echo "  3. Run tests: cargo test"
    echo ""
}

main "$@"
