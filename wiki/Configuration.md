<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Configuration

RepoLens utilise un fichier de configuration TOML (`.repolens.toml`) à la racine de votre projet pour personnaliser le comportement de l'audit.

## Structure de base

```toml
[general]
preset = "opensource"  # ou "enterprise", "strict"

[rules]
secrets = true
files = true
docs = true
security = true
workflows = true
quality = true
```

## Section `[general]`

### `preset`

Définit le preset à utiliser. Les presets sont des configurations prédéfinies :

- `opensource` : Standards open-source (par défaut)
- `enterprise` : Configuration entreprise
- `strict` : Sécurité maximale

```toml
[general]
preset = "opensource"
```

## Section `[rules]`

Active ou désactive les catégories de règles.

```toml
[rules]
secrets = true        # Détection de secrets
files = true          # Vérification des fichiers requis
docs = true           # Qualité de la documentation
security = true       # Bonnes pratiques de sécurité
workflows = true      # Validation des workflows GitHub Actions
quality = true        # Standards de qualité de code
licenses = true       # Conformité des licences (LIC001-LIC004)
dependencies = true   # Vulnérabilités des dépendances (DEP001-DEP002)
custom = true         # Règles personnalisées
```

## Configuration des secrets

### `[rules.secrets]`

```toml
[rules.secrets]
# Patterns à ignorer (chemins glob)
ignore_patterns = [
    "**/test/**",
    "**/tests/**",
    "**/__tests__/**",
    "**/*.test.*",
    "**/*.spec.*",
]

# Fichiers spécifiques à ignorer
ignore_files = [
    ".env.example",
    "config.example.json",
]
```

## Configuration des fichiers requis

### `[files.required]`

```toml
[files.required]
readme = true
license = true
contributing = true
code_of_conduct = true
security = true
```

## Configuration des actions

### `[actions]`

Définit quelles actions peuvent être exécutées automatiquement.

```toml
[actions]
gitignore = true  # Mettre à jour .gitignore automatiquement
```

### `[actions.license]`

```toml
[actions.license]
enabled = true
type = "MIT"  # MIT, Apache-2.0, GPL-3.0
# author = "Votre Nom"  # Optionnel
```

### `[actions.contributing]`

```toml
[actions.contributing]
enabled = true
```

### `[actions.code_of_conduct]`

```toml
[actions.code_of_conduct]
enabled = true
```

### `[actions.security_policy]`

```toml
[actions.security_policy]
enabled = true
```

### `[actions.branch_protection]`

```toml
[actions.branch_protection]
enabled = true
branch = "main"
required_approvals = 1
require_status_checks = true
block_force_push = true
require_signed_commits = false
```

### `[actions.github_settings]`

```toml
[actions.github_settings]
discussions = true
issues = true
wiki = false
vulnerability_alerts = true
automated_security_fixes = true
```

## Configuration des templates

### `[templates]`

Variables utilisées dans les templates générés.

```toml
[templates]
license_author = "Votre Nom"
license_year = "2025"
project_name = "Mon Projet"
project_description = "Description de mon projet"
```

## Exemples de configuration

### Configuration minimale

```toml
[general]
preset = "opensource"
```

### Configuration personnalisée

```toml
[general]
preset = "opensource"

[rules]
secrets = true
files = true
docs = true
security = true
workflows = false  # Désactiver la validation des workflows
quality = false    # Désactiver les vérifications de qualité

[rules.secrets]
ignore_patterns = [
    "**/test/**",
    "**/fixtures/**",
]

[files.required]
readme = true
license = true
contributing = false  # Pas de CONTRIBUTING requis
code_of_conduct = false
security = true

[actions]
gitignore = true

[actions.license]
enabled = true
type = "MIT"
author = "Mon Équipe"

[actions.branch_protection]
enabled = true
branch = "main"
required_approvals = 1
```

### Configuration entreprise

```toml
[general]
preset = "enterprise"

[rules]
secrets = true
files = true
docs = true
security = true
workflows = true
quality = true

[rules.secrets]
ignore_patterns = [
    "**/test/**",
    "**/tests/**",
    "**/fixtures/**",
    "**/mocks/**",
]

[actions.branch_protection]
enabled = true
branch = "main"
required_approvals = 2  # Plus strict pour l'entreprise
require_signed_commits = true
```

## Configuration des licences

### `["rules.licenses"]`

```toml
["rules.licenses"]
enabled = true
allowed_licenses = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC"]
denied_licenses = ["GPL-3.0", "AGPL-3.0"]
```

- `allowed_licenses` : Liste blanche de licences SPDX autorisées pour les dépendances
- `denied_licenses` : Liste noire de licences SPDX interdites

## Configuration du cache

### `[cache]`

```toml
[cache]
# Activer/désactiver le cache (défaut : true)
enabled = true
# Durée maximale des entrées de cache en heures (défaut : 24)
max_age_hours = 24
# Répertoire de cache (relatif à la racine du projet ou chemin absolu)
directory = ".repolens/cache"
```

Options CLI associées :
- `--no-cache` : Désactiver le cache pour un audit complet
- `--clear-cache` : Vider le cache avant l'audit
- `--cache-dir <DIR>` : Utiliser un répertoire de cache personnalisé

## Configuration des Git hooks

### `[hooks]`

```toml
[hooks]
# Installer le hook pre-commit (vérifie les secrets exposés)
pre_commit = true
# Installer le hook pre-push (lance un audit complet)
pre_push = true
# Échouer aussi sur les warnings (pas seulement les critiques)
fail_on_warnings = false
```

Installation via CLI :
```bash
repolens install-hooks              # Installer tous les hooks configurés
repolens install-hooks --pre-commit # Uniquement pre-commit
repolens install-hooks --pre-push   # Uniquement pre-push
repolens install-hooks --force      # Écraser les hooks existants (sauvegarde automatique)
repolens install-hooks --remove     # Supprimer les hooks RepoLens
```

## Priorité de configuration

1. **Fichier `.repolens.toml`** : Configuration locale (priorité la plus haute)
2. **Preset** : Configuration du preset sélectionné
3. **Valeurs par défaut** : Valeurs par défaut de RepoLens

## Validation de la configuration

```bash
# Vérifier la syntaxe de la configuration
repolens plan --dry-run

# Ou avec validation explicite
repolens init --validate
```

## Variables d'environnement

Certaines options peuvent être surchargées via des variables d'environnement :

```bash
# Désactiver l'application automatique
export REPOLENS_DRY_RUN=true

# Niveau de log
export RUST_LOG=debug
```

## Prochaines étapes

- Consultez les [Presets](Presets) pour des configurations prédéfinies
- Découvrez les [Catégories de règles](Categories-de-regles) pour comprendre chaque règle
