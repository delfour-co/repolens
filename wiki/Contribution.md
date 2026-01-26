<!-- Auto-generated header - Do not edit manually -->
![Version](https://img.shields.io/badge/version-local-gray)
![Docs](https://img.shields.io/badge/docs-RepoLens-blue)

---

# Contribution

Merci de votre intérêt pour contribuer à RepoLens ! Ce guide vous explique comment contribuer efficacement.

## Comment contribuer

### Signaler un bug

1. Vérifiez que le bug n'a pas déjà été signalé dans les [issues](https://github.com/delfour-co/cli--repolens/issues)
2. Créez une nouvelle issue avec :
   - Description claire du problème
   - Steps to reproduce
   - Comportement attendu vs comportement actuel
   - Version de RepoLens et environnement

### Proposer une fonctionnalité

1. Vérifiez que la fonctionnalité n'a pas déjà été proposée
2. Créez une issue avec :
   - Description de la fonctionnalité
   - Cas d'usage
   - Avantages
   - Exemples si possible

### Contribuer au code

1. **Fork le repository**
2. **Créer une branche**
   ```bash
   git checkout -b feature/ma-feature
   # ou
   git checkout -b fix/mon-bug
   ```
3. **Développer**
   - Suivez les [bonnes pratiques de développement](Developpement)
   - Écrivez des tests
   - Documentez votre code
4. **Tester**
   ```bash
   cargo check
   cargo test
   cargo fmt --all
   cargo clippy
   ```
5. **Commit**
   ```bash
   git add .
   git commit -m "feat: description de la feature"
   ```
   Utilisez des messages de commit conventionnels :
   - `feat:` : Nouvelle fonctionnalité
   - `fix:` : Correction de bug
   - `docs:` : Documentation
   - `refactor:` : Refactoring
   - `test:` : Tests
   - `chore:` : Maintenance
6. **Push et Pull Request**
   ```bash
   git push origin feature/ma-feature
   ```
   Créez une Pull Request sur GitHub

## Standards de code

### Formatage

```bash
# Formater automatiquement
cargo fmt --all
```

### Linting

```bash
# Vérifier avec clippy
cargo clippy -- -D warnings
```

### Documentation

- Documenter toutes les fonctions publiques avec `///`
- Ajouter des exemples dans la documentation
- Mettre à jour le README si nécessaire

### Tests

- Écrire des tests pour chaque nouvelle fonctionnalité
- Maintenir la couverture de code
- Tester les cas limites et les erreurs

## Processus de review

1. **Soumission** : Créez une Pull Request
2. **Review** : L'équipe va reviewer votre code
3. **Feedback** : Des commentaires peuvent être laissés
4. **Modifications** : Apportez les modifications demandées
5. **Approbation** : Une fois approuvé, votre PR sera mergée

## Checklist avant de soumettre

- [ ] Code formaté avec `cargo fmt`
- [ ] Pas de warnings clippy
- [ ] Tous les tests passent
- [ ] Documentation à jour
- [ ] Changelog mis à jour (si applicable)
- [ ] Tests ajoutés pour les nouvelles fonctionnalités
- [ ] Pas de secrets dans le code
- [ ] Messages de commit conventionnels

## Zones de contribution

### Facile pour commencer

- Documentation
- Tests
- Exemples
- Amélioration des messages d'erreur

### Intermédiaire

- Nouvelles règles d'audit
- Nouveaux formats de sortie
- Amélioration de l'UX CLI

### Avancé

- Optimisations de performance
- Nouveaux providers
- Refactoring de l'architecture

## Questions ?

- Ouvrez une issue pour poser une question
- Consultez la [documentation de développement](Developpement)
- Explorez le code source

## Code de conduite

Nous suivons le [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/). Soyez respectueux et inclusif dans toutes vos interactions.

## Merci !

Votre contribution est appréciée. Merci de prendre le temps d'améliorer RepoLens ! 🎉
