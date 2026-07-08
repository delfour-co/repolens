<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)

---

# Guide d'utilisation

Ce guide vous explique comment utiliser RepoLens pour auditer vos dépôts GitHub.

## Commandes principales

RepoLens propose neuf commandes :

- `init` : Initialiser la configuration
- `plan` : Générer un plan d'audit
- `apply` : Appliquer les correctifs (mode interactif supporté)
- `report` : Générer un rapport d'audit (avec validation JSON Schema)
- `schema` : Afficher ou exporter le JSON Schema des rapports d'audit
- `compare` : Comparer deux rapports d'audit JSON
- `install-hooks` : Installer ou supprimer les Git hooks
- `completions` : Générer les complétions shell
- `generate-man` : Générer les pages de manuel

`plan`, `report`, et `apply` acceptent un flag `--provider {github|gitlab}` pour choisir
explicitement la plateforme d'hébergement à auditer ; sans ce flag, le provider est auto-détecté
depuis le remote `origin` du dépôt (voir [Multi-provider](#multi-provider-github--gitlab)
ci-dessous).

## Initialisation

### Créer une configuration par défaut

```bash
repolens init
```

Cela crée un fichier `.repolens.toml` à la racine de votre projet avec les paramètres par défaut.

### Utiliser un preset

```bash
# Preset open-source (recommandé pour les projets publics)
repolens init --preset opensource

# Preset entreprise (pour les projets internes)
repolens init --preset enterprise

# Preset strict (sécurité maximale)
repolens init --preset strict
```

## Audit (Plan)

### Audit de base

```bash
repolens plan
```

Affiche les résultats de l'audit dans le terminal avec un formatage coloré.

### Multi-provider (GitHub / GitLab)

RepoLens audite deux plateformes d'hébergement : **GitHub** (via `gh`/`GITHUB_TOKEN`) et
**GitLab** (via `glab`/`GITLAB_TOKEN`). Le provider est déterminé, par ordre de priorité :

1. Le flag `--provider {github|gitlab}` passé à `plan`, `report` ou `apply` ;
2. La clé `provider = "github"` (ou `"gitlab"`) dans `.repolens.toml` ;
3. À défaut, auto-détection depuis l'URL du remote `origin` (GitHub par défaut si la détection
   échoue).

```bash
# Forcer l'audit d'un dépôt GitLab, même si le remote n'est pas hébergé sur gitlab.com
repolens plan --provider gitlab

# Auditer un dépôt GitHub explicitement
repolens plan --provider github
```

Les vérifications qui n'ont pas d'équivalent faisant autorité sur l'autre plateforme (par exemple
SEC013-017, qui reposent sur des API spécifiques à GitHub) sont simplement ignorées pour ce
provider plutôt que de générer un faux résultat.

### Auditer un autre répertoire

```bash
# Auditer un répertoire différent avec l'option -C
repolens -C /chemin/vers/projet plan

# Peut être combiné avec d'autres options
repolens -C ../autre-projet plan --format json
```

### Formats de sortie

```bash
# Format JSON (pour intégration avec d'autres outils)
repolens plan --format json

# Format SARIF (pour GitHub Security, CodeQL, etc.)
repolens plan --format sarif

# Format Markdown (pour documentation)
repolens plan --format markdown

# Format HTML (rapport visuel)
repolens plan --format html --output report.html
```

### Niveaux de verbosité

```bash
# Mode silencieux
repolens plan -q

# Mode normal (par défaut)
repolens plan

# Mode verbeux (-v) : affiche le timing total
repolens plan -v
# Sortie: Audit completed in 1.23s

# Mode très verbeux (-vv) : affiche le timing par catégorie
repolens plan -vv
# Sortie:
# [files] 12ms
# [docs] 45ms
# [security] 890ms
# Total: 1.23s

# Mode trace (-vvv) : informations de debug détaillées
repolens plan -vvv
```

Le niveau de verbosité peut aussi être configuré via la variable d'environnement `REPOLENS_VERBOSE` (0-3).

### Filtrer par catégories

Les 6 catégories valides sont : `files`, `docs`, `security`, `git`, `codeowners`, `metadata`.

```bash
# Auditer uniquement certaines catégories
repolens plan --only files,docs

# Auditer uniquement la sécurité
repolens plan --only security

# Exclure certaines catégories
repolens plan --skip metadata
```

