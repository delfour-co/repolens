<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Catégories de règles

RepoLens organise ses règles d'audit en six catégories principales.

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

**Objectif** : Vérifier les bonnes pratiques de sécurité.

### Vérifications

- Présence de SECURITY.md
- Configuration sécurisée des workflows
- Absence de dépendances vulnérables
- Configuration sécurisée de Git
- Protection des branches

### Bonnes pratiques

- ✅ Avoir une politique de sécurité claire
- ✅ Activer les alertes de vulnérabilité GitHub
- ✅ Utiliser Dependabot pour les mises à jour
- ✅ Protéger les branches principales
- ✅ Exiger des reviews de code

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

## 🎯 Quality

**Objectif** : Vérifier les standards de qualité de code.

### Vérifications

- Présence de fichiers de configuration (`.editorconfig`, etc.)
- Configuration de linter
- Configuration de formatter
- Tests unitaires
- Coverage de code

### Bonnes pratiques

- ✅ Configuration de linter (ESLint, Clippy, etc.)
- ✅ Configuration de formatter (Prettier, rustfmt, etc.)
- ✅ Tests unitaires et d'intégration
- ✅ Configuration de coverage

## Désactiver une catégorie

Pour désactiver une catégorie de règles :

```toml
[rules]
secrets = true
files = true
docs = false  # Désactiver la catégorie docs
security = true
workflows = true
quality = false  # Désactiver la catégorie quality
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
