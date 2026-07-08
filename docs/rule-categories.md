<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)

---

# Catégories de règles

RepoLens organise ses règles d'audit en **six catégories**. Chaque règle activée fait partie du
contrat plan/apply : elle correspond soit à une `Action` applicable (via `repolens apply`), soit
figure sur la liste explicite des règles "detection-only" documentées ci-dessous. RepoLens n'est
pas un scanner de secrets ni de dépendances — ces sujets sont couverts par des outils dédiés
(Dependabot/osv-scanner, gitleaks, cargo-deny, etc.).

## 📁 Files

**Objectif** : Vérifier la présence et le contenu de `.gitignore`.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| FILE002 | Warning | Fichier `.gitignore` absent |
| FILE003 | Info | Entrée recommandée manquante dans `.gitignore` (ex. `target/`, `node_modules/`, `.env`) |

### Bonnes pratiques

- ✅ Toujours avoir un `.gitignore` adapté à l'écosystème du projet
- ✅ Ignorer les répertoires de build et de dépendances (`target/`, `node_modules/`, `__pycache__/`)
- ✅ Ignorer les fichiers sensibles (`.env`, `*.key`, `*.pem`)

## 📚 Docs

**Objectif** : Valider la présence et la qualité de la documentation essentielle.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| DOC001 | Warning | Fichier `README` absent |
| DOC002 | Warning | README trop court (< 10 lignes) |
| DOC003 | Info | Section recommandée manquante dans le README (installation, usage, etc.) |
| DOC004 | Critical | Fichier `LICENSE` absent |
| DOC005 | Warning | Fichier `CONTRIBUTING` absent |
| DOC006 | Warning | Fichier `CODE_OF_CONDUCT` absent |
| DOC007 | Warning | Fichier `SECURITY` absent |
| DOC008 | Warning | Fichier `CHANGELOG` absent |
| DOC009 | Info | CHANGELOG ne suit pas le format [Keep a Changelog](https://keepachangelog.com/) |
| DOC010 | Info | Section `Unreleased` du CHANGELOG vide |

### Bonnes pratiques

- ✅ README avec installation, utilisation, exemples
- ✅ Licence explicite (`LICENSE`)
- ✅ Documenter le processus de contribution (`CONTRIBUTING.md`)
- ✅ Définir une politique de sécurité (`SECURITY.md`)
- ✅ Mettre à jour le `CHANGELOG.md` en suivant Keep a Changelog

## 🛡️ Security

**Objectif** : Vérifier et configurer la sécurité du dépôt sur la plateforme d'hébergement
(paramètres GitHub/GitLab), pas le code source lui-même.

### Règles de protection de branche (SEC007-010)

| Règle | Sévérité | Description |
|-------|----------|-------------|
| SEC007 | Info | Fichier `.github/settings.yml` absent |
| SEC008 | Warning | Pas de règles de protection de branche dans `settings.yml` |
| SEC009 | Warning | `required_pull_request_reviews` non configuré |
| SEC010 | Warning | `required_status_checks` non configuré |

### Fonctionnalités de sécurité GitHub (SEC011-014)

| Règle | Sévérité | Description |
|-------|----------|-------------|
| SEC011 | Warning | Vulnerability alerts désactivés |
| SEC012 | Warning | Dependabot security updates désactivés |
| SEC013 | Info | Secret scanning désactivé |
| SEC014 | Info | Push protection désactivée |

### Permissions GitHub Actions (SEC015-017)

| Règle | Sévérité | Description |
|-------|----------|-------------|
| SEC015 | Warning | GitHub Actions autorise toutes les actions (risque supply chain) |
| SEC016 | Warning | Permissions de workflow trop permissives (default != read) |
| SEC017 | Info | Pas d'approbation requise pour les workflows de forks |

> **Note multi-provider** : SEC011-017 s'appuient sur des API spécifiques à GitHub (vulnerability
> alerts, Dependabot, secret scanning, permissions Actions). GitLab n'a pas d'équivalent faisant
> autorité pour ces réglages ; ces règles ne sont donc jamais évaluées ni planifiées pour un audit
> `--provider gitlab`.

### Remédiation

- SEC007 : `repolens apply` crée un `.github/settings.yml` avec les sections de protection de
  branche déjà présentes.
- SEC008-010 : sur un `.github/settings.yml` déjà existant, `repolens apply` fusionne les sections
  manquantes (`branches:`, `required_pull_request_reviews`, `required_status_checks`) sans
  écraser le contenu déjà présent.