Fournir une catégorie inconnue ou retirée (par exemple `secrets` ou `dependencies`, supprimées en
v3.0.0) fait échouer la commande avec une erreur — `--only`/`--skip` ne l'ignorent plus
silencieusement :

```bash
repolens plan --only secrets
# error: invalid value 'secrets' for '--only <ONLY>'
#   [possible values: files, docs, security, git, codeowners, metadata]
```

## Application des correctifs

### Mode dry-run (aperçu)

```bash
repolens apply --dry-run
```

Affiche ce qui serait modifié sans appliquer les changements.

### Appliquer tous les correctifs

```bash
repolens apply
```

⚠️ **Attention** : Cette commande modifie votre dépôt. Utilisez `--dry-run` d'abord !

### Appliquer des correctifs spécifiques

```bash
# Appliquer uniquement les correctifs de fichiers
repolens apply --only files

# Appliquer fichiers et documentation
repolens apply --only files,docs
```

### Mode interactif

```bash
# Sélection interactive des actions avec aperçu diff
repolens apply --interactive
repolens apply -i

# Accepter toutes les actions sans confirmation
repolens apply --yes
repolens apply -y
```

Le mode interactif offre :
1. **Résumé visuel** des actions par catégorie
2. **Sélection multi-choix** (Espace pour toggle, Entrée pour confirmer)
3. **Aperçu diff** coloré pour chaque action (vert = ajouts, rouge = suppressions)
4. **Barre de progression** pendant l'exécution
5. **Résumé d'exécution** avec compteurs succès/échec

### Création automatique d'issues GitHub

Après l'exécution des actions, `repolens apply` crée automatiquement une issue GitHub par catégorie de warning détectée. Chaque issue contient un tableau récapitulatif des findings (rule_id, message, location) et est labellisée `repolens-audit`.

```bash
# Désactiver la création automatique d'issues
repolens apply --no-issues

# Combiner avec d'autres options
repolens apply --yes --no-issues
```

> **Note** : La création d'issues nécessite que le GitHub CLI (`gh`) soit installé et authentifié. Si `gh` n'est pas disponible, l'étape est ignorée avec un avertissement.

### Pull Request et fichiers commités

Lorsqu'une PR est créée automatiquement, seuls les fichiers modifiés par les actions sont commités (par exemple, `.gitignore`, fichiers créés). Les fichiers de rapport ne sont pas inclus dans le commit.

## Génération de rapports

### Rapport terminal

```bash
repolens report
```

Affiche un résumé des résultats d'audit dans le terminal.

### Export de rapport

```bash
# Rapport HTML
repolens report --format html --output audit-report.html

# Rapport Markdown
repolens report --format markdown --output audit-report.md

# Rapport JSON
repolens report --format json --output audit-report.json

# Rapport JSON avec référence au JSON Schema
repolens report --format json --schema

# Rapport JSON avec validation contre le schéma
repolens report --format json --schema --validate
```

## JSON Schema

RepoLens fournit un JSON Schema (draft-07) décrivant la structure du rapport JSON.

```bash
# Afficher le schéma sur stdout
repolens schema

# Exporter le schéma dans un fichier
repolens schema --output schemas/audit-report.schema.json
```

## Comparaison de rapports

Comparez deux rapports JSON pour détecter les régressions et améliorations.

```bash
# Générer deux rapports à des moments différents
repolens report --format json --output before.json
# ... faire des changements ...
repolens report --format json --output after.json

# Comparer (sortie terminal colorée)
repolens compare --base-file before.json --head-file after.json

# Comparer en JSON
repolens compare --base-file before.json --head-file after.json --format json

# Comparer en Markdown
repolens compare --base-file before.json --head-file after.json --format markdown

# Sauvegarder la comparaison
repolens compare --base-file before.json --head-file after.json --output comparison.md --format markdown

# Échouer si des régressions sont détectées (CI)
repolens compare --base-file baseline.json --head-file current.json --fail-on-regression
```

La comparaison inclut :
- **Score pondéré** : Critical=10, Warning=3, Info=1 avec diff
- **Nouveaux findings** : Régressions (présents dans head, absents dans base)
- **Findings résolus** : Améliorations (présents dans base, absents dans head)
- **Ventilation par catégorie** : Changements de comptage par catégorie

## Git Hooks

RepoLens peut installer des Git hooks pour automatiser les vérifications.

