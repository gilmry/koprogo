# Git Hooks pour KoproGo

Ce document explique le système de Git hooks utilisé dans KoproGo pour garantir la qualité du code.

## 🎯 Objectif

Les Git hooks automatisent les vérifications de qualité avant chaque commit et push, empêchant l'introduction de code non formaté, non testé ou cassé dans le repository.

## 📦 Installation

### Installation automatique

Les hooks sont installés automatiquement lors du `make setup`:

```bash
make setup
```

### Installation manuelle

Si vous avez déjà fait le setup initial:

```bash
make install-hooks
```

Ou directement:

```bash
./scripts/install-hooks.sh
```

## 🪝 Hooks Configurés

### Pre-commit Hook

**Déclenché**: Avant chaque `git commit`

**Vérifications**:
1. **Format Rust** (`cargo fmt --check`)
   - Vérifie que le code Rust est formaté selon les conventions
   - Si non formaté, exécute `cargo fmt` automatiquement
   - **Action**: Vous devez re-stage les fichiers et re-commit

2. **Lint Rust** (`cargo clippy`)
   - Vérifie les warnings et erreurs Clippy
   - Mode strict: `-D warnings` (tous les warnings sont des erreurs)
   - Utilise `SQLX_OFFLINE=true` pour la compilation sans DB
   - **Action**: Corrigez les erreurs avant de commit

3. **Format Frontend** (`prettier --check`)
   - Vérifie uniquement si des fichiers frontend sont modifiés
   - Vérifie TypeScript, Astro, Svelte
   - Si non formaté, exécute `prettier --write` automatiquement
   - **Action**: Vous devez re-stage les fichiers et re-commit

**Temps d'exécution**: ~10-30 secondes

### Pre-push Hook

**Déclenché**: Avant chaque `git push`

**Vérifications**:
1. **Tests unitaires** (`cargo test --lib`)
   - Exécute tous les tests unitaires du backend
   - Mode offline: `SQLX_OFFLINE=true`
   - **Action**: Corrigez les tests avant de push

2. **Tests BDD** (`cargo test --test bdd`)
   - Exécute les tests Cucumber (Gherkin)
   - Non-bloquant: affiche un warning si échec
   - **Action**: Optionnel, mais recommandé de corriger

3. **SQLx Cache** (`cargo sqlx prepare --check`)
   - Vérifie que le query cache est à jour
   - **Action**: Exécutez `cargo sqlx prepare` si nécessaire

4. **Build Frontend** (`npm run build`)
   - Vérifie que le frontend compile sans erreurs
   - Uniquement si `node_modules/` existe
   - **Action**: Corrigez les erreurs TypeScript/Astro

**Temps d'exécution**: ~1-3 minutes

## 🚫 Bypasser les Hooks

**⚠️ À utiliser avec précaution!**

### Bypasser pre-commit

```bash
git commit --no-verify -m "message"
# Ou
git commit -n -m "message"
```

### Bypasser pre-push

```bash
git push --no-verify
# Ou
git push --no-verify origin main
```

### Quand bypasser?

- **Commits WIP**: Travail en cours sur une branche feature
- **Urgences**: Hotfix critique en production
- **CI en échec**: Si vous savez que le CI va échouer de toute façon

**⚠️ Ne JAMAIS bypasser sur `main`!**

## 🔧 Dépannage

### Hooks ne s'exécutent pas

```bash
# Vérifier que les hooks existent
ls -la .git/hooks/

# Vérifier qu'ils sont exécutables
ls -l .git/hooks/pre-commit .git/hooks/pre-push

# Réinstaller
make install-hooks
```

### Erreur "cargo fmt check failed"

```bash
# Formatter automatiquement
make format

# Ou manuellement
cd backend && cargo fmt
```

### Erreur "clippy warnings"

```bash
# Lister les warnings
cd backend && SQLX_OFFLINE=true cargo clippy

# Corriger automatiquement (quand possible)
cd backend && SQLX_OFFLINE=true cargo clippy --fix
```

### Erreur "SQLx cache out of date"

```bash
# Régénérer le cache
cd backend
export DATABASE_URL="postgresql://koprogo:koprogo123@localhost:5432/koprogo_db"
cargo sqlx prepare
```

### Erreur "Frontend build failed"

```bash
# Vérifier les erreurs TypeScript
cd frontend
npm run build

# Vérifier avec Astro check
npx astro check
```

## 📝 Personnalisation

Les hooks sont dans `.git/hooks/` et peuvent être modifiés:

```bash
# Éditer pre-commit
nano .git/hooks/pre-commit

# Éditer pre-push
nano .git/hooks/pre-push
```

**Note**: Les modifications locales ne sont pas versionnées. Pour partager des changements, modifiez `scripts/install-hooks.sh` et commitez.

## 🔄 Workflow Recommandé

### Développement quotidien

```bash
# 1. Créer une branche
git checkout -b feat/ma-fonctionnalite

# 2. Développer avec commits fréquents
git add .
git commit -m "wip: ajout fonctionnalité"
# ✅ Pre-commit s'exécute

# 3. Push vers remote
git push origin feat/ma-fonctionnalite
# ✅ Pre-push s'exécute (tests)
```

### Avant de merger sur main

```bash
# 1. Vérifier qualité complète
make lint
make test
make format

# 2. Commit final
git add .
git commit -m "feat: nouvelle fonctionnalité complète"

# 3. Push
git push origin feat/ma-fonctionnalite

# 4. Créer PR sur GitHub
gh pr create --title "feat: ma fonctionnalité"
```

## 🤝 Contribution

Si vous trouvez des améliorations pour les hooks:

1. Modifiez `scripts/install-hooks.sh`
2. Testez avec `make install-hooks`
3. Documentez dans ce fichier
4. Créez une PR

## 📚 Références

- [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
- [Cargo fmt](https://doc.rust-lang.org/cargo/commands/cargo-fmt.html)
- [Clippy](https://github.com/rust-lang/rust-clippy)
- [Prettier](https://prettier.io/)
- [SQLx Offline Mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)
