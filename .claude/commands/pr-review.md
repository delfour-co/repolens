# PR Review Command

Review une Pull Request.

## Input

- `--pr`: Numéro de la PR (ou détection auto depuis la branche)

## Steps

1. **Récupérer les infos de la PR**
   ```bash
   gh pr view --json title,body,files,additions,deletions
   gh pr diff
   ```

2. **Analyser les changements**
   - Fichiers modifiés
   - Lignes ajoutées/supprimées
   - Impact sur l'architecture

3. **Exécuter les validations**
   ```bash
   cargo check
   cargo test --lib
   cargo clippy --all-targets
   ```

4. **Vérifier les standards**
   - [ ] Tests ajoutés pour nouveau code
   - [ ] Documentation mise à jour
   - [ ] Pas de `unwrap()` en production
   - [ ] Gestion d'erreurs appropriée
   - [ ] Pas de code mort

5. **Générer le feedback**

## Output

```markdown
## PR Review: #XX - Title

### Summary
- Files changed: X
- Additions: +X
- Deletions: -X

### Checks
| Check | Status |
|-------|--------|
| Build | OK/FAIL |
| Tests | OK/FAIL |
| Clippy | OK/FAIL |

### Code Review
- [OK/WARN/FAIL] Tests coverage
- [OK/WARN/FAIL] Documentation
- [OK/WARN/FAIL] Error handling

### Comments
- file.rs:42 - ...

### Verdict: APPROVE / REQUEST_CHANGES / COMMENT
```
