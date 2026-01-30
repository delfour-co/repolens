# Documentation WIKI

Ce dossier contient les pages Markdown qui seront synchronisées avec le WIKI GitHub du projet.

## Structure

Les fichiers Markdown dans ce dossier correspondent aux pages du WIKI GitHub :

- `Home.md` : Page d'accueil du WIKI
- `Installation.md` : Guide d'installation
- `Guide-d-utilisation.md` : Guide d'utilisation
- `Configuration.md` : Configuration avancée
- `Presets.md` : Documentation des presets
- `Categories-de-regles.md` : Catégories de règles
- `Bonnes-pratiques.md` : Bonnes pratiques
- `Developpement.md` : Guide de développement
- `Architecture.md` : Architecture technique
- `Contribution.md` : Guide de contribution

## Synchronisation

### Automatique (recommandé)

La synchronisation se fait **automatiquement via GitHub Actions** lors d'un push sur `main`/`master` si :
- Des fichiers dans `wiki/` sont modifiés
- `README.md`, `DEVELOPMENT.md` ou `CHANGELOG.md` sont modifiés

Le workflow `.github/workflows/sync-wiki.yml` gère cette synchronisation automatiquement.

### Manuelle (locale)

Pour synchroniser manuellement depuis votre machine locale, utilisez le script :

```bash
# Mettre à jour toutes les pages
./scripts/update-wiki.sh

# Mode dry-run (aperçu)
./scripts/update-wiki.sh --dry-run

# Mettre à jour une page spécifique
./scripts/update-wiki.sh Home.md

# Vérifier les différences
./scripts/update-wiki.sh --check

# Lister les pages disponibles
./scripts/update-wiki.sh --list
```

**Note** : En général, la synchronisation manuelle n'est pas nécessaire car la CI le fait automatiquement.

## Prérequis

- GitHub CLI (`gh`) installé et authentifié
- Le WIKI doit être activé dans les paramètres du repository GitHub
- Permissions d'écriture sur le repository

## Workflow recommandé

1. Modifier les fichiers Markdown dans ce dossier
2. Commit et push vers le repository
3. La CI synchronise automatiquement le WIKI lors du push

Pour une synchronisation manuelle locale (si nécessaire) :
1. Vérifier les changements : `./scripts/update-wiki.sh --check`
2. Prévisualiser : `./scripts/update-wiki.sh --dry-run`
3. Synchroniser : `./scripts/update-wiki.sh`

## Format

Les fichiers doivent être en Markdown (.md) et suivre les conventions GitHub Flavored Markdown.

## Liens

Les liens entre pages doivent utiliser le format relatif :
- `[Installation](Installation)` au lieu de `[Installation](Installation.md)`
- GitHub WIKI convertit automatiquement les noms de fichiers en liens

## Notes

- Les modifications locales ne sont pas automatiquement synchronisées
- Utilisez toujours `--dry-run` avant de synchroniser pour vérifier
- Le script clone le WIKI dans un dossier temporaire, fait les modifications, puis push
