# Contribuer à KoproGo

Bienvenue ! Cette courte synthèse explique comment préparer vos contributions et suivre le workflow Git attendu par l'équipe.

---

## ⚙️ Pré-requis

1. **Cloner** le dépôt et initialiser l'environnement:
   ```bash
   git clone git@github.com:gilmry/koprogo.git
   cd koprogo
   make setup
   ```
2. Vérifier les hooks Git (`make install-hooks`) si vous n'avez pas exécuté `make setup`.

Pour plus de contexte (DDD, architecture, etc.), voyez `README.md`, `CLAUDE.md` et les guides dans `.claude/guides/`.

---

## 🌿 Workflow Git

1. **Synchroniser** `main` :
   ```bash
   git checkout main
   git pull origin main
   ```
2. **Créer une branche** à partir de `main`, selon la table ci-dessous :

   | Type de travail | Préfixe | Exemple |
   |-----------------|---------|---------|
   | Nouvelle fonctionnalité | `feature/` | `feature/board-voting` |
   | Correction de bug | `fix/` | `fix/payment-rounding` |
   | Refactoring | `refactor/` | `refactor/auth-module` |
   | Documentation | `docs/` | `docs/guides-setup` |
   | Tâches de maintenance divers | `chore/` | `chore/new-branch-workflow` |

   ```bash
   git checkout -b <prefix>/<description-kebab-case>
   ```

3. **Commits** : petits, cohérents et en anglais (`feat:`, `fix:`, `docs:`…).
4. **Hooks** : laissez tourner le `pre-commit` (format, lint) et `pre-push` (tests). Les commandes sont décrites dans `docs/GIT_HOOKS.rst`.
5. **Tests** (`make test`, `cargo test`, `npm run test`) avant le push final.
6. **Pull Request** : référencez l'issue correspondante (ex. `Closes #57`) et décrivez les impacts.

---

## 📚 Ressources utiles

- `.claude/guides/feature-workflow.md` : déroulé complet « analyse → branche → TDD → PR ».
- `.claude/guides/bugfix-workflow.md` : méthode de correction de bugs via TDD.
- `docs/README.md` : plan de la documentation Sphinx et guides associés.
- Issue `#57` et branche `chore/new-branch-workflow` : origine de ce guide.

---

Merci d'aider KoproGo à rester fiable et bien documenté !
