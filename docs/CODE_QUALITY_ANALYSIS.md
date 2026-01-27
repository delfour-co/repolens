# Analyse de Qualité de Code et Dette Technique pour Rust

Ce document décrit les solutions disponibles pour auditer le code Rust et analyser la dette technique, similaires à SonarQube mais **gratuites** et **sans serveur à installer**.

## 🎯 Solutions Disponibles

### 1. SonarCloud ⭐ (Recommandé)

**Avantages :**
- ✅ **Gratuit pour les projets open source**
- ✅ Pas de serveur à installer (cloud)
- ✅ Interface web complète avec métriques
- ✅ Dette technique calculée automatiquement
- ✅ Intégration GitHub native
- ✅ Badges de qualité

**Limitations :**
- Support Rust limité (nécessite des outils supplémentaires)
- Nécessite un compte SonarCloud

**Configuration :**

1. Créez un compte sur [SonarCloud.io](https://sonarcloud.io)
2. Connectez votre repository GitHub
3. Obtenez votre `SONAR_TOKEN`
4. Ajoutez le token dans les secrets GitHub : `Settings > Secrets > Actions`
5. Décommentez le job `sonarcloud` dans `.github/workflows/code-quality.yml`

**Outils nécessaires :**
- `cargo-sonar` : Convertit les résultats Clippy au format SonarQube
- `cargo-llvm-cov` : Génère les rapports de couverture

---

### 2. GitHub Code Scanning (CodeQL) ✅ (Déjà configuré)

**Avantages :**
- ✅ **100% gratuit**
- ✅ Intégré directement dans GitHub
- ✅ Pas de configuration supplémentaire
- ✅ Détection automatique de vulnérabilités
- ✅ Affichage dans l'onglet "Security" du repository

**Utilisation :**
Le workflow `.github/workflows/code-quality.yml` inclut déjà l'analyse CodeQL.

**Résultats :**
- Accessibles dans `Security > Code scanning alerts`
- Alertes automatiques sur les Pull Requests

---

### 3. Outils Rust Complémentaires

#### cargo-deny ✅ (Configuré)

**Fonctionnalités :**
- Vérification des licences des dépendances
- Détection des dépendances dupliquées
- Vérification des sources des dépendances
- Détection des vulnérabilités connues

**Configuration :**
Le fichier `deny.toml` est déjà configuré avec des règles adaptées.

**Utilisation locale :**
```bash
cargo install cargo-deny --locked
cargo deny check
```

#### cargo-outdated ✅ (Configuré)

**Fonctionnalités :**
- Détecte les dépendances obsolètes
- Affiche les mises à jour disponibles
- Aide à maintenir les dépendances à jour

**Utilisation locale :**
```bash
cargo install cargo-outdated --locked
cargo outdated
```

#### cargo-audit ✅ (Déjà dans CI)

**Fonctionnalités :**
- Détecte les vulnérabilités connues dans les dépendances
- Utilise la base de données RustSec

**Utilisation :**
Déjà intégré dans `.github/workflows/ci.yml`

#### Clippy ✅ (Déjà dans CI)

**Fonctionnalités :**
- Linter Rust intégré
- Détecte les code smells
- Suggestions d'amélioration
- Analyse de complexité

**Utilisation :**
Déjà intégré dans `.github/workflows/ci.yml` avec des règles strictes

---

## 📊 Métriques Disponibles

### Métriques de Qualité

1. **Dette Technique**
   - Calculée par SonarCloud (si configuré)
   - Basée sur la complexité cyclomatique et les code smells

2. **Couverture de Code**
   - Déjà configurée avec `cargo-tarpaulin`
   - Upload vers Codecov
   - Visible dans `.github/workflows/ci.yml`

3. **Complexité**
   - Analysée par Clippy
   - Métriques disponibles dans les rapports

4. **Vulnérabilités**
   - `cargo-audit` : Vulnérabilités dans les dépendances
   - `cargo-deny` : Vulnérabilités et licences
   - CodeQL : Vulnérabilités dans le code source

5. **Dépendances**
   - `cargo-outdated` : Dépendances obsolètes
   - `cargo-deny` : Dépendances dupliquées et licences

---

## 🚀 Workflow d'Analyse

Le workflow `.github/workflows/code-quality.yml` exécute automatiquement :

1. **Dependency Audit** : Vérification des licences et vulnérabilités
2. **Outdated Dependencies** : Détection des dépendances obsolètes
3. **Clippy Analysis** : Analyse approfondie avec tous les lints
4. **Code Metrics** : Statistiques sur le code (lignes, fichiers, unsafe)
5. **CodeQL Analysis** : Analyse de sécurité GitHub
6. **Quality Report** : Rapport consolidé dans GitHub Actions

**Fréquence :**
- À chaque push sur `main`/`master`
- À chaque Pull Request
- Hebdomadaire (dimanche à minuit)

---

## 📈 Visualisation des Résultats

### Dans GitHub Actions

1. Allez dans l'onglet **Actions**
2. Sélectionnez le workflow **Code Quality & Technical Debt Analysis**
3. Consultez les résultats de chaque job

### Dans GitHub Security

1. Allez dans l'onglet **Security**
2. Consultez **Code scanning alerts** pour les résultats CodeQL
3. Consultez **Dependabot alerts** pour les vulnérabilités des dépendances

### Dans SonarCloud (si configuré)

1. Connectez-vous à [SonarCloud.io](https://sonarcloud.io)
2. Sélectionnez votre projet
3. Consultez le dashboard avec toutes les métriques

---

## 🔧 Configuration Avancée

### Personnaliser cargo-deny

Éditez `deny.toml` pour ajuster :
- Les licences autorisées/interdites
- Les crates bannis
- Les règles de vulnérabilités

### Personnaliser Clippy

Créez `clippy.toml` pour configurer les règles Clippy :
```toml
# Exemple de configuration Clippy
avoid-breaking-exported-api = false
msrv = "1.70"
```

### Ajouter d'autres outils

Vous pouvez ajouter d'autres outils dans le workflow :
- `cargo-machete` : Détection de code mort
- `cargo-geiger` : Détection de code unsafe
- `cargo-bloat` : Analyse de la taille du binaire

---

## 📚 Ressources

- [SonarCloud Documentation](https://docs.sonarcloud.io/)
- [cargo-deny Documentation](https://embarkstudios.github.io/cargo-deny/)
- [GitHub Code Scanning](https://docs.github.com/en/code-security/code-scanning)
- [Clippy Documentation](https://doc.rust-lang.org/clippy/)

---

## ✅ Checklist de Configuration

- [x] Workflow GitHub Actions créé
- [x] Configuration cargo-deny créée
- [x] CodeQL configuré
- [x] Clippy avec règles strictes
- [x] cargo-audit configuré
- [ ] SonarCloud configuré (optionnel)
- [ ] Badges de qualité ajoutés au README (optionnel)

---

## 🎓 Exemple d'Utilisation Locale

Pour exécuter les analyses localement :

```bash
# Installer les outils
cargo install cargo-deny cargo-outdated cargo-audit --locked

# Vérifier les dépendances
cargo deny check
cargo outdated
cargo audit

# Analyser le code
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

**Note :** Toutes ces solutions sont **gratuites** et fonctionnent directement dans GitHub Actions sans nécessiter de serveur dédié.
