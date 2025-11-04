# Claude Code Hooks

Hooks automatiques pour maintenir la documentation et la qualité du code.

## Configuration des Hooks

Les hooks sont configurés dans `.claude/settings.local.json` et s'exécutent automatiquement lors de certaines actions.

## Hook SessionStart : Vérification des Dépendances

**Déclencheur** : Au démarrage de chaque session Claude Code

**Actions** :
1. Vérifie que GitHub CLI (`gh`) est installé
2. Vérifie les autres dépendances requises (à venir)
3. Affiche des warnings si des dépendances manquent

**Configuration dans `.claude/settings.local.json`** :

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/scripts/check-dependencies.sh --quiet"
          }
        ]
      }
    ]
  }
}
```

**Script** : [`scripts/check-dependencies.sh`](../scripts/check-dependencies.sh)

**Usage manuel** :
```bash
# Vérifier les dépendances
make check-deps
# ou: ./scripts/check-dependencies.sh

# Installer automatiquement les dépendances manquantes
make install-deps
# ou: ./scripts/check-dependencies.sh --auto-install
```

**Dépendances vérifiées** :
- **GitHub CLI (`gh`)** : Requis pour gérer les issues, PRs et releases
  - Détection multi-source (snap, apt, installation manuelle)
  - Installation automatique via dépôt officiel GitHub
  - Options : `--quiet` (mode silencieux), `--auto-install` (installation automatique)

## Hook 1 : Pre-commit Documentation

**Déclencheur** : Avant chaque commit via Claude Code

**Actions** :
1. Vérifier si des fichiers backend ont changé
2. Si oui, régénérer la documentation Rust
3. Vérifier si des fichiers docs/*.md ont changé
4. Si oui, rebuild Sphinx docs

**Configuration** :

```json
{
  "hooks": {
    "pre-commit": {
      "enabled": true,
      "commands": [
        {
          "name": "Update Rust API docs if backend changed",
          "condition": "git diff --cached --name-only | grep '^backend/'",
          "command": "cd backend && SQLX_OFFLINE=true cargo doc --no-deps"
        },
        {
          "name": "Update Sphinx docs if docs changed",
          "condition": "git diff --cached --name-only | grep '^docs/.*\\.md$'",
          "command": "cd docs && ../.venv/bin/sphinx-build -M html . _build"
        }
      ]
    }
  }
}
```

## Hook 2 : Post-commit Structure Sync

**Déclencheur** : Après chaque commit

**Actions** :
1. Analyser la structure du projet
2. Mettre à jour `docs/PROJECT_STRUCTURE.md`
3. Vérifier que la structure miroir est à jour

**Script** : `.claude/scripts/sync-structure.sh`

```bash
#!/bin/bash
# Synchronise la documentation de structure avec le projet réel

echo "🔄 Syncing project structure documentation..."

# Générer la structure actuelle
tree -L 3 -I 'target|node_modules|.git|_build' . > /tmp/structure.txt

# Mettre à jour docs/PROJECT_STRUCTURE.md
cat > docs/PROJECT_STRUCTURE.md <<EOF
# Structure du Projet KoproGo

Dernière mise à jour : $(date +"%Y-%m-%d %H:%M:%S")

\`\`\`
$(cat /tmp/structure.txt)
\`\`\`

## Backend Structure

\`\`\`
$(tree -L 4 backend/src -I 'target')
\`\`\`

## Frontend Structure

\`\`\`
$(tree -L 3 frontend/src -I 'node_modules')
\`\`\`
EOF

echo "✅ Structure documentation updated"
```

## Hook 3 : Pre-push Validation

**Déclencheur** : Avant push vers origin

**Actions** :
1. Lancer `make ci` (lint + test + audit)
2. Vérifier que CHANGELOG.md a été mis à jour
3. Vérifier qu'il n'y a pas de TODOs critiques

**Configuration** :

```json
{
  "hooks": {
    "pre-push": {
      "enabled": true,
      "commands": [
        {
          "name": "Run CI checks",
          "command": "make ci",
          "blocking": true
        },
        {
          "name": "Check CHANGELOG updated",
          "command": ".claude/scripts/check-changelog.sh",
          "blocking": false
        },
        {
          "name": "Check critical TODOs",
          "command": "! grep -r 'TODO(CRITICAL)' backend/src",
          "blocking": false
        }
      ]
    }
  }
}
```

## Hook 4 : Documentation Auto-generation

**Déclencheur** : Lors de l'ajout de nouvelles entités/use cases

**Actions** :
1. Détecter les nouveaux fichiers dans `domain/entities/` ou `application/use_cases/`
2. Générer automatiquement la documentation squelette dans `docs/domain/`
3. Ajouter l'entité/use case au index

**Script** : `.claude/scripts/generate-entity-doc.sh`

```bash
#!/bin/bash
# Génère automatiquement la documentation pour une nouvelle entité

ENTITY_FILE=$1
ENTITY_NAME=$(basename "$ENTITY_FILE" .rs)

echo "📝 Generating documentation for entity: $ENTITY_NAME"

mkdir -p docs/domain/entities

cat > "docs/domain/entities/$ENTITY_NAME.md" <<EOF
# $ENTITY_NAME Entity

Auto-generated documentation for the $ENTITY_NAME entity.

## Overview

[Describe the entity purpose]

## Business Rules

[List the business rules enforced by this entity]

## Attributes

[List the entity attributes]

## Methods

[Document public methods]

## Related Entities

[Link to related entities]

## Use Cases

[Link to use cases that use this entity]

## Tests

[Link to test files]

---

*Last updated: $(date +"%Y-%m-%d")*
*Source: \`backend/src/domain/entities/$ENTITY_NAME.rs\`*
EOF

echo "✅ Documentation template created at docs/domain/entities/$ENTITY_NAME.md"
```

