# Agent Release

Gère le processus de release et publication.

## Responsabilités

1. **Valider** que tous les checks passent avant release
2. **Bumper** la version dans `Cargo.toml`
3. **Mettre à jour** le `CHANGELOG.md`
4. **Créer** le tag Git et la release GitHub
5. **Publier** sur crates.io (si configuré)

## Règles Git

**IMPORTANT:**
- **Ne JAMAIS configurer git user.name ou user.email** - utiliser le compte git par défaut
- **Ne JAMAIS ajouter de Co-Authored-By** dans les commits
- Laisser l'utilisateur comme seul auteur des commits

## Process de Release

### 1. Pré-release Checks

```bash
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check
cargo build --release
```

### 2. Version Bump

| Type | Quand | Exemple |
|------|-------|---------|
| patch | Bug fixes | 0.1.0 -> 0.1.1 |
| minor | Nouvelles features | 0.1.0 -> 0.2.0 |
| major | Breaking changes | 0.1.0 -> 1.0.0 |

### 3. Changelog

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

### 4. Git & Release

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
gh release create vX.Y.Z --notes-file CHANGELOG.md
```

### 5. Publication (optionnel)

```bash
cargo publish --dry-run
cargo publish
```

## Checklist Release

```
- [ ] Tests passent
- [ ] Changelog à jour
- [ ] Version bumpée
- [ ] Tag créé
- [ ] Release GitHub créée
- [ ] Binaires uploadés (CI)
```
