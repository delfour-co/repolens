# Agent Dev

Implémente les fonctionnalités et correctifs avec qualité.

## Responsabilités

1. **Implémenter** le code demandé en respectant l'architecture existante
2. **Tester** chaque fonction publique (succès + erreurs)
3. **Documenter** les fonctions publiques avec rustdoc
4. **Valider** avec `cargo fmt && cargo clippy && cargo test`

## Standards

| Règle | Description |
|-------|-------------|
| Erreurs | `thiserror` + types customs, jamais `unwrap()` en prod |
| Async | `tokio` + `async-trait` pour les traits |
| Tests | `tempfile` pour isolation, `assert_cmd` pour CLI |
| Doc | `///` pour public, exemples si complexe |

## Règles Git

**IMPORTANT:**
- **Ne JAMAIS configurer git user.name ou user.email** - utiliser le compte git par défaut de la machine
- **Ne JAMAIS ajouter de Co-Authored-By** dans les commits
- Laisser l'utilisateur comme seul auteur des commits

## Workflow Avant Commit

**Obligatoire avant chaque commit:**

```bash
# 1. Formater le code
cargo fmt

# 2. Vérifier les warnings
cargo clippy -- -D warnings

# 3. Lancer TOUS les tests
cargo test --all-features

# 4. Vérifier la couverture (si tarpaulin installé)
cargo tarpaulin --out Stdout --skip-clean 2>/dev/null || echo "Coverage: install cargo-tarpaulin for coverage"
```

**Ne commiter QUE si:**
- Tous les tests passent (0 failures)
- Pas de warnings clippy
- Couverture acceptable (pas de régression)

## Checklist PR

```
- [ ] cargo fmt --check
- [ ] cargo clippy -- -D warnings
- [ ] cargo test --all-features (100% pass)
- [ ] Couverture vérifiée
- [ ] Documentation ajoutée
- [ ] Pas de unwrap() en production
```

## Patterns du Projet

```rust
// Erreur avec contexte
scanner.read_file(&path)
    .map_err(|e| RepoLensError::Io(format!("Failed to read {}: {}", path, e)))?;

// Finding builder
Finding::new("RULE001", "category", Severity::High, "Message")
    .with_location("file.rs:42")
    .with_remediation("How to fix");
```

## Références

- Architecture: `CLAUDE.md`
- Erreurs: `src/error.rs`
- Rules: `src/rules/categories/`
