# Système de Seuils de Qualité (Quality Gates)

Ce document décrit le système de seuils de qualité mis en place pour garantir que seules les nightly builds respectant certains critères de qualité sont créées.

## 🎯 Objectif

Le système de **Quality Gates** (seuils de qualité) permet de :
- ✅ Bloquer automatiquement la création de nightly builds si les seuils ne sont pas respectés
- ✅ Maintenir un niveau de qualité constant dans le projet
- ✅ Détecter rapidement les régressions de qualité
- ✅ Forcer l'amélioration progressive de la qualité du code

## 📋 Fonctionnement

### Workflow

1. **CI passe** → Le workflow CI doit réussir avant de vérifier les seuils
2. **Vérification des seuils** → Le job `quality-gates` exécute toutes les vérifications
3. **Build nightly** → Si tous les seuils sont respectés, la nightly build est créée
4. **Échec** → Si un seuil n'est pas respecté, la nightly build est bloquée

### Schéma

```
┌─────────────┐
│   CI Pass   │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│  Quality Gates      │
│  - Coverage         │
│  - Clippy           │
│  - Security         │
│  - Dependencies     │
│  - Code Metrics     │
└──────┬──────────────┘
       │
       ├─── ✅ Tous les seuils OK
       │         │
       │         ▼
       │    ┌─────────────┐
       │    │ Nightly     │
       │    │ Build       │
       │    └─────────────┘
       │
       └─── ❌ Seuil non respecté
                 │
                 ▼
            ┌─────────────┐
            │ Build       │
            │ Bloquée     │
            └─────────────┘
```

## ⚙️ Configuration

### Fichier de Configuration

Les seuils sont définis dans `.github/quality-gates.toml` :

```toml
[coverage]
minimum = 80.0  # Couverture minimum requise (%)

[clippy]
max_warnings = 0  # Nombre maximum de warnings autorisés

[security]
max_critical_vulnerabilities = 0
max_high_vulnerabilities = 0

[dependencies]
max_outdated = 5  # Nombre maximum de dépendances obsolètes

[code_metrics]
min_tests = 20  # Nombre minimum de tests requis
max_binary_size = 10_000_000  # Taille max du binaire (bytes)
```

### Sections Disponibles

#### `[coverage]`
- `minimum` : Couverture de code minimum requise (en %)
- `target` : Couverture cible (pour information)
- `exclude` : Fichiers à exclure du calcul

#### `[clippy]`
- `max_warnings` : Nombre maximum de warnings Clippy autorisés
- `severity` : Niveau de sévérité minimum (`allow`, `warn`, `deny`)
- `strict` : Activer les règles strictes

#### `[security]`
- `max_critical_vulnerabilities` : Nombre max de vulnérabilités critiques
- `max_high_vulnerabilities` : Nombre max de vulnérabilités importantes
- `max_medium_vulnerabilities` : Nombre max de vulnérabilités moyennes
- `allow_unpatched` : Autoriser les vulnérabilités non corrigées

#### `[dependencies]`
- `max_outdated` : Nombre max de dépendances obsolètes
- `max_duplicates` : Nombre max de dépendances dupliquées
- `check_licenses` : Vérifier les licences des dépendances

#### `[code_metrics]`
- `min_tests` : Nombre minimum de tests requis
- `min_integration_tests` : Nombre minimum de tests d'intégration
- `max_binary_size` : Taille maximale du binaire (bytes)
- `max_cyclomatic_complexity` : Complexité cyclomatique maximale
- `max_unsafe_percentage` : Pourcentage max de code unsafe

#### `[documentation]`
- `min_documentation_coverage` : Couverture de documentation minimum (%)
- `require_public_docs` : Exiger de la documentation pour les fonctions publiques

#### `[tests]`
- `require_all_tests_pass` : Tous les tests doivent passer
- `max_test_duration` : Durée max d'exécution des tests (secondes)
- `enable_performance_tests` : Activer les tests de performance

#### `[nightly]`
- `strict_mode` : Mode strict pour nightly builds
- `block_on_coverage_decrease` : Bloquer si la couverture diminue
- `block_on_new_vulnerabilities` : Bloquer si nouvelles vulnérabilités
- `block_on_new_warnings` : Bloquer si nouveaux warnings

## 🔧 Utilisation

### Vérification Locale

Pour vérifier les seuils localement avant de pousser :

