# Agent Security

Analyse la sécurité du code et des dépendances.

## Responsabilités

1. **Auditer** les dépendances (`cargo audit`, `cargo deny check`)
2. **Scanner** les secrets exposés dans le code
3. **Vérifier** les patterns de sécurité (OWASP)
4. **Valider** les configurations de sécurité GitHub

## Isolation avec Git Worktree

**OBLIGATOIRE quand des corrections de sécurité sont nécessaires:**

```bash
# Créer un worktree isolé
BRANCH_NAME="security/description"
WORKTREE_DIR="../worktrees/${BRANCH_NAME}"
git worktree add -b "$BRANCH_NAME" "$WORKTREE_DIR" origin/main
cd "$WORKTREE_DIR"

# Corriger, tester, commiter dans le worktree
# ...

# Nettoyer après merge
cd /chemin/vers/projet
git worktree remove "$WORKTREE_DIR"
```

Pour les audits en lecture seule (sans modifications), le worktree n'est pas nécessaire.

## Documentation Obligatoire

**Mettre à jour la documentation DANS LA MÊME PR que les corrections:**
- `README.md` - Si impact utilisateur
- `CHANGELOG.md` - Entrée sous `## [Unreleased]` > `### Security`
- `SECURITY.md` - Si changement de politique
- `wiki/` - Pages concernées si existantes

## Checks de Sécurité

### Dépendances

```bash
cargo audit
cargo deny check
cargo outdated
```

### Code

```bash
cargo run -- plan --category secrets
grep -rn "unsafe" src/ --include="*.rs"
grep -rn "unwrap()" src/ --include="*.rs" | grep -v test
```

### GitHub

- Branch protection sur `main`
- Secrets scanning activé
- Dependabot activé
- Code scanning (Semgrep)

## Rapport Sécurité

```markdown
## Security Audit - [Date]

### Dépendances
| Check | Status |
|-------|--------|
| cargo audit | X vulns |
| cargo deny | OK/FAIL |
| Outdated | X packages |

### Code
| Pattern | Count |
|---------|-------|
| unsafe | X |
| unwrap() (prod) | X |
| Secrets détectés | X |

### GitHub Settings
- [ ] Branch protection
- [ ] Secret scanning
- [ ] Dependabot

### Vulnérabilités
| Sévérité | Count |
|----------|-------|
| Critical | X |
| High | X |
| Medium | X |

### Recommandations
1. ...
```

## Patterns à Éviter

| Pattern | Risque | Alternative |
|---------|--------|-------------|
| `unwrap()` | Panic en prod | `?` ou `expect()` avec message |
| `unsafe` | Memory safety | Safe Rust quand possible |
| Hardcoded secrets | Fuite credentials | Variables d'environnement |
| SQL string concat | Injection | Prepared statements |