```bash
# Installer tous les hooks configurés
repolens install-hooks

# Installer uniquement le pre-commit
repolens install-hooks --pre-commit

# Installer uniquement le pre-push
repolens install-hooks --pre-push

# Écraser les hooks existants (sauvegarde automatique)
repolens install-hooks --force

# Supprimer les hooks RepoLens (restaure les sauvegardes)
repolens install-hooks --remove
```

**Comportement des hooks** :
- **pre-commit** : Scanne d'abord le diff staged à la recherche de secrets à haute confiance
  (clés privées, tokens AWS/GitHub/GitLab/Slack/Google, etc. — un scan dédié, auto-contenu dans le
  hook, indépendant de la catégorie de règles `security`), puis lance un audit rapide
  `files`/`git` (hygiène `.gitignore`, fichiers sensibles trackés). Si un secret ou un problème
  d'hygiène est détecté, le commit est annulé.
- **pre-push** : Lance un audit complet avant le push. Si des problèmes sont trouvés, le push est annulé.

Les hooks peuvent être contournés avec `--no-verify` (ex: `git commit --no-verify`).

## Cache d'audit

RepoLens inclut un système de cache pour accélérer les audits répétés.

```bash
# Désactiver le cache
repolens plan --no-cache

# Vider le cache avant l'audit
repolens plan --clear-cache

# Utiliser un répertoire de cache personnalisé
repolens plan --cache-dir /tmp/repolens-cache
```

Ces options sont aussi disponibles pour la commande `report`.

## Exemples d'utilisation

### Exemple 1 : Audit complet d'un nouveau projet

```bash
# 1. Initialiser avec preset open-source
repolens init --preset opensource

# 2. Voir ce qui doit être corrigé
repolens plan -v

# 3. Prévisualiser les changements
repolens apply --dry-run

# 4. Appliquer les correctifs
repolens apply

# 5. Générer un rapport final
repolens report --format html --output final-report.html
```

### Exemple 2 : Vérification continue dans CI/CD

```bash
# Dans votre workflow GitHub Actions
repolens plan --format sarif --output repolens-results.sarif

# Publier les résultats dans GitHub Security
```

### Exemple 3 : Audit ciblé sur la sécurité des paramètres du dépôt

```bash
# Vérifier uniquement la configuration de sécurité (protection de branche,
# secret scanning GitHub, permissions Actions, etc.)
repolens plan --only security -vv

# Appliquer les correctifs disponibles (settings.yml, réglages GitHub/GitLab)
repolens apply --only security
```

