#!/bin/bash
# Release script for RepoLens
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 0.2.0

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from argument or prompt
if [ $# -eq 0 ]; then
    echo -e "${YELLOW}No version specified.${NC}"
    CURRENT_VERSION=$(grep '^version =' Cargo.toml | cut -d'"' -f2 | cut -d'-' -f1)
    echo -e "Current version in Cargo.toml: ${GREEN}${CURRENT_VERSION}${NC}"
    read -p "Enter new version (e.g., 0.2.0): " VERSION
else
    VERSION=$1
fi

# Validate version format
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$'; then
    echo -e "${RED}Error: Invalid version format: ${VERSION}${NC}"
    echo "Version must follow semantic versioning (e.g., 0.2.0, 1.0.0, 2.1.3)"
    exit 1
fi

TAG="v${VERSION}"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag ${TAG} already exists${NC}"
    exit 1
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: Working directory is not clean${NC}"
    echo "Please commit or stash your changes before creating a release"
    exit 1
fi

# Check if we're on main/master branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    echo -e "${YELLOW}Warning: You're not on main/master branch (current: ${CURRENT_BRANCH})${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${GREEN}Preparing release ${VERSION}...${NC}"

# Update version in Cargo.toml
echo "Updating Cargo.toml..."
sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

# Verify the change
if ! grep -q "version = \"${VERSION}\"" Cargo.toml; then
    echo -e "${RED}Error: Failed to update version in Cargo.toml${NC}"
    exit 1
fi

# Run tests
echo "Running tests..."
cargo test --quiet || {
    echo -e "${RED}Error: Tests failed${NC}"
    exit 1
}

# Check formatting
echo "Checking code formatting..."
cargo fmt --all -- --check || {
    echo -e "${RED}Error: Code is not properly formatted${NC}"
    echo "Run 'cargo fmt --all' to fix"
    exit 1
}

# Run clippy
echo "Running clippy..."
cargo clippy -- -D warnings || {
    echo -e "${RED}Error: Clippy found issues${NC}"
    exit 1
}

# Commit changes
echo "Committing version change..."
git add Cargo.toml
git commit -m "chore: bump version to ${VERSION}"

# Create tag
echo "Creating tag ${TAG}..."
git tag -a "$TAG" -m "Release ${VERSION}"

# Show summary
echo ""
echo -e "${GREEN}✓ Release ${VERSION} prepared successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review the changes:"
echo "     git log HEAD~1..HEAD"
echo "     git show ${TAG}"
echo ""
echo "  2. Push the commit and tag:"
echo "     git push origin ${CURRENT_BRANCH}"
echo "     git push origin ${TAG}"
echo ""
echo "  3. The GitHub Actions workflow will automatically:"
echo "     - Build the release binary"
echo "     - Generate the CHANGELOG"
echo "     - Create a GitHub release"
echo ""
echo -e "${YELLOW}To push now, run:${NC}"
echo "  git push origin ${CURRENT_BRANCH} && git push origin ${TAG}"