## Hook 5 : Test Coverage Update

**Déclencheur** : Après l'exécution des tests

**Actions** :
1. Générer le rapport de coverage avec tarpaulin
2. Mettre à jour le badge dans README.md
3. Ajouter le rapport dans docs/testing/coverage.md

**Script** : `.claude/scripts/update-coverage.sh`

```bash
#!/bin/bash
# Met à jour la documentation de coverage

echo "📊 Updating test coverage documentation..."

cd backend

# Générer le coverage
cargo tarpaulin --out Json --output-dir ../coverage

# Extraire le pourcentage
COVERAGE=$(jq -r '.coverage' ../coverage/tarpaulin-report.json)

echo "Coverage: $COVERAGE%"

# Mettre à jour le badge dans README
sed -i "s/coverage-[0-9]*%/coverage-${COVERAGE}%/" ../README.md

echo "✅ Coverage documentation updated"
```

## Workflow Complet avec Hooks

```
1. Developer fait des changements
   ↓
2. Claude Code détecte les fichiers modifiés
   ↓
3. [PRE-COMMIT HOOK] Régénère docs si nécessaire
   ↓
4. make pre-commit (format + lint)
   ↓
5. git commit
   ↓
6. [POST-COMMIT HOOK] Sync structure docs
   ↓
7. make ci (tests + audit)
   ↓
8. [PRE-PUSH HOOK] Validation complète
   ↓
9. git push
   ↓
10. GitHub Actions CI/CD
```

## Installation des Hooks

### Automatique (via settings.local.json)

Les hooks sont automatiquement actifs quand définis dans `.claude/settings.local.json`.

### Manuel (Git hooks)

Pour intégrer avec Git :

```bash
# Créer les hooks Git
cat > .git/hooks/pre-commit <<'EOF'
#!/bin/bash
.claude/scripts/pre-commit-hook.sh
EOF

chmod +x .git/hooks/pre-commit

cat > .git/hooks/pre-push <<'EOF'
#!/bin/bash
.claude/scripts/pre-push-hook.sh
EOF

chmod +x .git/hooks/pre-push
```

## Scripts Hooks

### .claude/scripts/pre-commit-hook.sh

```bash
#!/bin/bash
set -e

echo "🔍 Running pre-commit hooks..."

# Hook 1: Check formatting
echo "  • Checking code format..."
make format --dry-run || (echo "❌ Code needs formatting. Run: make format" && exit 1)

# Hook 2: Regenerate docs if needed
if git diff --cached --name-only | grep -q '^backend/src'; then
    echo "  • Regenerating Rust API docs..."
    cd backend && SQLX_OFFLINE=true cargo doc --no-deps --quiet
fi

if git diff --cached --name-only | grep -q '^docs/.*\.md$'; then
    echo "  • Rebuilding Sphinx docs..."
    cd docs && ../.venv/bin/sphinx-build -M html . _build -q
fi

# Hook 3: Run linter
echo "  • Running linter..."
make lint

echo "✅ Pre-commit hooks passed"
```

### .claude/scripts/pre-push-hook.sh

```bash
#!/bin/bash
set -e

echo "🚀 Running pre-push hooks..."

# Hook 1: Run full CI
echo "  • Running CI checks (this may take a few minutes)..."
make ci || (echo "❌ CI checks failed" && exit 1)

# Hook 2: Check CHANGELOG
if ! git diff HEAD origin/main --name-only | grep -q 'CHANGELOG.md'; then
    echo "⚠️  Warning: CHANGELOG.md not updated"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Hook 3: Sync structure docs
echo "  • Syncing project structure docs..."
.claude/scripts/sync-structure.sh

echo "✅ Pre-push hooks passed"
```

## Configuration Complète

Exemple de `.claude/settings.local.json` avec hooks :

```json
{
  "permissions": {
    "allow": [
      "Bash(make dev:*)",
      "Bash(cargo test:*)",
      "Bash(git add:*)",
      "Bash(git commit:*)",
      "Bash(git push:*)",
      ...
    ]
  },
  "hooks": {
    "pre-commit": {
      "enabled": true,
      "script": ".claude/scripts/pre-commit-hook.sh"
    },
    "post-commit": {
      "enabled": true,
      "script": ".claude/scripts/post-commit-hook.sh"
    },
    "pre-push": {
      "enabled": true,
      "script": ".claude/scripts/pre-push-hook.sh"
    }
  }
}
```

## Désactiver les Hooks

Pour désactiver temporairement :

```bash
# Via variable d'environnement
export CLAUDE_SKIP_HOOKS=1

# Ou via settings
# Éditer .claude/settings.local.json et mettre "enabled": false
```

## Troubleshooting

### Hook échoue mais je veux commit quand même

```bash
git commit --no-verify
```

### Hook prend trop de temps

Optimiser le hook en cachant les résultats ou en le rendant asynchrone.

### Hook n'est pas exécuté

Vérifier :
1. `.claude/settings.local.json` a `"enabled": true`
2. Le script existe et est exécutable (`chmod +x`)
3. Claude Code a les permissions nécessaires

---

## Ressources

- [Feature Workflow](.claude/guides/feature-workflow.md)
- [Bugfix Workflow](.claude/guides/bugfix-workflow.md)
- [Testing Guide](.claude/guides/testing-guide.md)