- SEC013-017 : `repolens apply` configure directement, via l'API GitHub, le secret scanning, la
  push protection, la politique d'actions autorisées, les permissions de workflow par défaut, et
  l'approbation des workflows de forks.

### Exemple de `.github/settings.yml`

```yaml
repository:
  name: my-repo
  private: false

branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
        dismiss_stale_reviews: true
      required_status_checks:
        strict: true
        contexts:
          - ci/test
          - ci/lint
      enforce_admins: true
      restrictions: null
```

### Bonnes pratiques

- ✅ Configurer `.github/settings.yml` pour la protection des branches
- ✅ Exiger des reviews de code avant merge (SEC009)
- ✅ Exiger des status checks avant merge (SEC010)
- ✅ Activer les alertes de vulnérabilité GitHub et Dependabot
- ✅ Activer le secret scanning et la push protection GitHub
- ✅ Restreindre les actions autorisées et les permissions de workflow par défaut

## 🔧 Git

**Objectif** : Vérifier l'hygiène du dépôt Git.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| GIT002 | Info | Fichier `.gitattributes` absent |
| GIT003 | Warning | Fichiers sensibles trackés (`.env`, `*.key`, `*.pem`, credentials, `*_rsa`) non ignorés |

### Bonnes pratiques

- ✅ Configurer `.gitattributes` pour définir les comportements de diff, merge et fin de ligne
- ✅ Ne jamais tracker de fichiers sensibles (les ajouter à `.gitignore`)
- ✅ Vérifier régulièrement les fichiers trackés par erreur

## 👥 CODEOWNERS

**Objectif** : Valider le fichier CODEOWNERS.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| CODE001 | Info (Critical en preset `enterprise`) | Fichier CODEOWNERS absent |
| CODE002 | Warning | Fichier CODEOWNERS avec erreurs de syntaxe |

### Emplacements supportés

RepoLens recherche le fichier CODEOWNERS dans :
- `CODEOWNERS`
- `.github/CODEOWNERS`
- `docs/CODEOWNERS`

### Bonnes pratiques

- ✅ Créer un fichier CODEOWNERS pour les reviews automatiques
- ✅ Utiliser des équipes plutôt que des utilisateurs individuels
- ✅ Couvrir les fichiers critiques (configs, sécurité, CI)

## 📊 Metadata

**Objectif** : Vérifier que les métadonnées du dépôt sont correctement configurées pour la
visibilité et la découvrabilité.

### Règles

| Règle | Sévérité | Description |
|-------|----------|-------------|
| META001 | Info | Description du dépôt manquante |
| META002 | Info | Aucun topic/tag configuré |
| META003 | Info | URL du site web (homepage) non configurée |

### Remédiation

`repolens apply` peut renseigner directement la description, les topics et l'URL du dépôt via
l'action `UpdateRepoMetadata`.

### Bonnes pratiques

- ✅ Ajouter une description claire au dépôt
- ✅ Configurer des topics pertinents pour la discoverability
- ✅ Ajouter une URL vers la documentation ou le site web

## Désactiver une catégorie

Pour exclure une catégorie de règles d'un audit, utilisez `--skip` (ou `--only` pour se limiter à
certaines catégories) :

```bash
# Auditer uniquement certaines catégories
repolens plan --only files,docs

# Exclure la catégorie metadata
repolens plan --skip metadata
```

Les catégories valides sont : `files`, `docs`, `security`, `git`, `codeowners`, `metadata`. Toute
autre valeur (y compris les catégories retirées en v3.0.0 : `secrets`, `dependencies`, `licenses`,
`history`, `issues`, `docker`, `workflows`, `quality`, `custom`) est rejetée par la CLI avec une
erreur.

## Priorité des règles

Les règles sont classées par niveau de sévérité :

- 🔴 **Critical** : Problèmes bloquants (ex. licence absente)
- 🟠 **Warning** : Problèmes importants à corriger
- 🔵 **Info** : Suggestions d'amélioration

## Personnalisation

Chaque règle peut être personnalisée dans `.repolens.toml` via `[rules.<RULE_ID>]` (activation et
sévérité). Consultez la page [Configuration](configuration.md) pour plus de détails.

## Prochaines étapes

- Consultez la [Configuration](configuration.md) pour personnaliser les règles
- Découvrez les [Presets](presets.md) qui préconfigurent ces règles
