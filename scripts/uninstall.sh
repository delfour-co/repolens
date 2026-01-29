#!/bin/bash

# Script de désinstallation de RepoLens
# Supprime proprement le binaire repolens du système

set -euo pipefail

# Couleurs pour les messages
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="repolens"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

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

# Trouver tous les emplacements où le binaire pourrait être installé
find_installations() {
    local locations=()
    
    # Emplacements communs
    local common_paths=(
        "/usr/local/bin"
        "/usr/bin"
        "$HOME/.local/bin"
        "$HOME/.cargo/bin"
    )
    
    # Vérifier chaque emplacement
    for path in "${common_paths[@]}"; do
        if [ -f "${path}/${BINARY_NAME}" ]; then
            locations+=("${path}/${BINARY_NAME}")
        fi
    fi
    
    # Vérifier aussi dans le PATH
    if command -v "$BINARY_NAME" &> /dev/null; then
        local path_location=$(command -v "$BINARY_NAME")
        if [[ ! " ${locations[@]} " =~ " ${path_location} " ]]; then
            locations+=("$path_location")
        fi
    fi
    
    echo "${locations[@]}"
}

# Vérifier les permissions
check_permissions() {
    if [ ! -w "$(dirname "$BINARY_PATH")" ]; then
        error "Permissions insuffisantes pour supprimer ${BINARY_PATH}"
        info "Essayez de lancer le script avec sudo:"
        echo "  sudo $0"
        exit 1
    fi
}

# Désinstaller le binaire
uninstall_binary() {
    local target="$1"
    
    if [ ! -f "$target" ]; then
        warn "Le fichier ${target} n'existe pas"
        return 1
    fi
    
    # Vérifier que c'est bien le bon binaire (optionnel mais recommandé)
    if ! file "$target" | grep -q "executable"; then
        warn "Le fichier ${target} ne semble pas être un exécutable"
        read -p "Voulez-vous quand même le supprimer? (o/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Oo]$ ]]; then
            info "Suppression annulée"
            return 1
        fi
    fi
    
    info "Suppression de ${target}..."
    rm -f "$target"
    
    if [ ! -f "$target" ]; then
        info "✓ Fichier supprimé avec succès"
        return 0
    else
        error "Échec de la suppression de ${target}"
        return 1
    fi
}

# Vérifier s'il reste des installations
check_remaining_installations() {
    local remaining=$(find_installations)
    
    if [ -z "$remaining" ]; then
        info "✓ Aucune installation restante trouvée"
        return 0
    else
        warn "Installations restantes trouvées:"
        for loc in $remaining; do
            echo "  - $loc"
        done
        return 1
    fi
}

# Nettoyer les fichiers de configuration utilisateur (optionnel)
cleanup_user_config() {
    local config_files=(
        "$HOME/.repolens.toml"
        "$HOME/.config/repolens"
    )
    
    local found=false
    for config in "${config_files[@]}"; do
        if [ -e "$config" ]; then
            found=true
            break
        fi
    done
    
    if [ "$found" = true ]; then
        warn "Fichiers de configuration utilisateur trouvés:"
        for config in "${config_files[@]}"; do
            if [ -e "$config" ]; then
                echo "  - $config"
            fi
        done
        
        read -p "Voulez-vous les supprimer également? (o/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Oo]$ ]]; then
            for config in "${config_files[@]}"; do
                if [ -e "$config" ]; then
                    rm -rf "$config"
                    info "✓ Supprimé: $config"
                fi
            done
        fi
    fi
}

# Fonction principale
main() {
    info "=== Désinstallation de RepoLens ==="
    echo
    
    # Trouver toutes les installations
    local installations=$(find_installations)
    
    if [ -z "$installations" ]; then
        warn "Aucune installation de ${BINARY_NAME} trouvée"
        info "Le binaire n'est peut-être pas installé ou n'est pas dans le PATH"
        exit 0
    fi
    
    info "Installations trouvées:"
    for inst in $installations; do
        echo "  - $inst"
    done
    echo
    
    # Demander confirmation
    read -p "Voulez-vous supprimer ces installations? (o/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Oo]$ ]]; then
        info "Désinstallation annulée"
        exit 0
    fi
    
    # Supprimer chaque installation
    local success=true
    for inst in $installations; do
        INSTALL_DIR=$(dirname "$inst")
        BINARY_PATH="$inst"
        
        check_permissions
        
        if ! uninstall_binary "$inst"; then
            success=false
        fi
        echo
    done
    
    if [ "$success" = true ]; then
        echo
        check_remaining_installations
        echo
        
        cleanup_user_config
        echo
        
        info "=== Désinstallation terminée ==="
    else
        error "Certaines installations n'ont pas pu être supprimées"
        exit 1
    fi
}

# Gestion des options
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir=*)
            INSTALL_DIR="${1#*=}"
            BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
            shift
            ;;
        --force|-f)
            # Mode non-interactif (pour les scripts)
            FORCE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir=DIR    Répertoire d'installation spécifique à désinstaller"
            echo "  --force, -f          Mode non-interactif (supprime sans confirmation)"
            echo "  --help, -h           Affiche cette aide"
            echo ""
            echo "Exemples:"
            echo "  sudo $0                                    # Désinstallation système"
            echo "  $0 --install-dir=\$HOME/.local/bin        # Désinstallation utilisateur"
            echo "  sudo $0 --force                           # Désinstallation sans confirmation"
            exit 0
            ;;
        *)
            error "Option inconnue: $1"
            echo "Utilisez --help pour voir les options disponibles"
            exit 1
            ;;
    esac
done

# Mode non-interactif si --force est utilisé
if [ "${FORCE:-false}" = true ]; then
    # Redéfinir read pour ne pas demander de confirmation
    read() {
        REPLY="o"
    }
fi

# Lancer la désinstallation
main
