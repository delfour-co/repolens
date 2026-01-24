#!/bin/bash
# Generate CHANGELOG from git commits
# Usage: ./scripts/generate-changelog.sh [from_tag] [to_tag]
# Example: ./scripts/generate-changelog.sh v0.1.0 v0.2.0

set -euo pipefail

FROM_TAG=${1:-$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || git rev-list --max-parents=0 HEAD)}
TO_TAG=${2:-HEAD}

# Get commits between tags
COMMITS=$(git log --pretty=format:"%h|%s|%b" ${FROM_TAG}..${TO_TAG} 2>/dev/null || echo "")

if [ -z "$COMMITS" ]; then
    # No commits found, create empty entry
    if [ "$TO_TAG" = "HEAD" ]; then
        VERSION=$(grep '^version =' Cargo.toml | cut -d'"' -f2 | cut -d'-' -f1)
        DATE=$(date +%Y-%m-%d)
    else
        VERSION=${TO_TAG#v}
        DATE=$(git log -1 --format=%cd --date=format:%Y-%m-%d ${TO_TAG} 2>/dev/null || date +%Y-%m-%d)
    fi
    
    cat << EOF
## [${VERSION}] - ${DATE}

No changes in this release.

EOF
    echo "[${VERSION}]: https://github.com/$(git remote get-url origin 2>/dev/null | sed 's/.*github.com[:/]\(.*\)\.git/\1/' || echo 'delfour-co/cli--repolens')/releases/tag/v${VERSION}"
    exit 0
fi

# Categorize commits
FEATURES=()
FIXES=()
CHANGES=()
BREAKING=()
SECURITY=()
OTHER=()

while IFS='|' read -r hash subject body; do
    # Check for breaking changes in body
    is_breaking=false
    if echo "$body" | grep -qi "BREAKING CHANGE"; then
        is_breaking=true
    fi
    
    # Extract type and description from conventional commits
    if echo "$subject" | grep -qE '^(feat|fix|perf|refactor|chore|docs|style|test|build|ci|security)(\(.+\))?!?:'; then
        # Conventional commit format
        type=$(echo "$subject" | sed -E 's/^([^:!]+).*/\1/' | sed 's/!$//')
        desc=$(echo "$subject" | sed -E 's/^[^:!]+(\(.+\))?!?: *//')
        
        # Check for breaking change indicator
        if echo "$subject" | grep -q '!'; then
            is_breaking=true
        fi
        
        case "$type" in
            feat|feature)
                if [ "$is_breaking" = true ]; then
                    BREAKING+=("${desc} (#${hash})")
                else
                    FEATURES+=("${desc} (#${hash})")
                fi
                ;;
            fix|bugfix)
                FIXES+=("${desc} (#${hash})")
                ;;
            perf|refactor|chore|docs|style|test|build|ci)
                CHANGES+=("${desc} (#${hash})")
                ;;
            security)
                SECURITY+=("${desc} (#${hash})")
                ;;
            *)
                OTHER+=("${desc} (#${hash})")
                ;;
        esac
    else
        # Non-conventional commit
        OTHER+=("${subject} (#${hash})")
    fi
done <<< "$COMMITS"

# Get version and date
if [ "$TO_TAG" = "HEAD" ]; then
    VERSION=$(grep '^version =' Cargo.toml | cut -d'"' -f2 | cut -d'-' -f1)
    DATE=$(date +%Y-%m-%d)
else
    VERSION=${TO_TAG#v}
    DATE=$(git log -1 --format=%cd --date=format:%Y-%m-%d ${TO_TAG})
fi

# Generate CHANGELOG entry
cat << EOF
## [${VERSION}] - ${DATE}

EOF

if [ ${#BREAKING[@]} -gt 0 ]; then
    echo "### BREAKING CHANGES"
    for item in "${BREAKING[@]}"; do
        echo "- ${item}"
    done
    echo ""
fi

if [ ${#FEATURES[@]} -gt 0 ]; then
    echo "### Added"
    for item in "${FEATURES[@]}"; do
        echo "- ${item}"
    done
    echo ""
fi

if [ ${#FIXES[@]} -gt 0 ]; then
    echo "### Fixed"
    for item in "${FIXES[@]}"; do
        echo "- ${item}"
    done
    echo ""
fi

if [ ${#CHANGES[@]} -gt 0 ]; then
    echo "### Changed"
    for item in "${CHANGES[@]}"; do
        echo "- ${item}"
    done
    echo ""
fi

if [ ${#SECURITY[@]} -gt 0 ]; then
    echo "### Security"
    for item in "${SECURITY[@]}"; do
        echo "- ${item}"
    done
    echo ""
fi

if [ ${#OTHER[@]} -gt 0 ]; then
    echo "### Other"
    for item in "${OTHER[@]}"; do
        echo "- ${item}"
    done
    echo ""
fi

echo "[${VERSION}]: https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/releases/tag/v${VERSION}"