```bash
# Installer les outils nécessaires
cargo install cargo-tarpaulin cargo-audit cargo-outdated cargo-deny --locked

# Générer la couverture
cargo tarpaulin --out Xml --output-dir coverage

# Vérifier les seuils
./.github/scripts/check-quality-gates.sh
```

### Dans GitHub Actions

Le workflow `.github/workflows/nightly.yml` vérifie automatiquement les seuils avant de créer une nightly build.

### Résultat de la Vérification

Le script affiche un résumé des vérifications :

```
🔍 Vérification des seuils de qualité...

✅ Couverture de code: 85.23% (minimum: 80.0%)
✅ Clippy warnings: 0 warnings (maximum: 0)
✅ Vulnérabilités de sécurité: Critiques: 0, Importantes: 0
✅ Dépendances obsolètes: 3 dépendances (maximum: 5)
✅ Nombre de tests: 25 tests (minimum: 20)
✅ Taille du binaire: 5242880 bytes (maximum: 10000000 bytes)

✅ Tous les seuils de qualité sont respectés !
```

En cas d'échec :

```
❌ Couverture de code: 75.50% (minimum requis: 80.0%)
✅ Clippy warnings: 0 warnings (maximum: 0)
...

❌ 1 seuil(s) de qualité non respecté(s)
La nightly build ne peut pas être créée.
```

## 📊 Seuils Recommandés

### Pour un Projet en Développement

```toml
[coverage]
minimum = 60.0

[clippy]
max_warnings = 5

[security]
max_critical_vulnerabilities = 0
max_high_vulnerabilities = 2

[dependencies]
max_outdated = 10
```

### Pour un Projet en Production

```toml
[coverage]
minimum = 80.0

[clippy]
max_warnings = 0

[security]
max_critical_vulnerabilities = 0
max_high_vulnerabilities = 0
max_medium_vulnerabilities = 3

[dependencies]
max_outdated = 3
```

### Pour un Projet Critique

```toml
[coverage]
minimum = 90.0

[clippy]
max_warnings = 0
strict = true

[security]
max_critical_vulnerabilities = 0
max_high_vulnerabilities = 0
max_medium_vulnerabilities = 0

[dependencies]
max_outdated = 0
max_duplicates = 0
check_licenses = true
```

## 🚀 Amélioration Progressive

### Stratégie

1. **Phase 1** : Définir des seuils réalistes basés sur l'état actuel
2. **Phase 2** : Augmenter progressivement les seuils chaque mois
3. **Phase 3** : Maintenir les seuils élevés une fois atteints

### Exemple d'Évolution

**Mois 1** :
```toml
[coverage]
minimum = 50.0
```

**Mois 2** :
```toml
[coverage]
minimum = 60.0
```

**Mois 3** :
```toml
[coverage]
minimum = 70.0
```

**Mois 4+** :
```toml
[coverage]
minimum = 80.0
```

## 🔍 Dépannage

### La nightly build est bloquée

1. Consultez les logs du job `quality-gates` dans GitHub Actions
2. Identifiez les seuils non respectés
3. Corrigez les problèmes ou ajustez les seuils si nécessaire

### Le script ne trouve pas les outils

Installez les outils manquants :

```bash
cargo install cargo-tarpaulin cargo-audit cargo-outdated cargo-deny --locked
```

### La couverture ne peut pas être calculée

Vérifiez que :
- `cargo-tarpaulin` est installé
- Les tests peuvent s'exécuter
- Le fichier `coverage/cobertura.xml` est généré

## 📚 Ressources

- [cargo-tarpaulin Documentation](https://docs.rs/cargo-tarpaulin/)
- [cargo-audit Documentation](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-outdated Documentation](https://github.com/kbknapp/cargo-outdated)
- [cargo-deny Documentation](https://embarkstudios.github.io/cargo-deny/)

## ✅ Checklist

- [x] Fichier de configuration créé (`.github/quality-gates.toml`)
- [x] Script de vérification créé (`.github/scripts/check-quality-gates.sh`)
- [x] Workflow nightly mis à jour avec vérification des seuils
- [x] Documentation complète
- [ ] Tests du script de vérification
- [ ] Intégration avec les métriques historiques (optionnel)

---

**Note** : Les seuils doivent être ajustés selon les besoins spécifiques de votre projet. Commencez avec des valeurs réalistes et augmentez-les progressivement.
