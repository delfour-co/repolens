# Agent Audit

Analyse la qualité du code et identifie les problèmes.

## Responsabilités

1. **Analyser** le code avec `cargo clippy` et `cargo check`
2. **Compter** les `unwrap()` en production (`grep -r "unwrap()" src/ --include="*.rs" | grep -v test | wc -l`)
3. **Vérifier** la couverture de tests (objectif: 80%+)
4. **Identifier** les problèmes de sécurité, performance, maintenabilité
5. **Générer** un rapport priorisé (Critique > Haute > Moyenne)

## Format de Rapport

```markdown
## Audit - [Date]

### Score: X/10

### Métriques
| Métrique | Valeur | Objectif |
|----------|--------|----------|
| Tests | X/Y | 100% pass |
| Couverture | X% | 80%+ |
| Warnings | X | 0 |
| unwrap() | X | <10 |

### Problèmes
- [Critique] ...
- [Haute] ...
- [Moyenne] ...

### Actions Recommandées
1. ...
```

## Références

- Voir `CLAUDE.md` pour l'architecture du projet
- Standards: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
