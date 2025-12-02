#!/bin/sh
# Slides Installation Script for Linux and macOS
# Usage: curl -sSL https://raw.githubusercontent.com/OWNER/REPO/master/scripts/install.sh | sh

set -e

# CONFIGURE THIS: Set your GitHub repo
OWNER="MichaelBrauner"
REPO="slides-rs"

BINARY_NAME="slides"
INSTALL_DIR="$HOME/.local/bin"
GITHUB_API="https://api.github.com/repos/$OWNER/$REPO/releases/latest"

# Colors (only if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    CYAN='\033[0;36m'
    GRAY='\033[0;90m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    CYAN=''
    GRAY=''
    NC=''
fi

echo "${CYAN}Installing slides...${NC}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64|amd64)
                ASSET_PATTERN="linux-x86_64"
                ;;
            *)
                echo "${RED}Error: Unsupported architecture: $ARCH${NC}"
                exit 1
                ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64)
                ASSET_PATTERN="macos-x86_64"
                ;;
            arm64)
                ASSET_PATTERN="macos-aarch64"
                ;;
            *)
                echo "${RED}Error: Unsupported architecture: $ARCH${NC}"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "${RED}Error: Unsupported OS: $OS${NC}"
        echo "For Windows, use the PowerShell installer."
        exit 1
        ;;
esac

echo "${GRAY}  Detected: $OS ($ARCH)${NC}"

# Create install directory
mkdir -p "$INSTALL_DIR"
echo "${GRAY}  Install directory: $INSTALL_DIR${NC}"

# Get latest release info
echo "${GRAY}  Fetching latest release...${NC}"
RELEASE_JSON=$(curl -sS "$GITHUB_API")

LATEST_TAG=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*: "\(.*\)".*/\1/')
if [ -z "$LATEST_TAG" ]; then
    echo "${RED}Error: Could not determine latest version.${NC}"
    echo "${YELLOW}Please download manually from: https://github.com/$OWNER/$REPO/releases${NC}"
    exit 1
fi

echo "${GRAY}  Latest version: $LATEST_TAG${NC}"

# Find download URL for our platform
DOWNLOAD_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url" | grep "$ASSET_PATTERN" | head -1 | sed 's/.*: "\(.*\)".*/\1/')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "${RED}Error: Binary for $ASSET_PATTERN not found in release.${NC}"
    exit 1
fi

DEST_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# Download binary
echo "${GRAY}  Downloading slides...${NC}"
if command -v curl > /dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$DEST_PATH"
elif command -v wget > /dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$DEST_PATH"
else
    echo "${RED}Error: Neither curl nor wget found.${NC}"
    exit 1
fi

# Make executable
chmod +x "$DEST_PATH"
echo "${GRAY}  Downloaded to: $DEST_PATH${NC}"

# Check if INSTALL_DIR is in PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        IN_PATH=1
        ;;
    *)
        IN_PATH=0
        ;;
esac

echo ""
echo "${GREEN}Installation complete!${NC}"
echo ""

if [ "$IN_PATH" -eq 0 ]; then
    echo "${YELLOW}IMPORTANT: Add ~/.local/bin to your PATH${NC}"
    echo ""

    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        bash)
            echo "Add this line to your ~/.bashrc:"
            echo "${CYAN}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
            echo ""
            echo "Then run: ${CYAN}source ~/.bashrc${NC}"
            ;;
        zsh)
            echo "Add this line to your ~/.zshrc:"
            echo "${CYAN}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
            echo ""
            echo "Then run: ${CYAN}source ~/.zshrc${NC}"
            ;;
        fish)
            echo "Run: ${CYAN}fish_add_path ~/.local/bin${NC}"
            ;;
        *)
            echo "Add ~/.local/bin to your PATH."
            ;;
    esac
    echo ""
fi

echo "Then you can use: ${CYAN}slides build${NC}"
echo ""

# Show version
if [ -x "$DEST_PATH" ]; then
    VERSION=$("$DEST_PATH" --version 2>/dev/null || echo "")
    if [ -n "$VERSION" ]; then
        echo "${GRAY}Installed: $VERSION${NC}"
    fi
fi
