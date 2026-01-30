#!/bin/bash
#
# Install git hooks for RepoLens
#
# This script copies the hooks from scripts/hooks/ to .git/hooks/
# Run this after cloning the repository.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_SOURCE="$SCRIPT_DIR/hooks"
HOOKS_TARGET="$SCRIPT_DIR/../.git/hooks"

echo -e "${YELLOW}Installing git hooks...${NC}"

# Check if we're in a git repository
if [ ! -d "$HOOKS_TARGET" ]; then
    echo -e "${RED}Error: .git/hooks directory not found${NC}"
    echo "Make sure you're running this from the repository root."
    exit 1
fi

# Install pre-commit hook
if [ -f "$HOOKS_SOURCE/pre-commit" ]; then
    cp "$HOOKS_SOURCE/pre-commit" "$HOOKS_TARGET/pre-commit"
    chmod +x "$HOOKS_TARGET/pre-commit"
    echo -e "${GREEN}Installed: pre-commit${NC}"
else
    echo -e "${YELLOW}Warning: pre-commit hook not found${NC}"
fi

# Install commit-msg hook
if [ -f "$HOOKS_SOURCE/commit-msg" ]; then
    cp "$HOOKS_SOURCE/commit-msg" "$HOOKS_TARGET/commit-msg"
    chmod +x "$HOOKS_TARGET/commit-msg"
    echo -e "${GREEN}Installed: commit-msg${NC}"
else
    echo -e "${YELLOW}Warning: commit-msg hook not found${NC}"
fi

echo -e "\n${GREEN}Git hooks installed successfully!${NC}"
echo ""
echo "The following checks will now run automatically:"
echo -e "  ${YELLOW}pre-commit:${NC}  cargo fmt --check, cargo clippy"
echo -e "  ${YELLOW}commit-msg:${NC}  Conventional Commits format validation"
echo ""
echo "To bypass hooks temporarily (not recommended):"
echo -e "  git commit ${YELLOW}--no-verify${NC} -m \"message\""
