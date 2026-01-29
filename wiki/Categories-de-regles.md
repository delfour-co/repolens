<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Catégories de règles

RepoLens organise ses règles d'audit en neuf catégories.

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

### Vérifications

- Présence de SECURITY.md
- Configuration sécurisée des workflows
- Configuration sécurisée de Git
- Protection des branches
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

### Bonnes pratiques

- ✅ Avoir une politique de sécurité claire (SECURITY.md)
- ✅ Activer les alertes de vulnérabilité GitHub
- ✅ Utiliser Dependabot pour les mises à jour
- ✅ Protéger les branches principales
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

### Vérifications

- Vulnérabilités dans les dépendances via l'API OSV (Open Source Vulnerabilities)
- Vulnérabilités via GitHub Security Advisories
- Support multi-écosystèmes : Cargo (Rust), npm (Node.js), PyPI (Python), Go modules

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
```

### Bonnes pratiques

- ✅ Mettre à jour régulièrement les dépendances
- ✅ Utiliser des fichiers de verrouillage (Cargo.lock, package-lock.json, etc.)
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
