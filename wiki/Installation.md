<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Installation

Ce guide vous explique comment installer RepoLens sur votre système.

## Prérequis

- **Rust** : Version stable (1.70+ recommandée)
- **Git** : Pour la gestion de version
- **GitHub CLI** (`gh`) : Optionnel, pour les fonctionnalités GitHub (installation via `gh auth login`)

## Installation depuis les sources

### 1. Cloner le repository

```bash
git clone https://github.com/delfour-co/cli--repolens.git
cd cli--repolens
```

### 2. Compiler le projet

```bash
# Compilation en mode release (recommandé)
cargo build --release

# Le binaire sera disponible à : target/release/repolens
```

### 3. Installer globalement (optionnel)

```bash
# Ajouter au PATH
sudo cp target/release/repolens /usr/local/bin/

# Ou créer un lien symbolique
sudo ln -s $(pwd)/target/release/repolens /usr/local/bin/repolens
```

### 4. Vérifier l'installation

```bash
repolens --help
```

## Installation de Rust

Si Rust n'est pas installé sur votre système :

```bash
# Installation via rustup (recommandé)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Suivre les instructions à l'écran
# Redémarrer le terminal après l'installation
```

Vérifier l'installation :

```bash
rustc --version
cargo --version
```

## Installation de GitHub CLI

Pour utiliser les fonctionnalités GitHub de RepoLens :

### Linux

```bash
# Via le gestionnaire de paquets
sudo apt install gh  # Debian/Ubuntu
sudo dnf install gh  # Fedora

# Ou via le script d'installation
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh
```

### macOS

```bash
brew install gh
```

### Authentification

```bash
gh auth login
```

## Installation via Cargo (futur)

Une fois publié sur crates.io :

```bash
cargo install repolens
```

## Dépannage

### Erreur de compilation

```bash
# Nettoyer et reconstruire
cargo clean
cargo build --release
```

### Problèmes avec les dépendances

```bash
# Mettre à jour les dépendances
cargo update

# Vérifier les versions
cargo tree
```

### Erreur "command not found"

Assurez-vous que le binaire est dans votre PATH :

```bash
# Vérifier le PATH
echo $PATH

# Ajouter manuellement si nécessaire
export PATH="$PATH:$(pwd)/target/release"
```

## Prochaines étapes

Une fois installé, consultez le [Guide d'utilisation](Guide-d-utilisation) pour commencer à utiliser RepoLens.
