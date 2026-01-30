# Audit Command

Exécute un audit complet du repository.

## Input

- `--category`: Catégorie spécifique (secrets, files, docs, security, workflows, quality)
- `--format`: Format de sortie (terminal, json, sarif, markdown, html)

## Steps

1. Vérifier la compilation
   ```bash
   cargo check
   ```

2. Lancer l'audit RepoLens
   ```bash
   cargo run -- plan --format terminal
   ```

3. Collecter les métriques
   ```bash
   cargo test --lib 2>&1 | tail -5
   cargo clippy --all-targets 2>&1 | grep -c warning || echo "0"
   grep -r "unwrap()" src/ --include="*.rs" | grep -v test | wc -l
   ```

4. Analyser les résultats et prioriser les findings

## Output

Rapport structuré avec:
- Score global
- Métriques clés
- Liste des problèmes par priorité
- Actions recommandées
