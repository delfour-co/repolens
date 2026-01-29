<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Guide d'utilisation

Ce guide vous explique comment utiliser RepoLens pour auditer vos dépôts GitHub.

## Commandes principales

RepoLens propose quatre commandes principales :

- `init` : Initialiser la configuration
- `plan` : Générer un plan d'audit
- `apply` : Appliquer les correctifs
- `report` : Générer un rapport d'audit

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

# Mode verbeux
repolens plan -v

# Mode très verbeux (debug)
repolens plan -vv

# Mode trace (maximum de détails)
repolens plan -vvv
```

### Filtrer par catégories

```bash
# Auditer uniquement certaines catégories
repolens plan --only secrets,files

# Auditer les dépendances et la sécurité
repolens plan --only dependencies,security

# Exclure certaines catégories
repolens plan --exclude quality
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

### Options interactives

```bash
# Confirmer chaque action
repolens apply --interactive

# Forcer l'application (sans confirmation)
repolens apply --force
```

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
```

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

### Exemple 3 : Audit ciblé sur les secrets

```bash
# Vérifier uniquement les secrets exposés
repolens plan --only secrets -vv

# Si des secrets sont trouvés, les corriger manuellement
# puis relancer l'audit
```

### Exemple 4 : Vérification de la sécurité des dépendances

```bash
# Vérifier les vulnérabilités dans les dépendances
repolens plan --only dependencies

# Vérifier la sécurité globale (code + dépendances)
repolens plan --only security,dependencies -v
```

### Exemple 5 : Utilisation des règles personnalisées

```bash
# Définir des règles personnalisées dans .repolens.toml
# Voir la page [Règles personnalisées](Custom-Rules) pour plus de détails

# Lancer l'audit avec les règles personnalisées
repolens plan --only custom

# Ou inclure les règles personnalisées dans un audit complet
repolens plan
```

## Configuration avancée

Consultez la page [Configuration](Configuration) pour les options avancées de configuration.

## Bonnes pratiques

Consultez la page [Bonnes pratiques](Bonnes-pratiques) pour des recommandations sur l'utilisation de RepoLens.

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

### Vérification de la sécurité des dépendances

RepoLens vérifie automatiquement les vulnérabilités dans vos dépendances via l'API OSV et GitHub Security Advisories. Support multi-écosystèmes : Rust, Node.js, Python, Go.

```bash
# Vérifier les dépendances
repolens plan --only dependencies
```

### Règles personnalisées

Créez vos propres règles d'audit via des patterns regex ou des commandes shell. Voir la page [Règles personnalisées](Custom-Rules) pour plus de détails.

### Couverture de tests

RepoLens vérifie que la couverture de code atteint au moins 80%. Configurez les quality gates dans `.github/quality-gates.toml`.

### Changelog automatique

Le changelog est généré automatiquement lors des releases. Voir la page [Changelog Automatique](Changelog-Automatique) pour plus de détails.

## Prochaines étapes

- Consultez la [Configuration](Configuration) pour personnaliser RepoLens
- Découvrez les [Presets](Presets) disponibles
- Explorez les [Catégories de règles](Categories-de-regles)
- Apprenez à créer des [Règles personnalisées](Custom-Rules)
- Découvrez le [Changelog Automatique](Changelog-Automatique)
