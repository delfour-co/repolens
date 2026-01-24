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

Déclenchement : Automatique après que le workflow CI soit terminé avec succès sur `main`/`master`

**Fonctionnalités :**
- Se déclenche automatiquement après la réussite du workflow CI
- Ne refait **pas** les vérifications (check, fmt, clippy, test) - elles sont déjà faites dans CI
- Build automatique avec version nightly **uniquement si la CI passe**
- Création d'une release pre-release sur GitHub
- Artefacts : binaire, checksums, archives

**Jobs :**
- `build` : Build le binaire nightly (se déclenche seulement si CI réussit)

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

### 5. Sync Wiki (`sync-wiki.yml`)

Déclenchement : 
- Automatique lors d'un push sur `main`/`master` si des fichiers dans `wiki/` sont modifiés
- Automatique si `README.md`, `DEVELOPMENT.md` ou `CHANGELOG.md` sont modifiés
- Manuel via `workflow_dispatch`

**Fonctionnalités :**
- Synchronisation automatique des pages du dossier `wiki/` vers le WIKI GitHub
- Détection des changements et mise à jour uniquement si nécessaire
- Création automatique de nouvelles pages
- Commit et push automatiques vers le WIKI

**Prérequis :**
- Le WIKI doit être activé dans Settings > Features > Wikis du repository

**Utilisation :**
- Les modifications dans `wiki/` sont automatiquement synchronisées
- Pour forcer une synchronisation : Actions → Sync Wiki → Run workflow

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
- `contents: write` : Pour créer des tags, releases et pousser vers le WIKI
- `pull-requests: read` : Pour lire les informations des PRs

Ces permissions sont configurées dans chaque workflow.

## Notes importantes

### Synchronisation du WIKI

Le workflow `sync-wiki.yml` synchronise automatiquement le contenu du dossier `wiki/` vers le WIKI GitHub. Pour que cela fonctionne :

1. Activez le WIKI dans les paramètres du repository (Settings > Features > Wikis)
2. Les modifications dans `wiki/` seront automatiquement synchronisées lors d'un push
3. Le workflow utilise `GITHUB_TOKEN` qui a les permissions nécessaires par défaut
