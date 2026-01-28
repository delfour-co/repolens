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

## Installation via crates.io

RepoLens est publie sur [crates.io](https://crates.io/crates/repolens). C'est la methode d'installation la plus simple si vous avez deja Rust installe :

```bash
cargo install repolens
```

Pour mettre a jour vers la derniere version :

```bash
cargo install repolens --force
```

## Installation via binaires pre-compiles

Des binaires pre-compiles sont disponibles pour les plateformes suivantes sur la [page des releases](https://github.com/kdelfour/repolens/releases) :

| Plateforme | Architecture | Fichier |
|---|---|---|
| Linux | x86_64 | `repolens-linux-x86_64.tar.gz` |
| Linux | ARM64 | `repolens-linux-aarch64.tar.gz` |
| macOS | Intel (x86_64) | `repolens-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon (ARM64) | `repolens-darwin-aarch64.tar.gz` |
| Windows | x86_64 | `repolens-windows-x86_64.zip` |

### Linux / macOS

```bash
# Telecharger et extraire (exemple pour Linux x86_64)
curl -L https://github.com/kdelfour/repolens/releases/latest/download/repolens-linux-x86_64.tar.gz | tar xz

# Rendre executable et deplacer dans le PATH
chmod +x repolens
sudo mv repolens /usr/local/bin/
```

### Windows

1. Telecharger `repolens-windows-x86_64.zip` depuis la [page des releases](https://github.com/kdelfour/repolens/releases)
2. Extraire l'archive
3. Ajouter le dossier contenant `repolens.exe` a votre variable d'environnement `PATH`

## Utilisation via GitHub Action

RepoLens est disponible en tant qu'Action GitHub officielle pour integrer l'audit directement dans vos workflows CI/CD.

### Utilisation basique

```yaml
name: RepoLens Audit
on: [push, pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: kdelfour/repolens-action@v1
        with:
          preset: opensource
```

### Inputs disponibles

| Input | Description | Defaut |
|---|---|---|
| `preset` | Preset de configuration (`opensource`, `enterprise`, `strict`) | `opensource` |
| `format` | Format de sortie (`terminal`, `json`, `sarif`, `markdown`, `html`) | `terminal` |
| `output` | Chemin du fichier de sortie | - |
| `categories` | Categories a auditer (separees par des virgules) | toutes |
| `exclude` | Categories a exclure (separees par des virgules) | - |
| `verbose` | Niveau de verbosite (`0`-`3`) | `0` |
| `fail-on-error` | Echouer le workflow si des problemes sont detectes | `false` |

### Outputs disponibles

| Output | Description |
|---|---|
| `score` | Score global de l'audit |
| `report-path` | Chemin du rapport genere |
| `issues-count` | Nombre de problemes detectes |

### Exemple avance avec publication SARIF

```yaml
name: RepoLens Security Audit
on: [push]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: kdelfour/repolens-action@v1
        id: audit
        with:
          preset: strict
          format: sarif
          output: repolens-results.sarif
          fail-on-error: true
      - uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: repolens-results.sarif
```

### Exemple audit multi-presets

```yaml
name: RepoLens Multi-Preset Audit
on: [pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        preset: [opensource, enterprise, strict]
    steps:
      - uses: actions/checkout@v4
      - uses: kdelfour/repolens-action@v1
        with:
          preset: ${{ matrix.preset }}
          format: markdown
          output: report-${{ matrix.preset }}.md
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
