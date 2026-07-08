<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)

---

# Presets

RepoLens propose trois presets de configuration prédéfinis pour différents cas d'usage.

## Preset `opensource`

**Utilisation** : Projets open-source publics

### Caractéristiques

- ✅ Toutes les règles activées
- ✅ Tous les fichiers requis (README, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY)
- ✅ Protection de branche avec 1 approbation requise
- ✅ Discussions GitHub activées
- ✅ Alertes de vulnérabilité activées

### Configuration

```bash
repolens init --preset opensource
```

### Fichiers générés

- `LICENSE` (MIT par défaut)
- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`
- Templates d'issues GitHub
- Template de pull request

### Paramètres de protection de branche

- Branche protégée : `main`
- Approbations requises : 1
- Checks de statut requis : Oui
- Force push bloqué : Oui
- Commits signés requis : Non

## Preset `enterprise`

**Utilisation** : Projets internes d'entreprise

### Caractéristiques

- ✅ Règles de sécurité strictes
- ✅ Pas de LICENSE (projets internes)
- ✅ Pas de CONTRIBUTING (processus internes)
- ✅ Protection de branche avec 2 approbations
- ✅ Commits signés requis
- ✅ URLs internes autorisées

### Configuration

```bash
repolens init --preset enterprise
```

### Fichiers générés

- `SECURITY.md` (toujours recommandé)
- Templates d'issues GitHub (optionnel)

### Paramètres de protection de branche

- Branche protégée : `main`
- Approbations requises : 2
- Checks de statut requis : Oui
- Force push bloqué : Oui
- Commits signés requis : Oui

### Différences avec `opensource`

- ❌ Pas de LICENSE
- ❌ Pas de CONTRIBUTING.md
- ❌ Pas de CODE_OF_CONDUCT.md
- ❌ Discussions GitHub désactivées
- ✅ Commits signés requis
- ✅ Plus d'approbations requises

## Preset `strict`

**Utilisation** : Environnements haute sécurité ou réglementés

### Caractéristiques

- ✅ Toutes les règles activées au maximum
- ✅ Tous les fichiers requis
- ✅ Protection de branche stricte (2 approbations)
- ✅ Commits signés requis
- ✅ Aucune URL interne autorisée

### Configuration

```bash
repolens init --preset strict
```

### Fichiers générés

- `LICENSE` (MIT par défaut)
- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`
- Templates d'issues GitHub
- Template de pull request

### Paramètres de protection de branche

- Branche protégée : `main`
- Approbations requises : 2
- Checks de statut requis : Oui
- Force push bloqué : Oui
- Commits signés requis : Oui

### Différences avec `opensource`

- ✅ Commits signés requis
- ✅ Plus d'approbations requises
- ❌ Aucune URL interne autorisée

## Comparaison des presets

| Fonctionnalité | opensource | enterprise | strict |
|----------------|------------|------------|--------|
| LICENSE | ✅ | ❌ | ✅ |
| CONTRIBUTING | ✅ | ❌ | ✅ |
| CODE_OF_CONDUCT | ✅ | ❌ | ✅ |
| SECURITY | ✅ | ✅ | ✅ |
| Approbations requises | 1 | 2 | 2 |
| Commits signés | ❌ | ✅ | ✅ |
| URLs internes | ❌ | ✅ | ❌ |
| Discussions GitHub | ✅ | ❌ | ✅ |

### Règles par preset

| Règle | opensource | enterprise | strict |
|-------|------------|------------|--------|
| SEC011-014 (Security features) | ✅ | ✅ | ✅ |
| SEC015-017 (Actions permissions) | ✅ | ✅ | ✅ |
| CODE001-002 (CODEOWNERS) | Info | Critical si absent | Info |

## Personnalisation d'un preset

Vous pouvez utiliser un preset comme base et le personnaliser :

```toml
[general]
preset = "opensource"

# Surcharger certaines options
[rules]
git = false  # Désactiver les vérifications .gitattributes / fichiers sensibles

[actions.branch_protection]
required_approvals = 2  # Plus strict que le preset
```

## Choisir le bon preset

### Utilisez `opensource` si :

- ✅ Votre projet est public sur GitHub
- ✅ Vous voulez une configuration standard pour l'open-source
- ✅ Vous acceptez les contributions externes

### Utilisez `enterprise` si :

- ✅ Votre projet est interne à votre entreprise
- ✅ Vous avez des processus internes spécifiques
- ✅ Vous n'avez pas besoin de LICENSE/CONTRIBUTING

### Utilisez `strict` si :

- ✅ Vous travaillez dans un environnement réglementé
- ✅ La sécurité est critique
- ✅ Vous voulez le maximum de vérifications

## Migration entre presets

Pour changer de preset :

```bash
# Sauvegarder l'ancienne configuration
cp .repolens.toml .repolens.toml.backup

# Réinitialiser avec le nouveau preset
repolens init --preset enterprise

# Personnaliser si nécessaire
# Éditer .repolens.toml
```

## Prochaines étapes

- Consultez la [Configuration](configuration.md) pour personnaliser votre preset
- Découvrez les [Catégories de règles](rule-categories.md) pour comprendre chaque règle
