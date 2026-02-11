<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)

---

# Catégories de règles

RepoLens organise ses règles d'audit en dix catégories.

## 🔒 Secrets

**Objectif** : Détecter les secrets, clés API, tokens et credentials exposés dans le code.

### Types de secrets détectés

- Clés API (AWS, Google Cloud, etc.)
- Tokens d'authentification (JWT, OAuth, etc.)
- Mots de passe et credentials
- Clés privées SSH
- Tokens GitHub
- Clés de chiffrement

### Configuration

```toml
[rules.secrets]
ignore_patterns = [
    "**/test/**",
    "**/tests/**",
    "**/*.example.*",
]
ignore_files = [
    ".env.example",
]
```

### Bonnes pratiques

- ✅ Utiliser des variables d'environnement
- ✅ Utiliser des gestionnaires de secrets (HashiCorp Vault, AWS Secrets Manager)
- ✅ Ne jamais commiter de secrets dans le code
- ✅ Utiliser `.env.example` pour documenter les variables nécessaires

## 📁 Files

**Objectif** : Vérifier la présence des fichiers essentiels pour un dépôt bien documenté.

### Fichiers vérifiés

- `README.md` : Documentation principale
- `LICENSE` : Licence du projet
- `CONTRIBUTING.md` : Guide de contribution
- `CODE_OF_CONDUCT.md` : Code de conduite
- `SECURITY.md` : Politique de sécurité

### Configuration

```toml
[files.required]
readme = true
license = true
contributing = true
code_of_conduct = true
security = true
```

### Bonnes pratiques

- ✅ Toujours avoir un README.md complet
- ✅ Spécifier clairement la licence
- ✅ Documenter le processus de contribution
- ✅ Définir une politique de sécurité

## 📚 Docs

**Objectif** : Valider la qualité et la complétude de la documentation.

### Vérifications

- Présence et qualité du README
- Documentation des APIs
- Exemples d'utilisation
- Documentation des configurations
- Changelog à jour

### Bonnes pratiques

- ✅ README avec installation, utilisation, exemples
- ✅ Documentation des APIs publiques
- ✅ Exemples de code fonctionnels
- ✅ Mettre à jour le CHANGELOG

## 🛡️ Security

**Objectif** : Vérifier les bonnes pratiques de sécurité et auditer le code pour les vulnérabilités.

### Règles de protection de branche

| Règle | Sévérité | Description |
|-------|----------|-------------|
| SEC007 | Info | Fichier `.github/settings.yml` absent |
| SEC008 | Warning | Pas de règles de protection de branche dans settings.yml |
| SEC009 | Warning | `required_pull_request_reviews` non configuré |
| SEC010 | Warning | `required_status_checks` non configuré |

### Vérifications

- Présence de SECURITY.md
- Configuration sécurisée des workflows
- Configuration sécurisée de Git
- Protection des branches (via `.github/settings.yml`)
- Présence de CODEOWNERS pour les reviews obligatoires
- Fichiers de verrouillage des dépendances (lock files)
- Fichiers de version runtime pour la reproductibilité

### Audit de sécurité du code

RepoLens effectue un audit complet de sécurité incluant :

- **Détection de code unsafe** : Recherche de blocs `unsafe` dans le code de production
- **Vérification des patterns dangereux** : Détection de patterns pouvant causer des vulnérabilités
- **Analyse avec Semgrep** : Intégration avec Semgrep pour détecter les vulnérabilités OWASP
- **Vérification des secrets** : Détection des secrets exposés (voir catégorie Secrets)

### Configuration

```toml
[security]
require_codeowners = true
require_lock_files = true
require_runtime_versions = true
```

### Exemple de `.github/settings.yml`

```yaml
repository:
  name: my-repo
  private: false

branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
        dismiss_stale_reviews: true
      required_status_checks:
        strict: true
        contexts:
          - ci/test
          - ci/lint
      enforce_admins: true
      restrictions: null
```

### Bonnes pratiques

- ✅ Avoir une politique de sécurité claire (SECURITY.md)
- ✅ Configurer `.github/settings.yml` pour la protection des branches
- ✅ Exiger des reviews de code avant merge (SEC009)
- ✅ Exiger des status checks avant merge (SEC010)
- ✅ Activer les alertes de vulnérabilité GitHub
- ✅ Utiliser Dependabot pour les mises à jour
- ✅ Exiger des reviews de code (CODEOWNERS)
- ✅ Utiliser des fichiers de verrouillage pour les dépendances
- ✅ Spécifier les versions runtime (`.nvmrc`, `.python-version`, etc.)
- ✅ Éviter le code `unsafe` dans le code de production
- ✅ Utiliser des outils d'analyse statique (Semgrep, CodeQL)

## ⚙️ Workflows