> **Note** : RepoLens n'est pas un scanner de secrets ni de dépendances. Pour ces besoins, utilisez
> un outil dédié (gitleaks, Dependabot, `cargo audit`/`cargo deny`, osv-scanner, etc.) ; le
> hook `pre-commit` installé par `repolens install-hooks` embarque toutefois un scan de secrets à
> haute confiance sur le diff staged (voir [Git Hooks](#git-hooks)).

### Exemple 4 : Utilisation via Docker

```bash
# Audit rapide du répertoire courant
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens plan

# Générer un rapport JSON
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens report --format json

# Avec accès à l'API GitHub
docker run --rm \
  -v "$(pwd)":/repo \
  -v ~/.config/gh:/home/repolens/.config/gh:ro \
  ghcr.io/systm-d/repolens plan
```

### Exemple 5 : Configuration via variables d'environnement

```bash
# Configurer le preset et le niveau de verbosité
export REPOLENS_PRESET=enterprise
export REPOLENS_VERBOSE=2

# Tous les audits utiliseront ces paramètres
repolens plan

# Les options CLI surchargent les variables d'environnement
repolens plan --preset strict  # Utilise strict malgré REPOLENS_PRESET=enterprise
```

## Configuration avancée

Consultez la page [Configuration](configuration.md) pour les options avancées de configuration.

## Bonnes pratiques

Consultez la page [Bonnes pratiques](best-practices.md) pour des recommandations sur l'utilisation de RepoLens.

## Dépannage

### Erreur "No configuration found"

```bash
# Créer une configuration
repolens init
```

### Erreur "GitHub API error"

```bash
# Vérifier l'authentification GitHub CLI
gh auth status

# Se reconnecter si nécessaire
gh auth login
```

### Résultats inattendus

```bash
# Vérifier la configuration
cat .repolens.toml

# Lancer avec plus de verbosité
repolens plan -vvv
```

## Nouvelles fonctionnalités

### Variables d'environnement

Configurez RepoLens via des variables d'environnement :

| Variable | Description |
|----------|-------------|
| `REPOLENS_PRESET` | Preset par défaut (opensource, enterprise, strict) |
| `REPOLENS_VERBOSE` | Niveau de verbosité (0-3) |
| `REPOLENS_CONFIG` | Chemin du fichier de configuration |
| `REPOLENS_NO_CACHE` | Désactiver le cache (true/false) |
| `REPOLENS_GITHUB_TOKEN` | Token GitHub pour les appels API |

### Option -C

Auditez un répertoire différent sans changer de répertoire courant :

```bash
repolens -C /chemin/vers/projet plan
```

### Timing détaillé

Le mode verbose affiche maintenant le temps d'exécution par catégorie :

```bash
repolens plan -vv
# [files] 12ms
# [git] 8ms
# [security] 890ms
# Total: 1.23s
```

### Hygiène Git

Règles pour l'hygiène du dépôt Git :
- **GIT002** : Fichier `.gitattributes` absent
- **GIT003** : Fichiers sensibles trackés

### Protection des branches

Vérification de la configuration de protection des branches :
- **SEC007-010** : Validation de `.github/settings.yml`

### Distribution Docker

Image Docker officielle multi-architecture :

```bash
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens plan
```

### Gestionnaires de paquets

Installation facilitée via :
- **Homebrew** : `brew install repolens`
- **Scoop** : `scoop install repolens`
- **AUR** : `yay -S repolens`
- **Debian/Ubuntu** : `apt install repolens`

### Intégration CI/CD

RepoLens s'intègre en CI/CD via l'image Docker (`ghcr.io/systm-d/repolens`), le binaire installé
directement dans le job, ou l'action GitHub composite officielle du dépôt (`systm-d/repolens@main`
— voir [Intégration CI/CD](ci-cd-integration.md)).

### Codes de sortie standardisés

RepoLens utilise des codes de sortie standardisés pour l'intégration CI/CD :

| Code | Signification | Exemple |
|------|--------------|---------|
| 0 | Succès | Audit terminé, pas de problèmes critiques |
| 1 | Problèmes critiques | Licence absente (DOC004) |
| 2 | Avertissements | Fichiers manquants, findings non critiques |
| 3 | Erreur d'exécution | Fichier non trouvé, erreur réseau |
| 4 | Arguments invalides | Catégorie inconnue ou retirée, preset invalide |

```bash
# Exemple d'utilisation en CI/CD
repolens plan
case $? in
  0) echo "Tout est OK!" ;;
  1) echo "Problèmes critiques - blocage de release" && exit 1 ;;
  2) echo "Avertissements - revue recommandée" ;;
  3) echo "Erreur lors de l'audit" && exit 1 ;;
  4) echo "Arguments invalides" && exit 1 ;;
esac
```

### Permissions sécurisées

Le fichier de configuration `.repolens.toml` est automatiquement protégé avec les permissions `600` (lecture/écriture propriétaire uniquement) sur les systèmes Unix pour protéger les données sensibles.

### Validation des catégories

Les catégories fournies via `--only` et `--skip` sont validées par la CLI. Une catégorie inconnue
ou retirée (par exemple une des 9 catégories supprimées en v3.0.0 : `secrets`, `dependencies`,
`licenses`, `history`, `issues`, `docker`, `workflows`, `quality`, `custom`) fait échouer la
commande — elle n'est plus silencieusement ignorée.

```bash
# Les catégories valides sont :
# files, docs, security, git, codeowners, metadata

repolens plan --only invalid
# error: invalid value 'invalid' for '--only <ONLY>'
#   [possible values: files, docs, security, git, codeowners, metadata]
```

### Couverture de tests

RepoLens vérifie que la couverture de code atteint au moins 80%. Configurez les quality gates dans `.github/quality-gates.toml`.

### Changelog automatique

Le changelog est généré automatiquement lors des releases. Voir la page [Changelog Automatique](automatic-changelog.md) pour plus de détails.

## Prochaines étapes

- Consultez la [Configuration](configuration.md) pour personnaliser RepoLens
- Découvrez les [Presets](presets.md) disponibles
- Explorez les [Catégories de règles](rule-categories.md)
- Découvrez le [Changelog Automatique](automatic-changelog.md)
