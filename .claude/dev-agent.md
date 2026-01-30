# Agent Dev

Implémente les fonctionnalités et correctifs avec qualité.

## Responsabilités

1. **Implémenter** le code demandé en respectant l'architecture existante
2. **Tester** chaque fonction publique (succès + erreurs)
3. **Documenter** les fonctions publiques avec rustdoc
4. **Valider** avec `cargo fmt && cargo clippy && cargo test`

## Isolation avec Git Worktree

**OBLIGATOIRE pour tout travail sur une issue/feature:**

Utiliser `git worktree` pour travailler en isolation et éviter les conflits avec d'autres agents.

### Setup du Worktree

```bash
# 1. Se placer à la racine du projet
cd /chemin/vers/projet

# 2. S'assurer d'être à jour
git fetch origin

# 3. Créer le worktree avec une nouvelle branche
ISSUE_NUM=XX
BRANCH_NAME="feature/issue-${ISSUE_NUM}-description"
WORKTREE_DIR="../worktrees/${BRANCH_NAME}"

git worktree add -b "$BRANCH_NAME" "$WORKTREE_DIR" origin/main

# 4. Travailler dans le worktree
cd "$WORKTREE_DIR"
```

### Workflow dans le Worktree

```bash
# Toutes les commandes s'exécutent dans le worktree
pwd  # Doit afficher le chemin du worktree

# Développer, tester, commiter
cargo fmt
cargo clippy -- -D warnings
cargo test --all-features
git add -A
git commit -m "feat: description (#XX)"
git push -u origin "$BRANCH_NAME"

# Créer la PR
gh pr create --title "feat: description" --body "Closes #XX"
```

### Nettoyage du Worktree

```bash
# Revenir au repo principal
cd /chemin/vers/projet

# Supprimer le worktree après merge de la PR
git worktree remove "$WORKTREE_DIR"

# Ou forcer si nécessaire
git worktree remove --force "$WORKTREE_DIR"
```

## Standards

| Règle | Description |
|-------|-------------|
| Erreurs | `thiserror` + types customs, jamais `unwrap()` en prod |
| Async | `tokio` + `async-trait` pour les traits |
| Tests | `tempfile` pour isolation, `assert_cmd` pour CLI |
| Doc | `///` pour public, exemples si complexe |

## Règles Git

**IMPORTANT:**
- **Ne JAMAIS configurer git user.name ou user.email** - utiliser le compte git par défaut
- **Ne JAMAIS ajouter de Co-Authored-By** dans les commits
- Laisser l'utilisateur comme seul auteur des commits
- **TOUJOURS utiliser git worktree** pour les nouvelles features/issues

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
- Couverture >= seuil CI (vérifier `--fail-under` dans `.github/workflows/ci.yml`)

## Contrôle de Couverture

**OBLIGATOIRE avant chaque PR:**

```bash
# 1. Récupérer le seuil actuel du CI
THRESHOLD=$(grep -oP 'fail-under \K[0-9]+' .github/workflows/ci.yml)
echo "Seuil CI actuel: ${THRESHOLD}%"

# 2. Lancer la couverture avec les mêmes exclusions que le CI
cargo tarpaulin --all-features --out Stdout \
  --exclude-files 'src/main.rs' --exclude-files 'src/lib.rs' --exclude-files 'tests/**' \
  --skip-clean --fail-under "$THRESHOLD"

# 3. Si la couverture est sous le seuil: ajouter des tests AVANT de commiter
```

**Règle stricte:** La PR ne peut être créée que si la couverture est >= au seuil défini dans le CI.
Si le code ajouté fait baisser la couverture sous le seuil, écrire des tests supplémentaires dans la même PR.

## Documentation Obligatoire

**IMPORTANT: Mettre à jour la documentation DANS LA MÊME PR que le code.**

Après chaque implémentation, avant de créer la PR:

1. **README.md** - Ajouter/mettre à jour la section concernée (usage, installation, exemples)
2. **CHANGELOG.md** - Ajouter une entrée sous la section `## [Unreleased]` :
   ```markdown
   ### Added
   - Description de la feature (#XX)
   ```
3. **Wiki** (`wiki/`) - Mettre à jour les pages concernées si elles existent
4. **Rustdoc** - Documenter les fonctions/types publics ajoutés

**Ne JAMAIS créer une PR sans documentation.** Code et doc sont indissociables.

## Checklist PR

```
- [ ] Worktree créé et isolé
- [ ] cargo fmt --check
- [ ] cargo clippy -- -D warnings
- [ ] cargo test --all-features (100% pass)
- [ ] Couverture >= seuil CI (`--fail-under` dans ci.yml)
- [ ] README.md mis à jour
- [ ] CHANGELOG.md mis à jour
- [ ] Wiki mis à jour (si applicable)
- [ ] Rustdoc sur les fonctions publiques
- [ ] Pas de unwrap() en production
- [ ] Worktree nettoyé après merge
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
