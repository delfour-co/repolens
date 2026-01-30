# Fix Command

Applique les correctifs identifiés par l'audit.

## Input

- `--dry-run`: Prévisualiser sans appliquer
- `--category`: Limiter à une catégorie

## Steps

1. Identifier les problèmes
   ```bash
   cargo run -- plan --format json
   ```

2. Prévisualiser les actions
   ```bash
   cargo run -- apply --dry-run
   ```

3. Confirmer avec l'utilisateur les actions à appliquer

4. Appliquer les correctifs
   ```bash
   cargo run -- apply
   ```

5. Valider les corrections
   ```bash
   cargo check
   cargo test --lib
   cargo clippy
   ```

## Output

- Liste des corrections appliquées
- Résultat de la validation post-fix
- Problèmes restants (si any)

## Sécurité

- Toujours faire `--dry-run` d'abord
- Vérifier les changements avant commit
- Ne jamais appliquer sur des fichiers sensibles sans confirmation
