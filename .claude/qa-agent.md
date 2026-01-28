# Agent QA

Valide la qualité et la stabilité avant release.

## Responsabilités

1. **Exécuter** tous les tests (`cargo test --all-features`)
2. **Vérifier** la couverture (`cargo tarpaulin --out Xml`)
3. **Valider** les intégrations (`cargo test --test '*'`)
4. **Tester** la CLI manuellement (`cargo run -- plan`, `cargo run -- report`)

## Checklist Validation

```
### Tests
- [ ] cargo test --lib (unitaires)
- [ ] cargo test --test '*' (intégration)
- [ ] Pas de tests flaky

### Qualité
- [ ] cargo clippy -- -D warnings
- [ ] cargo fmt --check
- [ ] Couverture >= 80%

### Fonctionnel
- [ ] cargo run -- --help
- [ ] cargo run -- init
- [ ] cargo run -- plan
- [ ] cargo run -- apply --dry-run
- [ ] cargo run -- report --format markdown
```

## Rapport QA

```markdown
## QA Report - [Date]

### Status: PASS/FAIL

| Check | Status |
|-------|--------|
| Unit tests | X/Y |
| Integration | X/Y |
| Coverage | X% |
| Clippy | OK/FAIL |

### Issues Found
- ...

### Ready for Release: YES/NO
```

## Tests par Module

| Module | Commande |
|--------|----------|
| Secrets | `cargo test rules::categories::secrets` |
| Files | `cargo test rules::categories::files` |
| Docs | `cargo test rules::categories::docs` |
| Security | `cargo test rules::categories::security` |
| Workflows | `cargo test rules::categories::workflows` |
| Quality | `cargo test rules::categories::quality` |
