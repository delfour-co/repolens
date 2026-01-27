# Agent Développement - RepoLens

## Rôle et Responsabilités

L'agent Dev est spécialisé dans le développement de nouvelles fonctionnalités, l'implémentation de correctifs, et l'amélioration continue du code.

## Objectifs Principaux

1. **Développement de Fonctionnalités**
   - Implémenter de nouvelles règles d'audit
   - Ajouter de nouveaux formats de sortie
   - Étendre les capacités du scanner

2. **Implémentation de Correctifs**
   - Appliquer les recommandations de l'audit
   - Corriger les bugs identifiés
   - Optimiser les performances

3. **Qualité du Code**
   - Écrire du code propre et maintenable
   - Suivre les principes SOLID
   - Respecter les idiomes Rust

## Guidelines de Travail

### Standards de Code

1. **Formatage et Linting**
   ```bash
   # Formater le code
   cargo fmt
   
   # Vérifier avec clippy
   cargo clippy --all-targets --all-features
   
   # S'assurer qu'il n'y a pas de warnings
   cargo check
   ```

2. **Gestion d'Erreurs**
   - Utiliser `anyhow::Result` pour la propagation
   - Ajouter des contextes avec `.with_context()`
   - Éviter les `unwrap()` dans le code de production
   - Utiliser `match` ou `if let` pour les `Option`

3. **Documentation**
   - Documenter toutes les fonctions publiques
   - Inclure des exemples dans la doc
   - Documenter les types complexes
   - Ajouter des notes de sécurité si nécessaire

4. **Tests**
   - Écrire des tests unitaires pour chaque fonction publique
   - Tester les cas d'erreur
   - Utiliser `tempfile` pour les tests isolés
   - Objectif: 80%+ de couverture

### Structure de Code

```rust
// Exemple de fonction bien structurée
/// Brief description
///
/// Detailed description with context.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of error cases
///
/// # Example
///
/// ```no_run
/// let result = function(param)?;
/// ```
pub async fn function(param: &str) -> Result<Vec<Finding>> {
    // Implementation
}
```

### Patterns à Suivre

1. **Traits pour Extensibilité**
   ```rust
   #[async_trait::async_trait]
   pub trait RuleCategory: Send + Sync {
       fn name(&self) -> &'static str;
       async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>>;
   }
   ```

2. **Builder Pattern pour Configuration**
   ```rust
   let finding = Finding::new("RULE001", "category", Severity::Critical, "Message")
       .with_location("file.rs:42")
       .with_description("Detailed description")
       .with_remediation("How to fix");
   ```

3. **Error Handling avec Context**
   ```rust
   let content = scanner.read_file(&path)
       .with_context(|| format!("Failed to read file: {}", path))?;
   ```

## Workflow de Développement

### 1. Préparation

```bash
# Vérifier l'état actuel
cargo check
cargo test

# Créer une branche si nécessaire
git checkout -b feature/nouvelle-fonctionnalite
```

### 2. Développement

1. **Écrire le Code**
   - Suivre les standards de code
   - Documenter au fur et à mesure
   - Écrire des tests en parallèle

2. **Tests**
   ```bash
   # Exécuter les tests
   cargo test --lib
   
   # Avec output
   cargo test --lib -- --nocapture
   
   # Test spécifique
   cargo test test_nom_du_test
   ```

3. **Vérification**
   ```bash
   # Formatage
   cargo fmt --check
   
   # Linting
   cargo clippy --all-targets --all-features
   
   # Compilation
   cargo build --release
   ```

### 3. Validation

```bash
# Tests complets
cargo test --all-features

# Vérifier la couverture (si disponible)
cargo tarpaulin --out Xml

# Vérifier les warnings
cargo check 2>&1 | grep -i warning
```

## Implémentation de Correctifs

### Checklist pour chaque Correctif

- [ ] Code écrit et formaté
- [ ] Tests ajoutés et passent
- [ ] Documentation complète
- [ ] Pas de warnings clippy
- [ ] Gestion d'erreurs appropriée
- [ ] Pas de `unwrap()` dans le code de production
- [ ] Complexité cyclomatique raisonnable

### Exemples de Correctifs

1. **Remplacer `unwrap()`**
   ```rust
   // Avant
   let value = something.unwrap();
   
   // Après
   let value = something
       .context("Failed to get value")?;
   ```

2. **Ajouter Documentation**
   ```rust
   /// Check for hardcoded secrets in source files
   ///
   /// Scans files with common source code extensions for patterns
   /// that indicate hardcoded secrets like API keys, tokens, and passwords.
   async fn check_hardcoded_secrets(...) -> Result<Vec<Finding>> {
       // ...
   }
   ```

3. **Réduire Complexité**
   ```rust
   // Extraire des fonctions helper
   fn helper_function(...) -> Result<...> {
       // Logique simplifiée
   }
   ```

## Commandes Utiles

```bash
# Développement
cargo check              # Vérifier compilation
cargo build              # Build
cargo run -- --help      # Tester la CLI
cargo test               # Tests

# Qualité
cargo fmt                # Formater
cargo clippy             # Linter
cargo test --lib         # Tests unitaires
cargo test --test '*'    # Tests d'intégration

# Debug
cargo test --lib -- --nocapture  # Tests avec output
RUST_BACKTRACE=1 cargo run       # Avec backtrace
```

## Références

- [CLAUDE.md](../CLAUDE.md) - Contexte du projet
- [DEVELOPMENT.md](../DEVELOPMENT.md) - Guide de développement
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
