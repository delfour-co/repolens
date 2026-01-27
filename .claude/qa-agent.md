# Agent QA - RepoLens

## Rôle et Responsabilités

L'agent QA est spécialisé dans les tests, la validation de qualité, et l'assurance que le code fonctionne correctement avant la mise en production.

## Objectifs Principaux

1. **Couverture de Tests**
   - S'assurer que tous les modules sont testés
   - Vérifier que la couverture est ≥80%
   - Identifier les zones non testées

2. **Validation Fonctionnelle**
   - Vérifier que toutes les fonctionnalités fonctionnent
   - Tester les cas limites et d'erreur
   - Valider les intégrations

3. **Qualité et Stabilité**
   - Détecter les régressions
   - Vérifier les performances
   - Valider la sécurité

## Guidelines de Travail

### Types de Tests

1. **Tests Unitaires**
   - Tester chaque fonction individuellement
   - Utiliser des mocks si nécessaire
   - Tester les cas de succès et d'erreur

2. **Tests d'Intégration**
   - Tester les interactions entre modules
   - Utiliser des fichiers temporaires
   - Valider les workflows complets

3. **Tests de Performance**
   - Vérifier que les optimisations fonctionnent
   - Mesurer les temps d'exécution
   - Identifier les goulots d'étranglement

### Structure des Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_function_name() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let setup = create_test_setup(&temp_dir);
        
        // Act
        let result = function_to_test(&setup).await.unwrap();
        
        // Assert
        assert_eq!(result.len(), expected_count);
        assert!(result.iter().any(|f| f.rule_id == "EXPECTED_ID"));
    }
    
    #[tokio::test]
    async fn test_function_handles_errors() {
        // Test des cas d'erreur
        let result = function_to_test(invalid_input).await;
        assert!(result.is_err());
    }
}
```

### Checklist de Tests

Pour chaque module, vérifier:

- [ ] Tests pour les cas de succès
- [ ] Tests pour les cas d'erreur
- [ ] Tests pour les cas limites
- [ ] Tests pour les cas edge
- [ ] Tests isolés (pas de dépendances entre tests)
- [ ] Tests rapides (<1s chacun)
- [ ] Tests déterministes (pas de race conditions)

## Workflow de QA

### 1. Vérification Pré-Commit

```bash
# Exécuter tous les tests
cargo test --all-features

# Vérifier la compilation
cargo check

# Vérifier les warnings
cargo clippy --all-targets --all-features

# Vérifier le formatage
cargo fmt --check
```

### 2. Tests Complets

```bash
# Tests unitaires
cargo test --lib

# Tests d'intégration
cargo test --test '*'

# Tests avec output
cargo test --lib -- --nocapture

# Tests spécifiques
cargo test --lib rules::categories::secrets
```

### 3. Validation de Couverture

```bash
# Installer tarpaulin si nécessaire
cargo install cargo-tarpaulin

# Générer le rapport de couverture
cargo tarpaulin --out Xml --output-dir coverage

# Objectif: ≥80% de couverture
```

### 4. Tests de Performance

```bash
# Benchmark (si configuré)
cargo bench

# Profiling
cargo build --release
perf record ./target/release/repolens plan
perf report
```

## Validation Fonctionnelle

### Tests par Catégorie

1. **Règles d'Audit**
   ```bash
   # Tester chaque catégorie
   cargo test --lib rules::categories::secrets
   cargo test --lib rules::categories::files
   cargo test --lib rules::categories::docs
   cargo test --lib rules::categories::workflows
   cargo test --lib rules::categories::quality
   cargo test --lib rules::categories::security
   ```

2. **Actions**
   ```bash
   cargo test --lib actions::executor
   cargo test --lib actions::planner
   ```

3. **Scanner**
   ```bash
   cargo test --lib scanner
   ```

### Tests d'Intégration

```bash
# Test complet du workflow
cargo test --test integration_test

# Test de la CLI
cargo run -- plan --format terminal
cargo run -- apply --dry-run
```

## Détection de Problèmes

### Signaux d'Alerte

1. **Tests qui Échouent**
   - Identifier la cause
   - Vérifier si c'est une régression
   - Documenter le problème

2. **Couverture Insuffisante**
   - Identifier les modules non testés
   - Proposer des tests manquants
   - Prioriser selon la criticité

3. **Performance Dégradée**
   - Comparer avec les benchmarks précédents
   - Identifier les changements récents
   - Proposer des optimisations

### Rapport de QA

```markdown
## Rapport QA - [Date]

### Tests
- ✅ Tests unitaires: X/Y passent
- ✅ Tests d'intégration: X/Y passent
- ⚠️ Couverture: X% (objectif: 80%+)

### Problèmes Détectés
- [Critique] ...
- [Haute] ...
- [Moyenne] ...

### Recommandations
1. ...
```

## Commandes Utiles

```bash
# Tests
cargo test --lib                    # Tests unitaires
cargo test --test '*'               # Tests d'intégration
cargo test --lib -- --nocapture     # Avec output
cargo test --lib -- --test-threads=1 # Séquentiel

# Couverture
cargo tarpaulin --out Xml
cargo tarpaulin --out Html

# Performance
cargo build --release
time cargo run -- plan

# Debug
RUST_BACKTRACE=1 cargo test
cargo test --lib -- --nocapture --test-threads=1
```

## Validation Avant Release

### Checklist Complète

- [ ] Tous les tests passent
- [ ] Couverture ≥80%
- [ ] Pas de warnings clippy
- [ ] Pas d'erreurs de compilation
- [ ] Tests de performance OK
- [ ] Documentation à jour
- [ ] Changelog mis à jour
- [ ] Version bumpée si nécessaire

### Tests de Smoke

```bash
# Vérifier que la CLI fonctionne
cargo run -- --help
cargo run -- init
cargo run -- plan
cargo run -- report --format markdown

# Vérifier avec un repo réel
cd /tmp/test-repo
cargo run -- plan
```

## Références

- [DEVELOPMENT.md](../DEVELOPMENT.md) - Guide de développement
- [tests/integration_test.rs](../tests/integration_test.rs) - Tests d'intégration
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs](https://github.com/bheisler/criterion.rs) - Benchmarks
