# Security Command

Exécute un audit de sécurité complet.

## Steps

1. **Audit des dépendances**
   ```bash
   cargo audit
   cargo deny check advisories
   cargo deny check licenses
   ```

2. **Scan du code**
   ```bash
   # Secrets exposés
   cargo run -- plan --category secrets

   # Patterns dangereux
   grep -rn "unsafe" src/ --include="*.rs"
   grep -rn "unwrap()" src/ --include="*.rs" | grep -v test | wc -l
   ```

3. **Vérifier les configs GitHub**
   ```bash
   gh api repos/{owner}/{repo} --jq '.security_and_analysis'
   gh api repos/{owner}/{repo}/branches/main/protection
   ```

4. **Analyser les résultats**

## Output

```markdown
## Security Audit - [Date]

### Dependencies
| Tool | Vulnerabilities |
|------|-----------------|
| cargo audit | X |
| cargo deny | X |

### Code Analysis
| Pattern | Count | Risk |
|---------|-------|------|
| unsafe | X | High |
| unwrap() | X | Medium |
| Secrets | X | Critical |

### GitHub Security
- [ ] Branch protection
- [ ] Secret scanning
- [ ] Dependabot alerts
- [ ] Code scanning

### Critical Issues
1. ...

### Recommendations
1. ...
```

## Sévérité

| Level | Action |
|-------|--------|
| Critical | Fix immédiatement |
| High | Fix avant prochaine release |
| Medium | Planifier le fix |
| Low | Nice to have |