**Objectif** : Valider les workflows GitHub Actions et la configuration CI/CD.

### Vérifications

- Présence de workflows CI/CD
- Validation de la syntaxe YAML
- Utilisation de bonnes pratiques
- Tests automatisés
- Linting et formatage

### Bonnes pratiques

- ✅ Workflows pour les tests
- ✅ Workflows pour le linting
- ✅ Workflows pour les releases
- ✅ Utiliser des actions officielles
- ✅ Éviter les secrets hardcodés dans les workflows

## 📦 Dependencies

**Objectif** : Vérifier la sécurité des dépendances et détecter les vulnérabilités connues.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| DEP001 | Critical/Warning | Vulnérabilité détectée dans une dépendance |
| DEP002 | Warning | Version de dépendance obsolète |
| DEP003 | Warning | Fichier de verrouillage (lock file) manquant pour l'écosystème détecté |

### Lock files requis par écosystème

| Manifest | Lock File Attendu |
|----------|-------------------|
| `Cargo.toml` | `Cargo.lock` |
| `package.json` | `package-lock.json`, `yarn.lock`, ou `pnpm-lock.yaml` |
| `pyproject.toml` | `poetry.lock` ou `uv.lock` |
| `Pipfile` | `Pipfile.lock` |
| `go.mod` | `go.sum` |
| `composer.json` | `composer.lock` |
| `Gemfile` | `Gemfile.lock` |
| `*.csproj` | `packages.lock.json` |
| `pubspec.yaml` | `pubspec.lock` |
| `Package.swift` | `Package.resolved` |
| `Podfile` | `Podfile.lock` |

### Écosystèmes supportés

| Écosystème | Manifest | Lock File | Support OSV |
|------------|----------|-----------|-------------|
| Rust (Cargo) | `Cargo.toml` | `Cargo.lock` | ✅ Oui |
| Node.js (npm) | `package.json` | `package-lock.json` | ✅ Oui |
| Python (pip/poetry) | `pyproject.toml` | `poetry.lock` | ✅ Oui |
| Go | `go.mod` | `go.sum` | ✅ Oui |
| .NET (NuGet) | `*.csproj` | `packages.lock.json` | ✅ Oui |
| Ruby (Bundler) | `Gemfile` | `Gemfile.lock` | ✅ Oui |
| Dart/Flutter (Pub) | `pubspec.yaml` | `pubspec.lock` | ✅ Oui |
| Swift (SPM) | `Package.swift` | `Package.resolved` | ❌ Non |
| iOS (CocoaPods) | `Podfile` | `Podfile.lock` | ❌ Non |

> **Note** : Les écosystèmes sans support OSV (Swift, CocoaPods) sont détectés et listés, mais aucune vérification de vulnérabilité n'est effectuée. Un finding informatif (DEP004) est généré pour ces cas.

### Sources de données

RepoLens interroge deux bases de données principales :

1. **OSV API** : Base de données open-source des vulnérabilités maintenue par Google
2. **GitHub Security Advisories** : Base de données GitHub des vulnérabilités

### Types de vulnérabilités détectées

- Vulnérabilités critiques (CVSS >= 7.0)
- Vulnérabilités importantes (CVSS >= 4.0)
- Vulnérabilités moyennes et faibles
- Informations sur les versions corrigées disponibles

### Configuration

```toml
[rules]
dependencies = true  # Activer la catégorie dependencies

# La règle dependencies/vulnerabilities est activée par défaut
```

### Exemple de résultat

```
🔴 Critical: Vulnerability CVE-2023-1234 (CVSS: 9.8) found in serde 1.0.130
   Description: Remote code execution vulnerability
   Remediation: Upgrade serde to version 1.0.150 or later
   Location: Cargo.lock

🟡 Warning: Lock file missing for detected ecosystem
   Ecosystem: Node.js (npm)
   Expected: package-lock.json, yarn.lock, or pnpm-lock.yaml
   Location: package.json
```

### Bonnes pratiques

- ✅ Mettre à jour régulièrement les dépendances
- ✅ **Toujours commiter les fichiers de verrouillage** (DEP003)
- ✅ Vérifier les vulnérabilités avant chaque release
- ✅ Configurer Dependabot pour les mises à jour automatiques
- ✅ Surveiller les alertes de sécurité GitHub

## 🎯 Quality

**Objectif** : Vérifier les standards de qualité de code.

### Vérifications

- Présence de fichiers de configuration (`.editorconfig`, etc.)
- Configuration de linter
- Configuration de formatter
- Tests unitaires
- Coverage de code (minimum 80% requis)

### Couverture de tests

RepoLens vérifie que la couverture de code atteint au moins **80%** via :

- Intégration avec `cargo-tarpaulin` pour Rust
- Génération de rapports de couverture en format XML (Cobertura)
- Vérification dans les workflows CI/CD
- Quality gates configurables dans `.github/quality-gates.toml`

