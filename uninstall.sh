#!/usr/bin/env bash
set -euo pipefail

# Coalbox Uninstaller
# Removes coalbox binaries from the install directory.
# Does NOT remove your vault — that's your data, we don't touch it.

INSTALL_DIR="${COALBOX_INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}▸${NC} $*"; }
warn()  { echo -e "${YELLOW}▸${NC} $*"; }
error() { echo -e "${RED}▸${NC} $*" >&2; }

echo ""
echo -e "  ${CYAN}Coalbox Uninstaller${NC}"
echo ""

# Remove binaries
for bin in coalbox coalbox-web; do
    target="${INSTALL_DIR}/${bin}"
    if [ -f "$target" ] || [ -L "$target" ]; then
        if [ -w "$INSTALL_DIR" ]; then
            rm "$target"
        else
            sudo rm "$target"
        fi
        info "Removed ${target}"
    fi
done

# Check if anything was removed
if ! command -v coalbox &>/dev/null; then
    info "Coalbox uninstalled successfully"
else
    warn "Coalbox is still available at $(which coalbox)"
fi

echo ""
info "Vault files were NOT removed. Your data is safe."
info "To delete your vault manually:"
info "  rm ~/.local/share/coalbox/vault.emberkeys"
echo ""
