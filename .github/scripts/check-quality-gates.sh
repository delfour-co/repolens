#!/bin/bash
# Script de vérification des seuils de qualité
# Usage: ./.github/scripts/check-quality-gates.sh [coverage_file] [clippy_file] [audit_file]

set -euo pipefail

# Couleurs pour l'output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Fichier de configuration des seuils
QUALITY_GATES_FILE=".github/quality-gates.toml"

# Vérifier que le fichier de configuration existe
if [ ! -f "$QUALITY_GATES_FILE" ]; then
    echo -e "${RED}❌ Fichier de configuration des seuils introuvable: $QUALITY_GATES_FILE${NC}"
    exit 1
fi

# Fonction pour extraire une valeur depuis le fichier TOML
extract_toml_value() {
    local section=$1
    local key=$2
    local default=$3
    grep -A 20 "\[$section\]" "$QUALITY_GATES_FILE" | grep "^$key" | head -1 | sed -E "s/.*= *([^#]*).*/\\1/" | tr -d ' ",' || echo "$default"
}

# Fonction pour comparer des nombres décimaux
compare_float() {
    awk "BEGIN {exit !($1 $2 $3)}"
}

# Fonction pour afficher un résultat de vérification
check_result() {
    local name=$1
    local passed=$2
    local message=$3
    
    if [ "$passed" = "true" ]; then
        echo -e "${GREEN}✅ $name: $message${NC}"
        return 0
    else
        echo -e "${RED}❌ $name: $message${NC}"
        return 1
    fi
}

echo "🔍 Vérification des seuils de qualité..."
echo ""

FAILED_CHECKS=0

# 1. Vérifier la couverture de code
COVERAGE_FILE="${1:-coverage/cobertura.xml}"
MIN_COVERAGE=$(extract_toml_value "coverage" "minimum" "80.0")

if [ -f "$COVERAGE_FILE" ]; then
    COVERAGE=""

    # Method 1: Extract line-rate from root <coverage> element (standard cobertura format)
    LINE_RATE=$(grep -oP '<coverage[^>]+line-rate="\K[0-9.]+' "$COVERAGE_FILE" 2>/dev/null | head -1 || true)
    if [ -n "$LINE_RATE" ] && [ "$LINE_RATE" != "0" ]; then
        COVERAGE=$(awk "BEGIN {printf \"%.2f\", $LINE_RATE * 100}")
    fi

    # Method 2: Use lines-valid and lines-covered attributes via xmllint
    if [ -z "$COVERAGE" ] && command -v xmllint &> /dev/null; then
        TOTAL_LINES=$(xmllint --xpath "string(//coverage/@lines-valid)" "$COVERAGE_FILE" 2>/dev/null || true)
        COVERED_LINES=$(xmllint --xpath "string(//coverage/@lines-covered)" "$COVERAGE_FILE" 2>/dev/null || true)

        if [ -n "$TOTAL_LINES" ] && [ "$TOTAL_LINES" != "0" ] && [ -n "$COVERED_LINES" ]; then
            COVERAGE=$(awk "BEGIN {printf \"%.2f\", ($COVERED_LINES / $TOTAL_LINES) * 100}")
        fi
    fi

    # Evaluate result
    if [ -n "$COVERAGE" ]; then
        if compare_float "$COVERAGE" ">=" "$MIN_COVERAGE"; then
            check_result "Couverture de code" true "$COVERAGE% (minimum: $MIN_COVERAGE%)"
        else
            check_result "Couverture de code" false "$COVERAGE% (minimum requis: $MIN_COVERAGE%)"
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
        fi
    else
        echo -e "${YELLOW}⚠️  Impossible de calculer la couverture depuis le fichier XML${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Fichier de couverture non trouvé ($COVERAGE_FILE), vérification ignorée${NC}"
    echo -e "${YELLOW}   Pour générer la couverture: cargo tarpaulin --out Xml --output-dir coverage${NC}"
fi

# 2. Vérifier Clippy
MAX_WARNINGS=$(extract_toml_value "clippy" "max_warnings" "0")
CLIPPY_FILE="${2:-clippy.json}"

if [ -f "$CLIPPY_FILE" ]; then
    CLIPPY_WARNINGS=$(grep -c '"level":"warning"' "$CLIPPY_FILE" || echo "0")
else
    # Générer le fichier Clippy si absent
    cargo clippy --all-targets --all-features --message-format=json > "$CLIPPY_FILE" 2>&1 || true
    CLIPPY_WARNINGS=$(grep -c '"level":"warning"' "$CLIPPY_FILE" || echo "0")
fi

if [ "$CLIPPY_WARNINGS" -le "$MAX_WARNINGS" ]; then
    check_result "Clippy warnings" true "$CLIPPY_WARNINGS warnings (maximum: $MAX_WARNINGS)"