### Configuration

```toml
[quality]
min_coverage = 80.0  # Pourcentage minimum de couverture requis
```

### Bonnes pratiques

- ✅ Configuration de linter (ESLint, Clippy, etc.)
- ✅ Configuration de formatter (Prettier, rustfmt, etc.)
- ✅ Tests unitaires et d'intégration
- ✅ **Couverture de code >= 80%**
- ✅ Tests des cas limites et des erreurs
- ✅ Tests de performance pour les parties critiques

## 📄 Licenses

**Objectif** : Vérifier la conformité des licences du projet et de ses dépendances.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| LIC001 | Warning | Aucune licence de projet détectée |
| LIC002 | Critical/Warning | Licence de dépendance incompatible ou non autorisée |
| LIC003 | Info | Licence de dépendance inconnue/non reconnue |
| LIC004 | Warning | Dépendance sans licence spécifiée |

### Détection de la licence du projet

RepoLens détecte la licence du projet depuis :
- Fichiers `LICENSE` / `LICENSE.md` / `LICENSE.txt`
- Champ `license` dans `Cargo.toml`
- Champ `license` dans `package.json`
- Champ `license` dans `setup.cfg` / `pyproject.toml`

### Analyse des dépendances

Fichiers de dépendances supportés :
- `Cargo.toml` (Rust)
- `package.json` / `node_modules/*/package.json` (Node.js)
- `requirements.txt` (Python)
- `go.mod` (Go)

### Matrice de compatibilité

RepoLens inclut une matrice de compatibilité pour les licences SPDX courantes :
MIT, Apache-2.0, GPL-2.0, GPL-3.0, BSD-2-Clause, BSD-3-Clause, ISC, MPL-2.0, LGPL-2.1, LGPL-3.0, AGPL-3.0, Unlicense

### Configuration

```toml
["rules.licenses"]
enabled = true
allowed_licenses = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC"]
denied_licenses = ["GPL-3.0", "AGPL-3.0"]
```

### Bonnes pratiques

- Toujours spécifier une licence pour le projet
- Définir une liste de licences autorisées pour les dépendances
- Vérifier la compatibilité des licences avant d'ajouter une dépendance
- Surveiller les dépendances sans licence (LIC004)

## 🔧 Git

**Objectif** : Vérifier l'hygiène du dépôt Git et les bonnes pratiques de gestion de version.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| GIT001 | Warning | Fichiers binaires volumineux (> 1 MB) détectés - devrait utiliser Git LFS |
| GIT002 | Info | Fichier `.gitattributes` absent |
| GIT003 | Warning | Fichiers sensibles trackés (.env, *.key, *.pem, credentials, *_rsa) |

### Bonnes pratiques

- ✅ Utiliser Git LFS pour les fichiers binaires volumineux
- ✅ Configurer `.gitattributes` pour définir les comportements de diff et merge
- ✅ Ne jamais tracker de fichiers sensibles (utiliser `.gitignore`)
- ✅ Vérifier régulièrement les fichiers trackés par erreur

### Configuration

```toml
[rules]
git = true  # Activer la catégorie git
```

## 🛠️ Custom (Règles personnalisées)

**Objectif** : Permettre aux utilisateurs de définir leurs propres règles d'audit via patterns regex ou commandes shell.

Consultez la page [Règles personnalisées](Custom-Rules) pour la documentation complète.

### Configuration

```toml
# Règle par pattern regex
[rules.custom."no-todo"]
pattern = "TODO"
severity = "warning"
files = ["**/*.rs"]
message = "TODO comment found"

# Règle par commande shell
[rules.custom."check-git-status"]
command = "git status --porcelain"
severity = "warning"
invert = true
message = "Working directory is not clean"
```

## Désactiver une catégorie

Pour désactiver une catégorie de règles :

```toml
[rules]
secrets = true
files = true
docs = false        # Désactiver la catégorie docs
security = true
workflows = true
quality = true
licenses = true     # Conformité des licences
dependencies = true # Vérification des dépendances
git = true          # Hygiène Git
custom = true       # Règles personnalisées
```

## Priorité des règles

Les règles sont classées par niveau de sévérité :

- 🔴 **Critical** : Problèmes de sécurité critiques
- 🟠 **High** : Problèmes importants à corriger
- 🟡 **Medium** : Améliorations recommandées
- 🔵 **Low** : Suggestions d'amélioration

## Personnalisation

Chaque catégorie peut être personnalisée dans `.repolens.toml`. Consultez la page [Configuration](Configuration) pour plus de détails.

## Prochaines étapes

- Consultez la [Configuration](Configuration) pour personnaliser les règles
- Découvrez les [Presets](Presets) qui préconfigurent ces règles
