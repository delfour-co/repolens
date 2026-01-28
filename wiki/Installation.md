<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Installation

Ce guide vous explique comment installer RepoLens sur votre système.

## Prérequis

- **Git** : Pour la gestion de version
- **GitHub CLI** (`gh`) : Optionnel, pour les fonctionnalités GitHub (installation via `gh auth login`)

## Installation via binaires pré-compilés (recommandé)

Des binaires pré-compilés sont disponibles pour toutes les plateformes majeures. Rendez-vous sur la [page Releases](https://github.com/delfour-co/cli--repolens/releases) pour télécharger la dernière version.

### Plateformes supportées

| Plateforme | Architecture | Archive |
|------------|-------------|---------|
| Linux | x86_64 | `repolens-linux-x86_64.tar.gz` |
| Linux | ARM64 | `repolens-linux-arm64.tar.gz` |
| macOS | Intel x86_64 | `repolens-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon ARM64 | `repolens-darwin-arm64.tar.gz` |
| Windows | x86_64 | `repolens-windows-x86_64.zip` |

### Linux (x86_64)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-linux-x86_64.tar.gz
tar xzf repolens-linux-x86_64.tar.gz
sudo mv repolens /usr/local/bin/
```

### Linux (ARM64)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-linux-arm64.tar.gz
tar xzf repolens-linux-arm64.tar.gz
sudo mv repolens /usr/local/bin/
```

### macOS (Apple Silicon)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-darwin-arm64.tar.gz
tar xzf repolens-darwin-arm64.tar.gz
sudo mv repolens /usr/local/bin/
```

### macOS (Intel)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-darwin-x86_64.tar.gz
tar xzf repolens-darwin-x86_64.tar.gz
sudo mv repolens /usr/local/bin/
```

### Windows (x86_64)

```powershell
# Telecharger l'archive depuis la page Releases
Invoke-WebRequest -Uri https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-windows-x86_64.zip -OutFile repolens-windows-x86_64.zip
Expand-Archive repolens-windows-x86_64.zip -DestinationPath .
Move-Item repolens.exe C:\Users\$env:USERNAME\bin\
```

### Verifier les checksums

Chaque release inclut un fichier `checksums.sha256` pour verifier l'integrite des archives :

```bash
# Telecharger le fichier de checksums
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/checksums.sha256

# Verifier (Linux)
sha256sum -c checksums.sha256 --ignore-missing

# Verifier (macOS)
shasum -a 256 -c checksums.sha256 --ignore-missing
```

### Verifier l'installation

```bash
repolens --version
```

## Installation depuis les sources

> **Note** : L'installation depuis les sources necessite **Rust** version stable (1.70+ recommandee).

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
