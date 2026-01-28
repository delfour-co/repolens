# Release Command

Prépare et exécute une release.

## Input

- `--type`: Type de release (patch, minor, major)
- `--dry-run`: Simuler sans exécuter

## Steps

1. **Valider l'état du repo**
   ```bash
   git status
   cargo test --all-features
   cargo clippy -- -D warnings
   ```

2. **Déterminer la nouvelle version**
   - Lire version actuelle dans `Cargo.toml`
   - Calculer nouvelle version selon `--type`

3. **Mettre à jour les fichiers**
   - Bumper `Cargo.toml`
   - Mettre à jour `CHANGELOG.md`

4. **Créer le commit et tag**
   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore: release vX.Y.Z"
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   ```

5. **Pusher et créer la release GitHub**
   ```bash
   git push origin main --tags
   gh release create vX.Y.Z --generate-notes
   ```

## Output

- Version précédente -> nouvelle version
- Changelog généré
- URL de la release GitHub

## Checklist

```
- [ ] Tous les tests passent
- [ ] Pas de warnings clippy
- [ ] CHANGELOG à jour
- [ ] Version bumpée
- [ ] Tag créé
- [ ] Release GitHub créée
```
