#!/usr/bin/env bash
# AETHER_02 — Linux MCP Server Installer
# Auto-detects Cursor, Claude Desktop, Windsurf, VS Code and registers the MCP server.
# Usage: curl -sSL https://raw.githubusercontent.com/foursecondfivefour/aether-linux-mcp-server/main/install.sh | bash

set -euo pipefail

BOLD=$(tput bold 2>/dev/null || echo "")
GREEN=$(tput setaf 2 2>/dev/null || echo "")
YELLOW=$(tput setaf 3 2>/dev/null || echo "")
RED=$(tput setaf 1 2>/dev/null || echo "")
NC=$(tput sgr0 2>/dev/null || echo "")

RELEASE_TAG="${1:-latest}"
REPO="foursecondfivefour/aether-linux-mcp-server"
BINARY_NAME="aether-mcp-server"
INSTALL_DIR="${HOME}/.local/bin"

echo "${BOLD}AETHER_02 — Linux MCP Server Installer${NC}"
echo "Repository: https://github.com/${REPO}"
echo ""

# Create install directory
mkdir -p "${INSTALL_DIR}"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# Download latest release
if [ "${RELEASE_TAG}" = "latest" ]; then
    echo "${GREEN}Downloading latest release...${NC}"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
else
    echo "${GREEN}Downloading release ${RELEASE_TAG}...${NC}"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${BINARY_NAME}"
fi

if command -v curl &> /dev/null; then
    curl -sSL "${DOWNLOAD_URL}" -o "${BINARY_PATH}"
elif command -v wget &> /dev/null; then
    wget -q "${DOWNLOAD_URL}" -O "${BINARY_PATH}"
else
    echo "${RED}Error: curl or wget required.${NC}"
    exit 1
fi

chmod +x "${BINARY_PATH}"
echo "${GREEN}Binary installed: ${BINARY_PATH}${NC}"

# Create .env with safe defaults if not exists
ENV_FILE="${HOME}/.config/aether-02/.env"
mkdir -p "$(dirname "${ENV_FILE}")"
if [ ! -f "${ENV_FILE}" ]; then
    cat > "${ENV_FILE}" << 'ENVEOF'
# AETHER_02 — Feature Gates (0=disabled, 1=enabled)
AETHER_KEXEC_LOAD=0
AETHER_MODULE_LOAD=0
AETHER_BPF_LOAD=0
AETHER_PTRACE_ATTACH=0
AETHER_NAMESPACE_CREATE=0
AETHER_OFFLINE_MOUNT=0
AETHER_PARTITION_EDIT=0
AETHER_TOKEN_MANIPULATION=0
ENVEOF
    echo "${GREEN}Created .env: ${ENV_FILE}${NC}"
fi

# MCP JSON configuration
MCP_CONFIG=$(cat << EOFJSON
{
  "mcpServers": {
    "aether-02": {
      "command": "${BINARY_PATH}",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
EOFJSON
)

# Auto-detect and register in all supported editors
register_editor() {
    local NAME="$1"
    local CONFIG_PATH="$2"
    local DIR
    DIR="$(dirname "${CONFIG_PATH}")"

    if [ -f "${CONFIG_PATH}" ]; then
        echo "${YELLOW}${NAME}: existing config found. Merging...${NC}"
        # Simple merge: add aether-02 entry
        if command -v jq &> /dev/null; then
            TMP=$(mktemp)
            jq --argjson aether "${MCP_CONFIG}" '.mcpServers += $aether.mcpServers' "${CONFIG_PATH}" > "${TMP}"
            mv "${TMP}" "${CONFIG_PATH}"
        else
            echo "${YELLOW}jq not found — skipping merge for ${NAME}. Add manually:${NC}"
            echo "${MCP_CONFIG}"
        fi
    else
        mkdir -p "${DIR}"
        echo "${MCP_CONFIG}" > "${CONFIG_PATH}"
        echo "${GREEN}${NAME}: registered at ${CONFIG_PATH}${NC}"
    fi
}

# Cursor
CURSOR_DIR="${HOME}/.cursor"
if [ -d "${CURSOR_DIR}" ]; then
    register_editor "Cursor" "${CURSOR_DIR}/mcp.json"
fi

# Claude Desktop
CLAUDE_DIR="${HOME}/.config/Claude"
if [ -d "${CLAUDE_DIR}" ] || [ -d "${HOME}/snap/claude" ]; then
    register_editor "Claude Desktop" "${CLAUDE_DIR}/claude_desktop_config.json"
fi

# Windsurf
WINDSURF_DIR="${HOME}/.codeium/windsurf"
if [ -d "${WINDSURF_DIR}" ] || [ -d "${HOME}/.windsurf" ]; then
    register_editor "Windsurf" "${WINDSURF_DIR}/mcp_config.json"
fi

# VS Code / VS Code Insiders
VSCODE_DIRS=("${HOME}/.config/Code" "${HOME}/.config/Code - Insiders" "${HOME}/.vscode")
for VSDIR in "${VSCODE_DIRS[@]}"; do
    if [ -d "${VSDIR}" ]; then
        register_editor "VS Code (${VSDIR})" "${VSDIR}/User/globalStorage/anthropic.claude-mcp/mcp.json"
    fi
done

echo ""
echo "${GREEN}${BOLD}AETHER_02 installed successfully!${NC}"
echo "Binary: ${BINARY_PATH}"
echo "Config: ${ENV_FILE}"
echo ""
echo "${YELLOW}Restart your editor to activate AETHER_02 (12 tools).${NC}"
echo "${YELLOW}Check feature gates in .env before enabling dangerous operations.${NC}"
