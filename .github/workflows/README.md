# GitHub Actions Workflows

Ce répertoire contient les workflows CI/CD pour le projet KoproGo.

## Workflows disponibles

### 1. CI Pipeline (`ci.yml`)

**Déclenchement:**
- Push sur `main`, `develop`, ou branches `claude/**`
- Pull requests vers `main` ou `develop`

**Jobs:**

#### Lint & Format Check
- Vérification du formatage Rust (`cargo fmt --check`)
- Analyse statique avec Clippy (`cargo clippy`)

#### Unit Tests
- Tests unitaires Rust (`cargo test --lib`)
- Tests des modules isolés sans dépendances externes

#### Integration Tests
- Tests d'intégration avec PostgreSQL
- Utilise les migrations SQLx
- Teste les interactions entre composants

#### BDD Tests
- Tests Behavior-Driven Development avec Cucumber
- Tests basés sur les fichiers `.feature`
- Validation des scénarios métier

#### E2E Tests
- Tests End-to-End de l'API complète
- Utilise reqwest pour tester les endpoints HTTP
- Valide le comportement de bout en bout

#### Frontend Check & Build
- Vérification TypeScript avec `astro check`
- Build du frontend Astro
- Vérification du formatage avec Prettier

#### Build
- Compilation release du backend
- Upload de l'artifact binaire
- S'exécute uniquement si tous les tests passent

---

### 2. Benchmarks (`benchmarks.yml`)

**Déclenchement:**
- Manuel via `workflow_dispatch`
- Planifié: Chaque lundi à 2h UTC

**Jobs:**

#### Performance Benchmarks
- Exécute les benchmarks Criterion
- Génère des rapports HTML détaillés
- Upload des résultats dans les artifacts (conservation: 30 jours)
- Commente les PR avec les résultats si applicable

**Usage manuel:**
```bash
# Via GitHub UI: Actions > Benchmarks > Run workflow
```

---

### 3. Security Audit (`security.yml`)

**Déclenchement:**
- Push sur `main` ou `develop`
- Pull requests vers `main` ou `develop`
- Planifié: Chaque dimanche à minuit UTC

**Jobs:**

#### Rust Security Audit
- Utilise `cargo-audit` pour détecter les vulnérabilités
- Vérifie les dépendances Rust contre la base RustSec

#### NPM Security Audit
- Exécute `npm audit` sur le frontend
- Seuil d'alerte: moderate

#### Dependency Review
- Analyse les changements de dépendances dans les PR
- Vérifie les licences et vulnérabilités connues
- Bloque si vulnérabilités >= moderate

---

## Architecture des tests

```
Backend Tests:
├── Unit Tests (--lib)
│   ├── Domain entities
│   ├── Domain services
│   └── Use cases
├── Integration Tests
│   ├── Repository implementations
│   └── Database operations
├── BDD Tests
│   ├── Building management scenarios
│   └── Business workflows
└── E2E Tests
    └── API endpoints

Frontend:
├── TypeScript check (astro check)
├── Build verification
└── Code formatting
```

## Caching Strategy

Tous les workflows utilisent un système de cache pour optimiser les temps d'exécution:

- **Cargo Registry**: Cache des crates téléchargées
- **Cargo Index**: Cache de l'index crates.io
- **Cargo Build**: Cache des artefacts de compilation
- **NPM**: Cache des node_modules

## Services Docker

Les tests nécessitant une base de données utilisent PostgreSQL 16 comme service GitHub Actions:

```yaml
services:
  postgres:
    image: postgres:16
    env:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: koprogo_test
```

## Artifacts

### CI Pipeline
- **koprogo-api**: Binaire release (7 jours)

### Benchmarks
- **benchmark-results**: Rapports Criterion HTML (30 jours)

## Variables d'environnement

```bash
# Backend
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/koprogo_test
CARGO_TERM_COLOR=always
RUST_BACKTRACE=1
```

## Badges de statut

Ajoutez ces badges dans votre README.md:

```markdown
![CI](https://github.com/gilmry/koprogo/actions/workflows/ci.yml/badge.svg)
![Security](https://github.com/gilmry/koprogo/actions/workflows/security.yml/badge.svg)
```

## Maintenance

- Les workflows utilisent les versions stables des actions
- Mettez à jour régulièrement les versions des actions
- Surveillez les notifications de dépréciations GitHub

## Dépannage

### Tests qui échouent
1. Vérifiez les logs de l'étape qui a échoué
2. Reproduisez localement avec les mêmes commandes
3. Vérifiez que les migrations sont à jour

### Problèmes de cache
```bash
# Nettoyez le cache via Settings > Actions > Caches
# Ou utilisez le hash dans la clé de cache
```

### Timeouts
- Les tests E2E peuvent prendre du temps
- Ajustez le timeout si nécessaire: `timeout-minutes: 30`
