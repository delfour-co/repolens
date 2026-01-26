<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Bonnes pratiques

Ce guide présente les meilleures pratiques pour utiliser RepoLens et maintenir un dépôt GitHub de qualité.

## Workflow recommandé

### 1. Initialisation d'un nouveau projet

```bash
# 1. Initialiser RepoLens
repolens init --preset opensource

# 2. Vérifier la configuration
cat .repolens.toml

# 3. Lancer un audit initial
repolens plan -v

# 4. Prévisualiser les correctifs
repolens apply --dry-run

# 5. Appliquer les correctifs
repolens apply
```

### 2. Maintenance continue

```bash
# Intégrer dans votre workflow CI/CD
# Voir la section CI/CD ci-dessous
```

## Intégration CI/CD

### GitHub Actions

Ajoutez RepoLens à vos workflows GitHub Actions :

```yaml
name: RepoLens Audit

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * 0'  # Hebdomadaire

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build RepoLens
        run: |
          git clone https://github.com/delfour-co/cli--repolens.git
          cd cli--repolens
          cargo build --release
          sudo cp target/release/repolens /usr/local/bin/
      
      - name: Run audit
        run: |
          repolens plan --format sarif --output repolens-results.sarif
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: repolens-results.sarif
```

### Pipeline local

```bash
#!/bin/bash
# Script de pré-commit

# Lancer l'audit
repolens plan

# Si des problèmes critiques sont trouvés, arrêter
if [ $? -ne 0 ]; then
    echo "❌ Audit échoué. Corrigez les problèmes avant de commiter."
    exit 1
fi
```

## Gestion des secrets

### ❌ À éviter

```bash
# Ne jamais faire ça
echo "api_key=sk_live_1234567890" >> config.json
git add config.json
git commit -m "Add config"
```

### ✅ Bonnes pratiques

1. **Utiliser des variables d'environnement**

```bash
# .env (ne pas commiter)
API_KEY=sk_live_1234567890

# .env.example (commiter)
API_KEY=sk_test_your_key_here
```

2. **Utiliser des gestionnaires de secrets**

- HashiCorp Vault
- AWS Secrets Manager
- GitHub Secrets (pour CI/CD)

3. **Vérifier avant de commiter**

```bash
# Lancer RepoLens avant chaque commit
repolens plan --only secrets
```

## Documentation

### README.md

Un bon README doit contenir :

- ✅ Description du projet
- ✅ Installation
- ✅ Utilisation de base
- ✅ Exemples de code
- ✅ Configuration
- ✅ Contribution
- ✅ Licence

### Exemple de structure

```markdown
# Mon Projet

Description courte du projet.

## Installation

```bash
npm install mon-projet
```

## Utilisation

```javascript
import { MonProjet } from 'mon-projet';

const instance = new MonProjet();
```

## Configuration

Voir [Configuration](docs/configuration.md)

## Contribution

Voir [CONTRIBUTING.md](CONTRIBUTING.md)

## Licence

MIT - Voir [LICENSE](LICENSE)
```

## Protection des branches

### Configuration recommandée

```toml
[actions.branch_protection]
enabled = true
branch = "main"
required_approvals = 1        # Au moins 1 review
require_status_checks = true  # Tests doivent passer
block_force_push = true       # Pas de force push
require_signed_commits = false # Optionnel selon le contexte
```

### Pour les projets critiques

```toml
[actions.branch_protection]
required_approvals = 2
require_signed_commits = true
```

## Gestion des dépendances

### Vérifications régulières

```bash
# Vérifier les vulnérabilités
repolens plan --only security

# Utiliser Dependabot (GitHub)
# Activer dans les paramètres du dépôt
```

### Mises à jour

- ✅ Mettre à jour régulièrement les dépendances
- ✅ Tester après chaque mise à jour
- ✅ Utiliser des versions fixes (lock files)

## Tests

### Structure recommandée

```
tests/
├── unit/          # Tests unitaires
├── integration/   # Tests d'intégration
└── fixtures/      # Données de test
```

### Coverage

- ✅ Viser au moins 80% de coverage
- ✅ Tester les cas limites
- ✅ Tester les erreurs

## Code Review

### Checklist

- [ ] Code lisible et bien documenté
- [ ] Tests ajoutés/modifiés
- [ ] Pas de secrets exposés
- [ ] Documentation mise à jour
- [ ] Pas de warnings de linter
- [ ] Tests passent

## Sécurité

### Checklist de sécurité

- [ ] Aucun secret dans le code
- [ ] SECURITY.md présent
- [ ] Alertes de vulnérabilité activées
- [ ] Dependabot configuré
- [ ] Branches protégées
- [ ] Reviews de code obligatoires

## Performance

### Optimisations

- ✅ Utiliser `--dry-run` avant `apply`
- ✅ Filtrer par catégories si nécessaire
- ✅ Utiliser des presets appropriés

## Maintenance

### Audit régulier

```bash
# Audit hebdomadaire
repolens plan --format html --output weekly-audit.html

# Audit mensuel complet
repolens plan -vv --format json --output monthly-audit.json
```

### Mise à jour de RepoLens

```bash
# Mettre à jour depuis les sources
cd cli--repolens
git pull
cargo build --release
```

## Dépannage

### Problèmes courants

1. **Erreur de configuration**

```bash
# Vérifier la syntaxe
repolens init --validate
```

2. **Résultats inattendus**

```bash
# Mode debug
repolens plan -vvv
```

3. **Performance lente**

```bash
# Filtrer par catégories
repolens plan --only secrets,files
```

## Ressources

- [Guide d'utilisation](Guide-d-utilisation)
- [Configuration](Configuration)
- [Presets](Presets)
- [Catégories de règles](Categories-de-regles)

## Prochaines étapes

- Intégrez RepoLens dans votre workflow
- Configurez les presets selon vos besoins
- Automatisez les audits avec CI/CD
