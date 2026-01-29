<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# RepoLens - Documentation

Bienvenue dans la documentation de RepoLens, un outil CLI pour auditer les dépôts GitHub et garantir le respect des bonnes pratiques, de la sécurité et de la conformité.

## Qu'est-ce que RepoLens ?

RepoLens est un outil en ligne de commande écrit en Rust qui permet d'auditer automatiquement vos dépôts GitHub pour :

- 🔒 **Sécurité** : Détection de secrets exposés, audit de sécurité du code, validation des politiques de sécurité
- 📋 **Conformité** : Vérification des fichiers requis (README, LICENSE, CONTRIBUTING, etc.)
- 📚 **Documentation** : Validation de la qualité et de la complétude de la documentation
- ⚙️ **CI/CD** : Validation des workflows GitHub Actions
- 🎯 **Qualité** : Standards de qualité de code avec vérification de la couverture de tests (≥80%)
- 📦 **Dépendances** : Vérification de la sécurité des dépendances via OSV API et GitHub Advisories
- 🛠️ **Règles personnalisées** : Support des règles d'audit personnalisées via regex ou commandes shell

## Navigation

### Pour les Utilisateurs

- [Installation](Installation) - Comment installer RepoLens
- [Guide d'utilisation](Guide-d-utilisation) - Utilisation de base et exemples
- [Configuration](Configuration) - Configuration avancée
- [Presets](Presets) - Presets disponibles (opensource, enterprise, strict)
- [Catégories de règles](Categories-de-regles) - Détails des règles d'audit
- [Règles personnalisées](Custom-Rules) - Créer vos propres règles d'audit
- [Changelog Automatique](Changelog-Automatique) - Génération automatique du changelog
- [Bonnes pratiques](Bonnes-pratiques) - Recommandations et préconisations

### Pour les Développeurs

- [Développement](Developpement) - Guide de développement et contribution
- [Architecture](Architecture) - Architecture du projet
- [Contribution](Contribution) - Comment contribuer au projet

## Démarrage rapide

```bash
# Installation via crates.io
cargo install repolens

# Ou télécharger le binaire pré-compilé depuis les releases :
# https://github.com/kdelfour/repolens/releases

# Initialisation
repolens init --preset opensource

# Audit
repolens plan

# Application des correctifs (mode interactif ou automatique)
repolens apply --interactive
repolens apply --dry-run

# Générer un rapport JSON avec validation de schéma
repolens report --format json --schema --validate

# Comparer deux rapports d'audit
repolens compare --base-file before.json --head-file after.json

# Installer les git hooks (pre-commit + pre-push)
repolens install-hooks
```

Pour l'intégration CI/CD, utilisez l'Action GitHub officielle :

```yaml
- uses: kdelfour/repolens-action@v1
  with:
    preset: opensource
```

## Fonctionnalités principales

- ✅ Audit automatique des dépôts GitHub
- ✅ Détection de secrets et credentials exposés
- ✅ **Audit de sécurité du code** : Détection de code unsafe, analyse Semgrep, vérification des patterns dangereux
- ✅ **Vérification de la sécurité des dépendances** : Scan multi-écosystèmes (Rust, Node.js, Python, Go) via OSV API et GitHub Advisories
- ✅ **Couverture de tests** : Vérification de la couverture minimale de 80% avec quality gates configurables
- ✅ **Règles personnalisées** : Support des règles d'audit personnalisées via patterns regex ou commandes shell
- ✅ Vérification des fichiers requis
- ✅ Validation des workflows GitHub Actions
- ✅ Génération de plans d'action
- ✅ Application automatique des correctifs
- ✅ Formats de sortie multiples (Terminal, JSON, SARIF, Markdown, HTML)
- ✅ **Cache d'audit** : Système de cache avec invalidation SHA256 pour des audits plus rapides
- ✅ **Git hooks** : Hooks pre-commit (secrets) et pre-push (audit complet) intégrés
- ✅ **Comparaison de rapports** : Comparaison de deux rapports JSON pour détecter régressions et améliorations
- ✅ **JSON Schema** : Schéma JSON (draft-07) pour valider les rapports d'audit
- ✅ **Conformité des licences** : Vérification de la compatibilité des licences des dépendances (LIC001-LIC004)
- ✅ **Changelog automatique** : Génération automatique du CHANGELOG à partir des commits

## Support

- 📖 Consultez la documentation complète ci-dessous
- 🐛 [Signaler un bug](https://github.com/delfour-co/cli--repolens/issues)
- 💡 [Proposer une fonctionnalité](https://github.com/delfour-co/cli--repolens/issues)
- 📧 Questions ? Ouvrez une issue sur GitHub
