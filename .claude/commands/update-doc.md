# Update Documentation Command

Met à jour la documentation du projet et synchronise avec le WIKI GitHub.

## Objectif

Cet agent parcourt la documentation du projet, analyse les changements, et met à jour automatiquement :
- Les fichiers de documentation locaux (README.md, DEVELOPMENT.md, etc.)
- Le WIKI GitHub avec les pages appropriées
- La cohérence entre les différentes sources de documentation
- Les badges de version sur les pages wiki

## Étapes

1. **Analyser la documentation existante**
   - Lire les fichiers de documentation principaux :
     - `README.md`
     - `DEVELOPMENT.md`
     - `CHANGELOG.md`
     - `CLAUDE.md`
     - Fichiers dans `wiki/` (pages WIKI locales)
   - Identifier les sections qui ont changé ou qui sont obsolètes

2. **Vérifier la cohérence**
   - Comparer le contenu entre les différentes sources
   - Identifier les incohérences
   - Détecter les informations manquantes

3. **Mettre à jour la documentation**
   - Synchroniser les informations entre les fichiers
   - Mettre à jour les exemples de code
   - Corriger les liens et références
   - Ajouter les nouvelles fonctionnalités documentées dans le code

4. **Mettre à jour le WIKI GitHub**
   - La synchronisation se fait automatiquement via GitHub Actions lors d'un push
   - Le workflow `.github/workflows/sync-wiki.yml` synchronise automatiquement les pages
   - Pour une synchronisation manuelle locale, utiliser `scripts/update-wiki.sh`
   - Vérifier que toutes les pages sont à jour
   - S'assurer que les liens entre pages fonctionnent

5. **Vérifier les résultats**
   - Lancer une vérification finale
   - S'assurer que tous les fichiers sont cohérents

## Synchronisation automatique

La synchronisation du WIKI se fait **automatiquement via GitHub Actions** après :
- Un build Nightly réussi → Badge "nightly" (orange)
- Une Release réussie → Badge "stable" (vert)

Le workflow `.github/workflows/sync-wiki.yml` gère cette synchronisation.

### Déclencheurs
- `workflow_run` : Après "Nightly Build" ou "Release" (succès uniquement)
- `workflow_dispatch` : Déclenchement manuel avec choix du type de version

## Système de badges

Chaque page wiki inclut un header automatique avec des badges :

```markdown
<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-{type}-{color})
![Updated](https://img.shields.io/badge/updated-{date}-blue)

---
```

### Types de badges

| Type | Badge | Couleur | Déclencheur |
|------|-------|---------|-------------|
| Local | `version-local-gray` | Gris | Fichiers locaux |
| Nightly | `version-nightly-orange` | Orange | Workflow Nightly Build |
| Stable | `version-stable-green` | Vert | Workflow Release |

### Fonctionnement

1. **Fichiers locaux** (`wiki/`) : Badge "local" gris
2. **Synchronisation CI** : Le workflow remplace automatiquement le header avec :
   - Le type de version (nightly/stable) basé sur le workflow déclencheur
   - La date de mise à jour
3. **Wiki GitHub** : Affiche le badge correspondant à la dernière synchronisation

### Bonnes pratiques pour les badges

- Ne **jamais** modifier manuellement le header (entre `<!-- Auto-generated header` et `---`)
- Le badge "local" indique que la documentation n'a pas encore été synchronisée
- Le badge "nightly" indique une version en développement
- Le badge "stable" indique une version release

## Commandes utiles (pour synchronisation manuelle locale)

```bash
# Lancer la mise à jour complète (local uniquement)
./scripts/update-wiki.sh

# Mettre à jour une page spécifique
./scripts/update-wiki.sh Home.md

# Vérifier les différences avant mise à jour
./scripts/update-wiki.sh --dry-run

# Vérifier les différences sans mettre à jour
./scripts/update-wiki.sh --check
```

**Note** : En général, il n'est pas nécessaire d'utiliser le script manuellement car la CI synchronise automatiquement.

## Structure de la documentation

### Fichiers locaux
- `README.md` : Documentation principale pour les utilisateurs
- `DEVELOPMENT.md` : Guide de développement
- `CHANGELOG.md` : Historique des changements
- `CLAUDE.md` : Contexte pour Claude AI

### Pages WIKI (dans `wiki/`)
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

## Bonnes pratiques

- Toujours vérifier la cohérence entre les sources
- Mettre à jour les exemples de code si l'API change
- Vérifier que les liens fonctionnent
- Maintenir la structure et le formatage
- Ajouter des sections manquantes si nécessaire

## Détection automatique

L'agent doit détecter :
- Nouvelles fonctionnalités dans le code non documentées
- Changements dans l'API ou les commandes CLI
- Fichiers de configuration ou presets modifiés
- Nouvelles catégories de règles
- Changements dans la structure du projet

## Synchronisation

La synchronisation doit :
1. Lire les fichiers locaux de documentation
2. Comparer avec les pages WIKI existantes
3. Mettre à jour uniquement ce qui a changé
4. Créer de nouvelles pages si nécessaire
5. Supprimer les pages obsolètes (avec confirmation)

## Notes

- **Synchronisation automatique** : La CI synchronise après les workflows Nightly Build et Release
- Le script `update-wiki.sh` est disponible pour une synchronisation manuelle locale
- L'authentification GitHub CLI (`gh`) est nécessaire uniquement pour le script local
- Les pages WIKI sont dans le dossier `wiki/` localement
- **Important** : Le workflow CI nécessite un secret `WIKI_PAT` (Personal Access Token avec scope `repo`)
- Le `GITHUB_TOKEN` standard ne peut pas pousser vers les wikis GitHub
- Toujours faire un `--dry-run` avant la mise à jour manuelle réelle

## Configuration requise pour la CI

### Secret WIKI_PAT

Le workflow de synchronisation nécessite un Personal Access Token pour pousser vers le wiki :

1. Créer un PAT sur https://github.com/settings/tokens
2. Sélectionner "Generate new token (classic)"
3. Donner le scope `repo` (Full control of private repositories)
4. Ajouter le secret `WIKI_PAT` dans Settings > Secrets and variables > Actions

### Activation du Wiki

Le wiki doit être activé dans les paramètres du dépôt : Settings > Features > Wikis
