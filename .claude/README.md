# Agents Claude - RepoLens

Ce dossier contient les agents spécialisés pour différents aspects du projet RepoLens.

## Agents Disponibles

### 🔍 [audit-agent.md](./audit-agent.md)
**Agent Audit** - Spécialisé dans l'analyse de code et la vérification de qualité

- Analyse systématique du code
- Identification des problèmes de sécurité et performance
- Génération de rapports détaillés
- Vérification de conformité aux standards

**Quand l'utiliser**: Pour auditer le code, vérifier la qualité, identifier les problèmes

### 💻 [dev-agent.md](./dev-agent.md)
**Agent Développement** - Spécialisé dans l'implémentation et l'amélioration du code

- Développement de nouvelles fonctionnalités
- Implémentation de correctifs
- Respect des standards de code
- Écriture de tests et documentation

**Quand l'utiliser**: Pour développer de nouvelles features, corriger des bugs, améliorer le code

### ✅ [qa-agent.md](./qa-agent.md)
**Agent QA** - Spécialisé dans les tests et la validation

- Couverture de tests complète
- Validation fonctionnelle
- Tests de performance
- Validation avant release

**Quand l'utiliser**: Pour tester le code, valider les fonctionnalités, vérifier la qualité

## Commandes Disponibles

### [commands/audit.md](./commands/audit.md)
Commande pour exécuter un audit du repository

### [commands/fix.md](./commands/fix.md)
Commande pour appliquer les correctifs identifiés

### [commands/update-doc.md](./commands/update-doc.md)
Commande pour mettre à jour la documentation

## Utilisation

Pour utiliser un agent spécifique, référencez-le dans votre conversation :

```
@audit-agent.md Analyse le code et génère un rapport
@dev-agent.md Implémente cette fonctionnalité
@qa-agent.md Vérifie que tous les tests passent
```

Ou utilisez les commandes :

```
@audit.md
@fix.md
@update-doc.md
```

## Workflow Recommandé

1. **Audit** → Utiliser `@audit-agent.md` pour identifier les problèmes
2. **Développement** → Utiliser `@dev-agent.md` pour implémenter les correctifs
3. **QA** → Utiliser `@qa-agent.md` pour valider et tester

## Structure

```
.claude/
├── README.md              # Ce fichier
├── audit-agent.md         # Agent Audit
├── dev-agent.md           # Agent Développement
├── qa-agent.md            # Agent QA
└── commands/              # Commandes réutilisables
    ├── audit.md
    ├── fix.md
    └── update-doc.md
```
