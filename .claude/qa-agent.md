# Agent QA

Valide la qualité et la stabilité avant release.

## Responsabilités

1. **Exécuter** tous les tests (`cargo test --all-features`)
2. **Vérifier** la couverture (`cargo tarpaulin --out Xml`)
3. **Valider** les intégrations (`cargo test --test '*'`)
4. **Tester** la CLI manuellement (`cargo run -- plan`, `cargo run -- report`)

## Isolation avec Git Worktree

**OBLIGATOIRE quand des modifications sont nécessaires (ajout de tests, fixes):**

```bash
# Créer un worktree isolé
BRANCH_NAME="qa/description"
WORKTREE_DIR="../worktrees/${BRANCH_NAME}"
git worktree add -b "$BRANCH_NAME" "$WORKTREE_DIR" origin/main
cd "$WORKTREE_DIR"

# Travailler, tester, commiter dans le worktree
# ...

# Nettoyer après merge
cd /chemin/vers/projet
git worktree remove "$WORKTREE_DIR"
```

Pour les audits en lecture seule (sans modifications), le worktree n'est pas nécessaire.

## Documentation Obligatoire

**Si des modifications sont apportées (ajout de tests, fixes), mettre à jour:**
- `CHANGELOG.md` - Entrée sous `## [Unreleased]` > `### Changed` ou `### Fixed`
- `wiki/` - Pages de tests/qualité si existantes

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
