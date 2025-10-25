# Claude Code Configuration

Ce dossier contient la configuration et les guides pour utiliser Claude Code efficacement sur le projet KoproGo.

## Structure

```
.claude/
├── README.md                      # Ce fichier
├── settings.local.json            # Permissions Claude Code
├── guides/                        # Guides de développement
│   ├── feature-workflow.md       # Workflow pour nouvelles features
│   ├── bugfix-workflow.md        # Workflow pour corrections de bugs
│   ├── testing-guide.md          # Guide des tests
│   └── architecture-guide.md     # Guide architecture hexagonale
└── templates/                     # Templates réutilisables
    ├── entity-template.md        # Template pour nouvelle entité
    ├── use-case-template.md      # Template pour use case
    ├── repository-template.md    # Template pour repository
    └── handler-template.md       # Template pour handler HTTP
```

## Quick Start

### 1. Développer une nouvelle feature

Lisez [guides/feature-workflow.md](guides/feature-workflow.md) qui vous guidera étape par étape :

1. Analyse des besoins
2. Design de la solution (architecture hexagonale)
3. Tests-first (TDD)
4. Implémentation layer par layer
5. Tests d'intégration
6. Documentation

### 2. Corriger un bug

Suivez [guides/bugfix-workflow.md](guides/bugfix-workflow.md) :

1. Reproduction du bug (tests)
2. Investigation de la cause
3. Correction ciblée
4. Validation et régression

### 3. Comprendre l'architecture

Consultez [guides/architecture-guide.md](guides/architecture-guide.md) pour :

- Hexagonal Architecture (Ports & Adapters)
- Domain-Driven Design (DDD)
- Règles de dépendance entre layers
- Patterns et bonnes pratiques

## Utilisation des Templates

Les templates dans `templates/` sont des guides réutilisables pour créer de nouveaux composants.

Exemple pour créer une nouvelle entité :
```
Claude, utilise le template .claude/templates/entity-template.md pour créer une entité Payment
```

## Permissions

Le fichier `settings.local.json` contient les permissions pré-approuvées pour Claude Code :

- Commandes `make` (dev, test, lint, format, etc.)
- Commandes Cargo (build, test, clippy, fmt)
- Commandes Docker Compose
- Commandes Git (add, commit, push)
- npm/prettier
- SQLX offline mode

Ces permissions permettent à Claude de travailler de manière autonome sans demander d'approbation à chaque commande.

## Commandes Utiles

### Développement
```bash
make dev              # Lance l'environnement dev
make test            # Tous les tests
make pre-commit      # Format + lint (avant commit)
make ci              # Vérifications CI complètes
```

### Tests
```bash
make test-unit       # Tests unitaires (domain)
make test-int        # Tests d'intégration
make test-bdd        # Tests BDD/Cucumber
cargo test --lib test_name  # Un test spécifique
```

### Qualité
```bash
make format          # Format le code
make lint            # Vérifie la qualité
make audit           # Audit de sécurité
```

## Variables d'Environnement

### Backend
Copiez `backend/.env.example` vers `backend/.env` et ajustez :
```bash
DATABASE_URL=postgresql://koprogo:koprogo123@localhost:5432/koprogo_db
RUST_LOG=info
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

### Frontend
Copiez `frontend/.env.example` vers `frontend/.env` et ajustez :
```bash
PUBLIC_API_URL=http://localhost:8080/api/v1
```

## SQLX Offline Mode

Important : pour compiler sans connexion DB active, utilisez :
```bash
export SQLX_OFFLINE=true
```

Ou utilisez les commandes Makefile qui l'activent automatiquement :
```bash
make lint    # Utilise SQLX_OFFLINE=true automatiquement
make docs    # Utilise SQLX_OFFLINE=true automatiquement
```

## Documentation

- [CLAUDE.md](../CLAUDE.md) : Instructions générales pour Claude Code
- [README.md](../README.md) : Documentation générale du projet
- [docs/](../docs/) : Documentation Sphinx complète
- Rust API Docs : `make docs` (ouvre dans le navigateur)

## Support

Pour toute question sur l'utilisation de Claude Code avec KoproGo, consultez :

1. Les guides dans `.claude/guides/`
2. Le fichier `CLAUDE.md` à la racine
3. La documentation du projet dans `docs/`
4. Les exemples dans le code existant

---

**Note** : Cette structure est version-contrôlée (commitée dans git) pour être partagée entre tous les développeurs utilisant Claude Code.
