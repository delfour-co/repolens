# Claude Agents - RepoLens

Configuration Claude Code pour le projet RepoLens.

## Agents

| Agent | Rôle | Usage |
|-------|------|-------|
| [audit-agent](./audit-agent.md) | Analyse qualité et problèmes | `@audit-agent` |
| [dev-agent](./dev-agent.md) | Implémentation et correctifs | `@dev-agent` |
| [qa-agent](./qa-agent.md) | Tests et validation | `@qa-agent` |
| [release-agent](./release-agent.md) | Gestion des releases | `@release-agent` |
| [security-agent](./security-agent.md) | Audit sécurité | `@security-agent` |

## Commands

| Command | Description |
|---------|-------------|
| [/audit](./commands/audit.md) | Audit complet du repo |
| [/fix](./commands/fix.md) | Appliquer les correctifs |
| [/update-doc](./commands/update-doc.md) | Mettre à jour la documentation |
| [/release](./commands/release.md) | Préparer une release |
| [/pr-review](./commands/pr-review.md) | Review une PR |
| [/security](./commands/security.md) | Audit de sécurité |

## Git Worktree (Isolation Multi-Agents)

Quand plusieurs agents travaillent en parallèle, chacun **DOIT** utiliser `git worktree` pour éviter les conflits:

```bash
# Chaque agent crée son worktree isolé
git worktree add -b feature/issue-XX ../worktrees/feature/issue-XX origin/main

# L'agent travaille dans son worktree
cd ../worktrees/feature/issue-XX

# Après merge, nettoyage
git worktree remove ../worktrees/feature/issue-XX
```

### Pourquoi ?

Les agents parallèles partagent le même dépôt git. Sans worktree:
- Les `git checkout` s'écrasent mutuellement
- Les fichiers modifiés sont perdus entre les branches
- Les commits se mélangent

Avec worktree, chaque agent a son propre répertoire de travail.

### Structure

```
project/                    # Repo principal (main)
../worktrees/
├── feature/issue-4/        # Worktree agent #1
├── feature/issue-10/       # Worktree agent #2
└── feature/issue-20/       # Worktree agent #3
```

## Workflow Typique

```
1. /audit          → Identifier les problèmes
2. @dev-agent      → Implémenter les fixes (via worktree)
3. @qa-agent       → Valider les changements
4. /security       → Vérifier la sécurité
5. /release        → Publier la version (via worktree)
```

## Structure

```
.claude/
├── README.md           # Ce fichier
├── settings.local.json # Config locale
├── *-agent.md          # Agents spécialisés
└── commands/           # Commandes réutilisables
    ├── audit.md
    ├── fix.md
    ├── update-doc.md
    ├── release.md
    ├── pr-review.md
    └── security.md
```

## Références

- [CLAUDE.md](../CLAUDE.md) - Contexte projet
- [DEVELOPMENT.md](../DEVELOPMENT.md) - Guide développeur
