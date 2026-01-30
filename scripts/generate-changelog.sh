#!/usr/bin/env bash
set -euo pipefail

# Generate a changelog entry between two git refs in Keep a Changelog format.
# Usage: generate-changelog.sh <from-ref> <to-ref>
#
# Categorises commits using Conventional Commits prefixes:
#   feat     -> Added
#   fix      -> Fixed
#   docs     -> Documentation
#   security -> Security
#   perf     -> Changed
#   refactor -> Changed
#   BREAKING -> Removed / Changed (breaking)
#   chore/ci/test/style/build -> skipped by default

FROM_REF="${1:?Usage: generate-changelog.sh <from-ref> <to-ref>}"
TO_REF="${2:?Usage: generate-changelog.sh <from-ref> <to-ref>}"

# Extract version number from tag (strip leading 'v')
VERSION="${TO_REF#v}"
DATE=$(date -u '+%Y-%m-%d')

# Collect commits between the two refs (subject only, one per line)
COMMITS=$(git log "${FROM_REF}..${TO_REF}" --pretty=format:"%s" --no-merges 2>/dev/null || true)

if [ -z "$COMMITS" ]; then
  # Fallback: output a minimal entry
  echo "## [${VERSION}] - ${DATE}"
  echo ""
  echo "### Changed"
  echo ""
  echo "- Release ${VERSION}"
  exit 0
fi

# Associative arrays for each category
declare -a ADDED=()
declare -a FIXED=()
declare -a CHANGED=()
declare -a SECURITY=()
declare -a DOCS=()
declare -a REMOVED=()
declare -a DEPRECATED=()

# Regex for conventional commit parsing (stored in variable for bash compatibility)
COMMIT_RE='^([a-zA-Z]+)(\([^)]*\))?(!)?\: (.+)$'

while IFS= read -r line; do
  [ -z "$line" ] && continue

  # Strip conventional commit prefix and optional scope
  # Pattern: type(scope)!: description  or  type!: description  or  type: description
  if [[ "$line" =~ $COMMIT_RE ]]; then
    type="${BASH_REMATCH[1]}"
    breaking="${BASH_REMATCH[3]}"
    desc="${BASH_REMATCH[4]}"
  else
    # Non-conventional commit — put in Changed
    desc="$line"
    type="other"
    breaking=""
  fi

  # Capitalise first letter of description
  desc="$(echo "${desc:0:1}" | tr '[:lower:]' '[:upper:]')${desc:1}"

  # Handle breaking changes
  if [ "$breaking" = "!" ]; then
    CHANGED+=("$desc (BREAKING)")
    continue
  fi

  case "$type" in
    feat)       ADDED+=("$desc") ;;
    fix)        FIXED+=("$desc") ;;
    docs)       DOCS+=("$desc") ;;
    security)   SECURITY+=("$desc") ;;
    perf)       CHANGED+=("$desc") ;;
    refactor)   CHANGED+=("$desc") ;;
    revert)     REMOVED+=("$desc") ;;
    deprecate)  DEPRECATED+=("$desc") ;;
    # Skip chore, ci, test, style, build — they don't belong in user-facing changelogs
    chore|ci|test|style|build) ;;
    *)          CHANGED+=("$desc") ;;
  esac
done <<< "$COMMITS"

# --- Output ---

echo "## [${VERSION}] - ${DATE}"
echo ""

print_section() {
  local title="$1"
  shift
  local items=("$@")
  if [ ${#items[@]} -gt 0 ]; then
    echo "### ${title}"
    echo ""
    for item in "${items[@]}"; do
      echo "- ${item}"
    done
    echo ""
  fi
}

print_section "Added"         "${ADDED[@]+"${ADDED[@]}"}"
print_section "Changed"       "${CHANGED[@]+"${CHANGED[@]}"}"
print_section "Deprecated"    "${DEPRECATED[@]+"${DEPRECATED[@]}"}"
print_section "Removed"       "${REMOVED[@]+"${REMOVED[@]}"}"
print_section "Fixed"         "${FIXED[@]+"${FIXED[@]}"}"
print_section "Security"      "${SECURITY[@]+"${SECURITY[@]}"}"
print_section "Documentation" "${DOCS[@]+"${DOCS[@]}"}"
