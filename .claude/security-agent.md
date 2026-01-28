# Agent Security

Analyse la sécurité du code et des dépendances.

## Responsabilités

1. **Auditer** les dépendances (`cargo audit`, `cargo deny check`)
2. **Scanner** les secrets exposés dans le code
3. **Vérifier** les patterns de sécurité (OWASP)
4. **Valider** les configurations de sécurité GitHub

## Checks de Sécurité

### Dépendances

```bash
# Vulnérabilités connues
cargo audit

# Licences et advisories
cargo deny check

# Dépendances obsolètes
cargo outdated
```

### Code

```bash
# Secrets dans le code
cargo run -- plan --category secrets

# Patterns dangereux
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
