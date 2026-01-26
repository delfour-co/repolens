<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# RepoLens - Documentation

Bienvenue dans la documentation de RepoLens, un outil CLI pour auditer les dépôts GitHub et garantir le respect des bonnes pratiques, de la sécurité et de la conformité.

## Qu'est-ce que RepoLens ?

RepoLens est un outil en ligne de commande écrit en Rust qui permet d'auditer automatiquement vos dépôts GitHub pour :

- 🔒 **Sécurité** : Détection de secrets exposés, validation des politiques de sécurité
- 📋 **Conformité** : Vérification des fichiers requis (README, LICENSE, CONTRIBUTING, etc.)
- 📚 **Documentation** : Validation de la qualité et de la complétude de la documentation
- ⚙️ **CI/CD** : Validation des workflows GitHub Actions
- 🎯 **Qualité** : Standards de qualité de code

## Navigation

### Pour les Utilisateurs

- [Installation](Installation) - Comment installer RepoLens
- [Guide d'utilisation](Guide-d-utilisation) - Utilisation de base et exemples
- [Configuration](Configuration) - Configuration avancée
- [Presets](Presets) - Presets disponibles (opensource, enterprise, strict)
- [Catégories de règles](Categories-de-regles) - Détails des règles d'audit
- [Bonnes pratiques](Bonnes-pratiques) - Recommandations et préconisations

### Pour les Développeurs

- [Développement](Developpement) - Guide de développement et contribution
- [Architecture](Architecture) - Architecture du projet
- [Contribution](Contribution) - Comment contribuer au projet

## Démarrage rapide

```bash
# Installation
git clone https://github.com/delfour-co/cli--repolens.git
cd cli--repolens
cargo build --release

# Initialisation
repolens init --preset opensource

# Audit
repolens plan

# Application des correctifs
repolens apply --dry-run
```

## Fonctionnalités principales

- ✅ Audit automatique des dépôts GitHub
- ✅ Détection de secrets et credentials exposés
- ✅ Vérification des fichiers requis
- ✅ Validation des workflows GitHub Actions
- ✅ Génération de plans d'action
- ✅ Application automatique des correctifs
- ✅ Formats de sortie multiples (Terminal, JSON, SARIF, Markdown, HTML)

## Support

- 📖 Consultez la documentation complète ci-dessous
- 🐛 [Signaler un bug](https://github.com/delfour-co/cli--repolens/issues)
- 💡 [Proposer une fonctionnalité](https://github.com/delfour-co/cli--repolens/issues)
- 📧 Questions ? Ouvrez une issue sur GitHub
