#!/bin/bash

# check-dependencies.sh
# Checks for required dependencies (gh CLI, etc.) and optionally installs them
# Usage:
#   ./check-dependencies.sh           # Interactive mode
#   ./check-dependencies.sh --quiet   # Non-interactive, warnings only
#   ./check-dependencies.sh --auto-install  # Auto-install missing deps

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
QUIET_MODE=false
AUTO_INSTALL=false

for arg in "$@"; do
    case $arg in
        --quiet)
            QUIET_MODE=true
            ;;
        --auto-install)
            AUTO_INSTALL=true
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --quiet         Non-interactive mode, only show warnings"
            echo "  --auto-install  Automatically install missing dependencies"
            echo "  --help          Show this help message"
            exit 0
            ;;
    esac
done

# Function to print colored messages
print_success() {
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${GREEN}✓${NC} $1"
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${BLUE}ℹ${NC} $1"
    fi
}

# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to detect OS
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ "$(uname)" = "Darwin" ]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# Function to install gh CLI on Ubuntu/Debian
install_gh_debian() {
    print_info "Installing GitHub CLI via official repository..."

    # Check if running with sudo
    if [ "$EUID" -ne 0 ]; then
        print_warning "Installation requires sudo privileges"

        # Install dependencies and add repository
        sudo apt-get update
        sudo apt-get install -y curl gpg

        # Add GitHub CLI repository
        curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
            sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
        sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg

        echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | \
            sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null

        # Install gh
        sudo apt-get update
        sudo apt-get install -y gh
    else
        # Already root
        apt-get update
        apt-get install -y curl gpg

        curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
            dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
        chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg

        echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | \
            tee /etc/apt/sources.list.d/github-cli.list > /dev/null

        apt-get update
        apt-get install -y gh
    fi

    print_success "GitHub CLI installed successfully"
}

# Function to install gh CLI on macOS
install_gh_macos() {
    print_info "Installing GitHub CLI via Homebrew..."

    if ! command_exists brew; then
        print_error "Homebrew is not installed. Please install it first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        return 1
    fi

    brew install gh
    print_success "GitHub CLI installed successfully"
}

# Function to check and install gh CLI
check_gh_cli() {
    if command_exists gh; then
        GH_VERSION=$(gh --version | head -n 1)
        print_success "GitHub CLI is installed: $GH_VERSION"
        return 0
    fi

    print_warning "GitHub CLI (gh) is not installed"

    if [ "$QUIET_MODE" = true ]; then
        print_info "Run 'make install-deps' to install missing dependencies"
        return 1
    fi

    echo ""
    print_info "GitHub CLI is required for managing issues, PRs, and releases"
    echo ""

    OS=$(detect_os)

    if [ "$AUTO_INSTALL" = true ]; then
        case "$OS" in
            ubuntu|debian)
                install_gh_debian
                return $?
                ;;
            macos)
                install_gh_macos
                return $?
                ;;
            *)
                print_error "Unsupported OS for auto-installation: $OS"
                print_info "Please install manually from: https://cli.github.com/"
                return 1
                ;;
        esac
    else
        # Interactive mode
        echo "Installation options:"
        case "$OS" in
            ubuntu|debian)
                echo "  1) Install via official GitHub CLI repository (recommended)"
                echo "  2) Install via snap: sudo snap install gh"
                echo "  3) Skip installation"
                ;;
            macos)
                echo "  1) Install via Homebrew: brew install gh"
                echo "  2) Skip installation"
                ;;
            *)
                echo "  See: https://cli.github.com/ for installation instructions"
                return 1
                ;;
        esac

        echo ""
        read -p "Choose an option (1-3, default: 3): " choice

        case "$choice" in
            1)
                if [ "$OS" = "macos" ]; then
                    install_gh_macos
                else
                    install_gh_debian
                fi
                return $?
                ;;
            2)
                if [ "$OS" != "macos" ]; then
                    print_info "Installing via snap..."
                    sudo snap install gh
                    print_success "GitHub CLI installed successfully"
                    return 0
                else
                    print_info "Skipping installation"
                    return 1
                fi
                ;;
            *)
                print_info "Skipping installation"
                return 1
                ;;
        esac
    fi
}

# Main execution
main() {
    if [ "$QUIET_MODE" = false ]; then
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  Checking KoproGo Dependencies"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
    fi

    MISSING_COUNT=0

    # Check GitHub CLI
    if ! check_gh_cli; then
        MISSING_COUNT=$((MISSING_COUNT + 1))
    fi

    # Add more dependency checks here in the future
    # Example:
    # if ! check_docker; then
    #     MISSING_COUNT=$((MISSING_COUNT + 1))
    # fi

    echo ""

    if [ $MISSING_COUNT -eq 0 ]; then
        if [ "$QUIET_MODE" = false ]; then
            print_success "All dependencies are installed"
        fi
        exit 0
    else
        if [ "$QUIET_MODE" = true ]; then
            print_warning "$MISSING_COUNT missing dependencies detected"
            print_info "Run './scripts/check-dependencies.sh' for details or 'make install-deps' to install"
        else
            print_warning "$MISSING_COUNT missing dependencies detected"
            echo ""
            print_info "You can install missing dependencies by running:"
            echo "  make install-deps"
            echo ""
            print_info "Or install manually following the instructions above"
        fi
        exit 0  # Non-blocking, just warn
    fi
}

main "$@"
