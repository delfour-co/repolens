#!/bin/bash

# Script d'installation de RepoLens
# Build et installe le binaire repolens dans le système

set -euo pipefail

# Couleurs pour les messages
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="repolens"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release"
BINARY_PATH="${BUILD_DIR}/${BINARY_NAME}"

# Fonction pour afficher les messages
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Vérifier si on est root pour l'installation système
check_permissions() {
    if [ ! -w "$INSTALL_DIR" ]; then
        error "Permissions insuffisantes pour écrire dans ${INSTALL_DIR}"
        info "Essayez de lancer le script avec sudo:"
        echo "  sudo $0"
        exit 1
    fi
}

# Vérifier les prérequis
check_prerequisites() {
    info "Vérification des prérequis..."
    
    if ! command -v cargo &> /dev/null; then
        error "Cargo n'est pas installé"
        info "Installez Rust depuis: https://rustup.rs/"
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        error "Rust n'est pas installé"
        info "Installez Rust depuis: https://rustup.rs/"
        exit 1
    fi
    
    info "✓ Cargo trouvé: $(cargo --version)"
    info "✓ Rust trouvé: $(rustc --version)"
}

# Builder le projet
build_project() {
    info "Construction du projet en mode release..."
    
    cd "$PROJECT_ROOT"
    
    if [ ! -f "Cargo.toml" ]; then
        error "Cargo.toml introuvable dans ${PROJECT_ROOT}"
        exit 1
    fi
    
    # Builder en mode release
    cargo build --release
    
    if [ ! -f "$BINARY_PATH" ]; then
        error "Le binaire n'a pas été généré à ${BINARY_PATH}"
        exit 1
    fi
    
    info "✓ Build terminé avec succès"
}

# Installer le binaire
install_binary() {
    info "Installation du binaire dans ${INSTALL_DIR}..."
    
    # Vérifier si le binaire existe déjà
    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        warn "Le binaire existe déjà dans ${INSTALL_DIR}"
        read -p "Voulez-vous le remplacer? (o/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Oo]$ ]]; then
            info "Installation annulée"
            exit 0
        fi
        info "Remplacement du binaire existant..."
    fi
    
    # Copier le binaire
    cp "$BINARY_PATH" "${INSTALL_DIR}/${BINARY_NAME}"
    
    # Rendre exécutable
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    info "✓ Binaire installé avec succès"
}

# Vérifier l'installation
verify_installation() {
    info "Vérification de l'installation..."
    
    if command -v "$BINARY_NAME" &> /dev/null; then
        INSTALLED_VERSION=$("$BINARY_NAME" --version 2>/dev/null || echo "version inconnue")
        info "✓ Installation réussie!"
        info "  Binaire: $(command -v ${BINARY_NAME})"
        info "  Version: ${INSTALLED_VERSION}"
    else
        warn "Le binaire n'est pas dans le PATH"
        warn "Assurez-vous que ${INSTALL_DIR} est dans votre PATH"
    fi
}

# Fonction principale
main() {
    info "=== Installation de RepoLens ==="
    echo
    
    check_permissions
    check_prerequisites
    echo
    
    build_project
    echo
    
    install_binary
    echo
    
    verify_installation
    echo
    
    info "=== Installation terminée ==="
    info "Vous pouvez maintenant utiliser: ${BINARY_NAME} --help"
}

# Gestion des options
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir=*)
            INSTALL_DIR="${1#*=}"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir=DIR    Répertoire d'installation (défaut: /usr/local/bin)"
            echo "  --help, -h           Affiche cette aide"
            echo ""
            echo "Exemples:"
            echo "  sudo $0                                    # Installation système"
            echo "  INSTALL_DIR=\$HOME/.local/bin $0          # Installation utilisateur"
            echo "  $0 --install-dir=\$HOME/.local/bin        # Installation utilisateur"
            exit 0
            ;;
        *)
            error "Option inconnue: $1"
            echo "Utilisez --help pour voir les options disponibles"
            exit 1
            ;;
    esac
done

# Lancer l'installation
main
