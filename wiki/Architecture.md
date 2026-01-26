<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Architecture

Ce document décrit l'architecture technique de RepoLens.

## Vue d'ensemble

RepoLens est construit en Rust avec une architecture modulaire qui sépare les préoccupations :

- **CLI** : Interface en ligne de commande
- **Config** : Gestion de la configuration
- **Scanner** : Analyse du dépôt
- **Rules Engine** : Moteur d'exécution des règles
- **Actions** : Planification et exécution des correctifs
- **Providers** : Intégration avec les APIs externes
- **Output** : Formats de sortie

## Architecture en couches

```
┌─────────────────────────────────────────┐
│           CLI Layer                     │
│  (main.rs, cli/commands/)              │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│        Business Logic                    │
│  (rules/, actions/, scanner/)           │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│        Infrastructure                   │
│  (config/, providers/, output/)         │
└─────────────────────────────────────────┘
```

## Modules principaux

### CLI (`src/cli/`)

Gère l'interface en ligne de commande et le routage des commandes.

**Responsabilités** :
- Parsing des arguments (via `clap`)
- Routage vers les commandes appropriées
- Gestion de la sortie utilisateur

**Commandes** :
- `init` : Initialisation de la configuration
- `plan` : Génération du plan d'audit
- `apply` : Application des correctifs
- `report` : Génération de rapports

### Configuration (`src/config/`)

Gère le chargement et la validation de la configuration.

**Responsabilités** :
- Chargement depuis `.repolens.toml`
- Application des presets
- Validation de la configuration
- Fusion des configurations (local + preset + defaults)

**Fichiers** :
- `loader.rs` : Chargement de la configuration
- `presets/` : Définitions des presets

### Scanner (`src/scanner/`)

Analyse le dépôt pour collecter les informations nécessaires.

**Responsabilités** :
- Scan du système de fichiers
- Extraction d'informations Git
- Détection de fichiers et patterns

**Modules** :
- `filesystem.rs` : Scan des fichiers
- `git.rs` : Informations Git (branches, commits, etc.)

### Rules Engine (`src/rules/`)

Moteur d'exécution des règles d'audit.

**Responsabilités** :
- Exécution des règles par catégorie
- Collecte des findings
- Calcul de la sévérité

**Structure** :
- `engine.rs` : Moteur principal
- `categories/` : Catégories de règles
  - `secrets.rs` : Détection de secrets
  - `files.rs` : Vérification des fichiers
  - `docs.rs` : Qualité de la documentation
  - `security.rs` : Bonnes pratiques de sécurité
  - `workflows.rs` : Validation des workflows
  - `quality.rs` : Standards de qualité
- `patterns/` : Patterns de détection
  - `secrets.rs` : Patterns de secrets

### Actions (`src/actions/`)

Planification et exécution des actions correctives.

**Responsabilités** :
- Planification des actions basée sur les findings
- Exécution des actions
- Génération de fichiers depuis templates

**Modules** :
- `planner.rs` : Planification des actions
- `executor.rs` : Exécution des actions
- `templates.rs` : Génération depuis templates
- `github_settings.rs` : Configuration GitHub
- `branch_protection.rs` : Protection des branches
- `gitignore.rs` : Mise à jour de .gitignore

### Providers (`src/providers/`)

Intégration avec les APIs externes.

**Responsabilités** :
- Communication avec GitHub API
- Abstraction des APIs externes

**Modules** :
- `github.rs` : Provider GitHub (via `gh` CLI)

### Output (`src/cli/output/`)

Formats de sortie pour les résultats.

**Responsabilités** :
- Formatage des résultats
- Export dans différents formats

**Formats** :
- `terminal.rs` : Sortie terminal colorée
- `json.rs` : Format JSON
- `sarif.rs` : Format SARIF (pour GitHub Security)
- `markdown.rs` : Format Markdown
- `html.rs` : Format HTML

## Flux de données

### Commande `plan`

```
CLI (plan)
  ↓
Config Loader
  ↓
Scanner (scan filesystem + git)
  ↓
Rules Engine (execute rules)
  ↓
Action Planner (generate actions)
  ↓
Output Formatter
  ↓
Terminal/File
```

### Commande `apply`

```
CLI (apply)
  ↓
Config Loader
  ↓
Action Executor
  ↓
  ├─ Template Generator (create files)
  ├─ GitHub Provider (update settings)
  └─ Git Operations (update .gitignore)
  ↓
Results
```

## Patterns de conception

### Strategy Pattern

Les formats de sortie utilisent le pattern Strategy :

```rust
trait OutputFormatter {
    fn format(&self, results: &AuditResults) -> String;
}
```

### Factory Pattern

Les actions sont créées via un factory basé sur les findings :

```rust
impl ActionPlanner {
    fn plan_actions(&self, findings: &[Finding]) -> Vec<Box<dyn Action>> {
        // Création d'actions basée sur les findings
    }
}
```

### Provider Pattern

Les intégrations externes utilisent le pattern Provider :

```rust
trait Provider {
    async fn get_repo_info(&self) -> Result<RepoInfo>;
}
```

## Gestion des erreurs

### Types d'erreurs

- **ConfigError** : Erreurs de configuration
- **ScanError** : Erreurs de scan
- **RuleError** : Erreurs d'exécution de règles
- **ActionError** : Erreurs d'exécution d'actions
- **ProviderError** : Erreurs d'API externes

### Propagation

Utilisation de `anyhow::Result` pour la propagation d'erreurs avec contexte :

```rust
fn scan_repository() -> anyhow::Result<ScanResults> {
    let files = scan_files()?;
    let git_info = get_git_info()?;
    Ok(ScanResults { files, git_info })
}
```

## Performance

### Optimisations

- **Async I/O** : Utilisation de `tokio` pour les opérations I/O
- **Lazy evaluation** : Chargement à la demande
- **Caching** : Cache des résultats de scan
- **Parallel execution** : Exécution parallèle des règles indépendantes

### Profiling

```bash
# Profiler avec flamegraph
cargo install flamegraph
cargo flamegraph --bin repolens -- plan
```

## Extensibilité

### Ajouter une nouvelle règle

1. Créer la fonction de règle dans `src/rules/categories/`
2. Enregistrer dans `src/rules/engine.rs`
3. Ajouter la configuration dans `src/config/loader.rs`

### Ajouter un nouveau format

1. Créer le module dans `src/cli/output/`
2. Implémenter le trait `OutputFormatter`
3. Enregistrer dans `src/cli/output/mod.rs`

### Ajouter un nouveau provider

1. Créer le module dans `src/providers/`
2. Implémenter le trait `Provider`
3. Utiliser dans les actions appropriées

## Tests

### Structure des tests

```
tests/
├── unit/              # Tests unitaires (dans les modules)
├── integration_test.rs # Tests d'intégration
└── fixtures/          # Données de test
```

### Stratégie de test

- **Unit tests** : Chaque module a ses tests
- **Integration tests** : Tests du CLI complet
- **Mock providers** : Mocks pour les tests d'intégration

## Sécurité

### Considérations

- Validation stricte de la configuration
- Sanitization des inputs
- Pas d'exécution de code arbitraire
- Gestion sécurisée des secrets (jamais loggés)

## Dépendances principales

- **clap** : CLI framework
- **tokio** : Runtime async
- **serde** : Sérialisation
- **toml** : Parsing TOML
- **regex** : Pattern matching
- **tracing** : Logging
- **tera** : Templates

## Prochaines étapes

- Consultez le [Guide de Développement](Developpement) pour commencer à contribuer
- Explorez le code source pour comprendre les détails d'implémentation
