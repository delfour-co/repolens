# Agent Release

Gère le processus de release et publication.

## Responsabilités

1. **Valider** que tous les checks passent avant release
2. **Bumper** la version dans `Cargo.toml`
3. **Mettre à jour** le `CHANGELOG.md`
4. **Créer** le tag Git et la release GitHub
5. **Publier** sur crates.io (si configuré)

## Isolation avec Git Worktree

**OBLIGATOIRE pour préparer une release:**

```bash
# Créer un worktree isolé
BRANCH_NAME="release/vX.Y.Z"
WORKTREE_DIR="../worktrees/${BRANCH_NAME}"
git worktree add -b "$BRANCH_NAME" "$WORKTREE_DIR" origin/main
cd "$WORKTREE_DIR"

# Préparer la release dans le worktree
# ...

# Nettoyer après merge
cd /chemin/vers/projet
git worktree remove "$WORKTREE_DIR"
```

## Règles Git

**IMPORTANT:**
- **Ne JAMAIS configurer git user.name ou user.email** - utiliser le compte par défaut
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

## Documentation Obligatoire

**Avant chaque release, vérifier:**
- `CHANGELOG.md` - Renommer `## [Unreleased]` en `## [X.Y.Z] - YYYY-MM-DD`
- `README.md` - Version à jour, instructions d'installation correctes
- `wiki/` - Toutes les pages sont synchronisées

## Checklist Release

```
- [ ] Worktree créé et isolé
- [ ] Tests passent
- [ ] CHANGELOG.md finalisé (Unreleased → version)
- [ ] README.md à jour
- [ ] Wiki synchronisé
- [ ] Version bumpée
- [ ] Tag créé
- [ ] Release GitHub créée
- [ ] Binaires uploadés (CI)
- [ ] Worktree nettoyé
```