else
    check_result "Clippy warnings" false "$CLIPPY_WARNINGS warnings (maximum autorisé: $MAX_WARNINGS)"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# 3. Vérifier les vulnérabilités de sécurité
AUDIT_FILE="${3:-audit.json}"
MAX_CRITICAL=$(extract_toml_value "security" "max_critical_vulnerabilities" "0")
MAX_HIGH=$(extract_toml_value "security" "max_high_vulnerabilities" "0")

if command -v cargo-audit &> /dev/null; then
    if [ -f "$AUDIT_FILE" ]; then
        AUDIT_OUTPUT=$(cat "$AUDIT_FILE")
    else
        # Générer le fichier d'audit si absent
        cargo audit --json > "$AUDIT_FILE" 2>&1 || true
        AUDIT_OUTPUT=$(cat "$AUDIT_FILE")
    fi
    
    # Compter les vulnérabilités
    CRITICAL_COUNT=$(echo "$AUDIT_OUTPUT" | grep -c '"severity":"critical"' || echo "0")
    HIGH_COUNT=$(echo "$AUDIT_OUTPUT" | grep -c '"severity":"high"' || echo "0")
    
    if [ "$CRITICAL_COUNT" -le "$MAX_CRITICAL" ] && [ "$HIGH_COUNT" -le "$MAX_HIGH" ]; then
        check_result "Vulnérabilités de sécurité" true "Critiques: $CRITICAL_COUNT, Importantes: $HIGH_COUNT"
    else
        check_result "Vulnérabilités de sécurité" false "Critiques: $CRITICAL_COUNT (max: $MAX_CRITICAL), Importantes: $HIGH_COUNT (max: $MAX_HIGH)"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
else
    echo -e "${YELLOW}⚠️  cargo-audit non disponible, vérification de sécurité ignorée${NC}"
fi

# 4. Vérifier les dépendances obsolètes
OUTDATED_FILE="${4:-outdated.json}"
MAX_OUTDATED=$(extract_toml_value "dependencies" "max_outdated" "5")

if command -v cargo-outdated &> /dev/null; then
    if [ -f "$OUTDATED_FILE" ]; then
        OUTDATED_COUNT=$(grep -c '"name"' "$OUTDATED_FILE" || echo "0")
    else
        # Générer le fichier outdated si absent
        cargo outdated --format json > "$OUTDATED_FILE" 2>&1 || true
        OUTDATED_COUNT=$(grep -c '"name"' "$OUTDATED_FILE" || echo "0")
    fi
    
    if [ "$OUTDATED_COUNT" -le "$MAX_OUTDATED" ]; then
        check_result "Dépendances obsolètes" true "$OUTDATED_COUNT dépendances (maximum: $MAX_OUTDATED)"
    else
        check_result "Dépendances obsolètes" false "$OUTDATED_COUNT dépendances (maximum autorisé: $MAX_OUTDATED)"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
else
    echo -e "${YELLOW}⚠️  cargo-outdated non disponible, vérification ignorée${NC}"
fi

# 5. Vérifier le nombre de tests
MIN_TESTS=$(extract_toml_value "code_metrics" "min_tests" "20")
TEST_COUNT=$(cargo test --all-features --lib --tests --no-run --message-format=json 2>&1 | grep -c '"type":"test"' || echo "0")

if [ "$TEST_COUNT" -ge "$MIN_TESTS" ]; then
    check_result "Nombre de tests" true "$TEST_COUNT tests (minimum: $MIN_TESTS)"
else
    check_result "Nombre de tests" false "$TEST_COUNT tests (minimum requis: $MIN_TESTS)"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# 6. Vérifier la taille du binaire (si disponible)
MAX_BINARY_SIZE=$(extract_toml_value "code_metrics" "max_binary_size" "10000000")
if [ -f "target/release/repolens" ]; then
    BINARY_SIZE=$(stat -c%s target/release/repolens 2>/dev/null || stat -f%z target/release/repolens 2>/dev/null || echo "0")
    if [ "$MAX_BINARY_SIZE" != "0" ] && [ "$BINARY_SIZE" -gt "$MAX_BINARY_SIZE" ]; then
        check_result "Taille du binaire" false "${BINARY_SIZE} bytes (maximum: ${MAX_BINARY_SIZE} bytes)"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    else
        check_result "Taille du binaire" true "${BINARY_SIZE} bytes (maximum: ${MAX_BINARY_SIZE} bytes)"
    fi
fi

echo ""
if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✅ Tous les seuils de qualité sont respectés !${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_CHECKS seuil(s) de qualité non respecté(s)${NC}"
    echo -e "${RED}La nightly build ne peut pas être créée.${NC}"
    exit 1
fi
