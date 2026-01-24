# GitHub Actions Workflows

Ce répertoire contient les workflows GitHub Actions pour RepoLens.

## Workflows disponibles

### 1. CI (`ci.yml`)

Déclenchement : À chaque push et pull request

**Jobs :**
- `check` : Vérifie que le code compile
- `fmt` : Vérifie le formatage du code
- `clippy` : Analyse statique avec clippy
- `test` : Exécute les tests

### 2. Nightly Build (`nightly.yml`)

Déclenchement : À chaque push sur `main`/`master` (sans tag)

**Fonctionnalités :**
- Build automatique avec version nightly
- Création d'une release pre-release sur GitHub
- Artefacts : binaire, checksums, archives

### 3. Create Release (`create-release.yml`)

Déclenchement : Manuel via `workflow_dispatch`

**Fonctionnalités :**
- Auto-incrémentation de version (patch, minor, major)
- Mise à jour automatique de `Cargo.toml`
- Exécution des tests et vérifications
- Création et push automatique du tag
- Déclenche automatiquement le workflow `release.yml`

**Utilisation :**
1. Aller sur Actions → Create Release
2. Cliquer sur "Run workflow"
3. Choisir le type d'incrémentation
4. Cocher "Create and push tag automatically"
5. Lancer le workflow

### 4. Release (`release.yml`)

Déclenchement : Automatique lors du push d'un tag `v*.*.*`

**Fonctionnalités :**
- Build du binaire optimisé
- Génération automatique du CHANGELOG
- Création de la release GitHub avec artefacts
- Mise à jour du CHANGELOG.md dans le dépôt

## Flux de release

```
1. Déclencher "Create Release" depuis Actions
   ↓
2. Calcul de la nouvelle version (auto-incrémentation)
   ↓
3. Mise à jour de Cargo.toml
   ↓
4. Tests et vérifications
   ↓
5. Commit et push du tag
   ↓
6. Déclenchement automatique de "Release"
   ↓
7. Build et publication de la release
```

## Permissions requises

Les workflows nécessitent les permissions suivantes :
- `contents: write` : Pour créer des tags et releases
- `pull-requests: read` : Pour lire les informations des PRs

Ces permissions sont configurées dans chaque workflow.
