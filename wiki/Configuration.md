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
secrets = true      # Détection de secrets
files = true        # Vérification des fichiers requis
docs = true         # Qualité de la documentation
security = true     # Bonnes pratiques de sécurité
workflows = true    # Validation des workflows GitHub Actions
quality = true      # Standards de qualité de code
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
