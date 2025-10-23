# 🏢 KoproGo - Green SaaS Property Management

> Plateforme SaaS de gestion de copropriété construite avec une architecture hexagonale, optimisée pour la performance et la conformité.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Actix-web](https://img.shields.io/badge/Actix--web-4.9-blue.svg)](https://actix.rs/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-blue.svg)](https://www.postgresql.org/)
[![Astro](https://img.shields.io/badge/Astro-4.0-purple.svg)](https://astro.build/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

![CI Pipeline](https://github.com/gilmry/koprogo/actions/workflows/ci.yml/badge.svg)
![Security Audit](https://github.com/gilmry/koprogo/actions/workflows/security.yml/badge.svg)

## 🎯 Vue d'ensemble

KoproGo est une solution complète de gestion de copropriété construite avec une **architecture hexagonale** (Ports & Adapters) et une approche **Domain-Driven Design (DDD)**. Le projet met l'accent sur la performance, la testabilité, la sécurité et la conformité.

### Caractéristiques Principales

- ⚡ **Performance** : Latence P99 < 5ms, throughput > 100k req/s (objectif)
- 🏗️ **Architecture Hexagonale** : Séparation stricte des couches (Domain, Application, Infrastructure)
- 🧪 **Tests Complets** : Unitaires, Intégration, BDD (Cucumber), E2E, Load tests
- 🔒 **Sécurité** : Conforme GDPR, ISO 27001 ready
- 🌱 **Écologique** : < 0.5g CO2/requête (objectif)
- 📦 **Stack Moderne** : Rust + Actix-web + Astro + PostgreSQL

## 📁 Structure du Projet

```
koprogo/
├── backend/                    # Backend Rust
│   ├── src/
│   │   ├── domain/            # 🎯 Logique métier pure (DDD)
│   │   │   ├── entities/      # Aggregates et Entities
│   │   │   └── services/      # Services de domaine
│   │   ├── application/       # 🎬 Use cases et orchestration
│   │   │   ├── dto/           # Data Transfer Objects
│   │   │   ├── ports/         # Interfaces (traits)
│   │   │   └── use_cases/     # Cas d'usage métier
│   │   └── infrastructure/    # 🔌 Adapters externes
│   │       ├── database/      # PostgreSQL repositories
│   │       └── web/           # API REST Actix-web
│   ├── tests/
│   │   ├── integration/       # Tests d'intégration
│   │   ├── bdd.rs            # Tests BDD (Cucumber)
│   │   └── e2e/              # Tests End-to-End
│   ├── benches/              # Benchmarks (Criterion)
│   ├── migrations/           # Migrations SQLx
│   └── Cargo.toml
│
├── frontend/                  # Frontend Astro
│   ├── src/
│   │   ├── components/       # Composants Svelte (Islands)
│   │   ├── layouts/          # Layouts Astro
│   │   └── pages/            # Pages SSG
│   └── package.json
│
├── docker-compose.yml        # Environnement de développement
├── Makefile                  # Commandes utilitaires
└── README.md
```

## 🏗️ Architecture Hexagonale

### Principe des Couches

```
┌─────────────────────────────────────────────┐
│          Infrastructure Layer               │
│  (Actix-web, PostgreSQL, Adapters)         │
│                                             │
│  ┌───────────────────────────────────────┐ │
│  │      Application Layer                │ │
│  │  (Use Cases, DTOs, Ports)            │ │
│  │                                       │ │
│  │  ┌─────────────────────────────────┐ │ │
│  │  │     Domain Layer                │ │ │
│  │  │  (Entities, Value Objects,      │ │ │
│  │  │   Business Logic)                │ │ │
│  │  └─────────────────────────────────┘ │ │
│  └───────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### Règles Strictes

1. **Domain** → Aucune dépendance externe
2. **Application** → Dépend uniquement du Domain
3. **Infrastructure** → Implémente les ports définis par Application

### Exemple : Building Aggregate

```rust
// Domain Layer - Entité métier pure
pub struct Building {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub total_units: i32,
    // ... invariants métier
}

impl Building {
    pub fn new(name: String, address: String, ...) -> Result<Self, String> {
        // Validation des invariants
        if name.is_empty() {
            return Err("Building name cannot be empty".to_string());
        }
        // Création sécurisée
        Ok(Self { ... })
    }
}

// Application Layer - Port (interface)
#[async_trait]
pub trait BuildingRepository: Send + Sync {
    async fn create(&self, building: &Building) -> Result<Building, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
}

// Infrastructure Layer - Adapter PostgreSQL
pub struct PostgresBuildingRepository {
    pool: DbPool,
}

#[async_trait]
impl BuildingRepository for PostgresBuildingRepository {
    async fn create(&self, building: &Building) -> Result<Building, String> {
        sqlx::query("INSERT INTO buildings ...")
            .execute(&self.pool)
            .await?;
        Ok(building.clone())
    }
}
```

## 🚀 Démarrage Rapide

### Prérequis

- Rust 1.75+
- Node.js 20+
- Docker & Docker Compose
- PostgreSQL 15 (ou via Docker)

### Installation

```bash
# 1. Cloner le projet
git clone https://github.com/votre-user/koprogo.git
cd koprogo

# 2. Démarrer PostgreSQL avec Docker
make docker-up

# 3. Configuration de l'environnement
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env

# 4. Lancer les migrations
make migrate

# 5. Démarrer le backend
cd backend
cargo run

# 6. Dans un autre terminal, démarrer le frontend
cd frontend
npm install
npm run dev
```

### Accès

- 🌐 **Frontend** : http://localhost:3000
- 🔌 **API** : http://localhost:8080/api/v1
- ❤️ **Health Check** : http://localhost:8080/api/v1/health

## 🧪 Tests

KoproGo dispose d'une suite de tests complète suivant la pyramide de tests :

### Tests Unitaires (100% coverage Domain)

```bash
# Tous les tests unitaires
cargo test --lib

# Tests avec coverage
make coverage
```

Les tests unitaires sont intégrés directement dans les modules du domaine :

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_create_building_success() { ... }

    #[test]
    fn test_create_building_validation_fails() { ... }
}
```

### Tests d'Intégration

```bash
# Tests d'intégration avec PostgreSQL (testcontainers)
cargo test --test integration

# Ou via Makefile
make test-integration
```

### Tests BDD (Behavior-Driven Development)

```bash
# Tests Cucumber/Gherkin
cargo test --test bdd

# Ou
make test-bdd
```

Exemple de feature Gherkin :

```gherkin
Feature: Building Management
  Scenario: Create a new building
    Given a coproperty management system
    When I create a building named "Résidence Les Jardins" in "Paris"
    Then the building should be created successfully
```

### Tests E2E (End-to-End)

```bash
# Tests API complets
cargo test --test e2e

# Ou
make test-e2e
```

### Load Tests / Benchmarks

```bash
# Benchmarks Criterion
cargo bench

# Ou
make bench
```

## 🔄 CI/CD Pipelines

KoproGo dispose d'une infrastructure CI/CD complète avec GitHub Actions pour garantir la qualité du code et automatiser les déploiements.

### Workflows Automatiques

#### 🚀 CI Pipeline (`.github/workflows/ci.yml`)
Se déclenche automatiquement sur chaque push et pull request :

**Tests Backend Rust :**
- ✅ **Lint & Format** : `cargo fmt --check` + `cargo clippy`
- ✅ **Unit Tests** : Tests unitaires isolés (`cargo test --lib`)
- ✅ **Integration Tests** : Tests avec PostgreSQL et migrations
- ✅ **BDD Tests** : Tests Cucumber/Gherkin
- ✅ **E2E Tests** : Tests complets des endpoints API

**Tests Frontend :**
- ✅ **TypeScript Check** : Vérification Astro (`astro check`)
- ✅ **Build** : Compilation du frontend
- ✅ **Format Check** : Validation Prettier

**Build Final :**
- ✅ **Release Build** : Compilation optimisée
- ✅ **Artifacts** : Upload du binaire (7 jours de rétention)

#### 🔒 Security Audit (`.github/workflows/security.yml`)
Analyse de sécurité automatique :

- 🛡️ **Cargo Audit** : Scan des vulnérabilités Rust
- 🛡️ **NPM Audit** : Scan des vulnérabilités JavaScript
- 🛡️ **Dependency Review** : Analyse des dépendances dans les PR
- 📅 **Planification** : Hebdomadaire (dimanche minuit)

#### 📊 Benchmarks (`.github/workflows/benchmarks.yml`)
Tests de performance :

- ⚡ **Criterion Benchmarks** : Mesure des performances
- 📈 **Rapports HTML** : Visualisations détaillées
- 📦 **Artifacts** : Résultats conservés 30 jours
- 🎯 **Déclenchement** : Manuel ou planifié (lundi 2h UTC)

### Optimisations

- **Caching** : Cache intelligent de Cargo (registry, index, build) et NPM
- **Parallélisation** : Tous les tests s'exécutent en parallèle
- **Services PostgreSQL** : Configuration automatique pour les tests
- **Migrations** : Application automatique via SQLx

### Surveiller les Workflows

```bash
# Via GitHub CLI
gh run list --workflow=ci.yml
gh run watch

# Ou visitez directement
# https://github.com/gilmry/koprogo/actions
```

Voir [.github/workflows/README.md](.github/workflows/README.md) pour la documentation complète.

## 📊 Performances

### Objectifs

- **Latence P99** : < 5ms
- **Throughput** : > 100k req/s
- **Memory** : < 128MB par instance
- **CO2** : < 0.5g par requête

### Optimisations

- Compilation en mode release avec LTO
- Connection pooling (max 10 connections)
- Async/await non-blocking (Tokio)
- Minimal allocations dans hot paths
- Lazy loading et caching stratégique

## 🔒 Sécurité & Conformité

### GDPR (Règlement Général sur la Protection des Données)

- ✅ Chiffrement des données sensibles (AES-256)
- ✅ Droit d'accès, rectification, effacement
- ✅ Portabilité des données
- ✅ Audit logging complet
- ✅ Consentement explicite

### ISO 27001 (en préparation)

- 🔐 Authentification forte (MFA prévu)
- 🔒 TLS 1.3 obligatoire
- 📝 Journalisation des accès
- 🛡️ Tests de sécurité réguliers

## 📖 API Documentation

### Endpoints Principaux

#### Buildings (Immeubles)

```http
GET    /api/v1/buildings           # Liste des immeubles
POST   /api/v1/buildings           # Créer un immeuble
GET    /api/v1/buildings/:id       # Détails d'un immeuble
PUT    /api/v1/buildings/:id       # Modifier un immeuble
DELETE /api/v1/buildings/:id       # Supprimer un immeuble
```

#### Units (Lots)

```http
GET    /api/v1/units               # Liste des lots
POST   /api/v1/units               # Créer un lot
GET    /api/v1/buildings/:id/units # Lots d'un immeuble
PUT    /api/v1/units/:id/assign-owner/:owner_id # Assigner propriétaire
```

#### Owners (Copropriétaires)

```http
GET    /api/v1/owners              # Liste des copropriétaires
POST   /api/v1/owners              # Créer un copropriétaire
GET    /api/v1/owners/:id          # Détails copropriétaire
```

#### Expenses (Charges)

```http
GET    /api/v1/expenses            # Liste des charges
POST   /api/v1/expenses            # Créer une charge
GET    /api/v1/buildings/:id/expenses # Charges d'un immeuble
PUT    /api/v1/expenses/:id/mark-paid # Marquer comme payée
```

### Exemple de Requête

```bash
# Créer un immeuble
curl -X POST http://localhost:8080/api/v1/buildings \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Résidence Les Jardins",
    "address": "123 Rue de la Paix",
    "city": "Paris",
    "postal_code": "75001",
    "country": "France",
    "total_units": 50,
    "construction_year": 1985
  }'
```

## 🐳 Docker

### Développement

```bash
# Démarrer tous les services
docker-compose up

# Voir les logs
docker-compose logs -f

# Arrêter
docker-compose down
```

### Production (à venir)

Configuration Kubernetes + Helm charts pour déploiement OVH Cloud.

## 🛠️ Commandes Utiles

```bash
# Makefile helpers
make help              # Afficher l'aide
make dev               # Démarrer en mode dev
make test              # Tous les tests
make test-unit         # Tests unitaires uniquement
make test-integration  # Tests d'intégration
make test-bdd          # Tests BDD
make test-e2e          # Tests E2E
make bench             # Benchmarks
make coverage          # Coverage report
make lint              # Linters
make format            # Formatter le code
make build             # Build release
make docker-up         # Démarrer Docker
make docker-down       # Arrêter Docker
```

## 🗺️ Roadmap

### Phase 1 - MVP ✅ (Actuel)
- [x] Architecture hexagonale
- [x] Domain models (Building, Unit, Owner, Expense, Meeting, Document)
- [x] API REST complète
- [x] Frontend Astro basique
- [x] Tests complets (Unit, Integration, BDD, E2E, Load)
- [x] Docker Compose

### Phase 2 - Performance & Scale 🚧
- [ ] ScyllaDB pour données à haute vélocité
- [ ] DragonflyDB pour cache distribué
- [ ] MinIO pour stockage documents
- [ ] Optimisation < 5ms P99 latency

### Phase 3 - Production 🚧
- [x] CI/CD GitHub Actions (Pipelines complètes)
- [x] Infrastructure as Code (Terraform modules OVH)
- [x] Helm charts (Kubernetes)
- [x] Ansible playbooks (K3s, sécurité)
- [ ] Déploiement production OVH Cloud
- [ ] Monitoring (Prometheus + Grafana)

### Phase 4 - Conformité 🔒
- [ ] Authentification JWT + MFA
- [ ] Audit logging complet
- [ ] Conformité GDPR complète
- [ ] Certification ISO 27001

### Phase 5 - Features Avancées 🎯
- [ ] Génération documents automatique
- [ ] Notifications temps réel
- [ ] Dashboard analytics
- [ ] Export comptable
- [ ] Mobile app (Flutter)

## 📚 Documentation

Documentation complète disponible dans le dossier `docs/` :

### Guides de Déploiement
- **[VPS Deployment Guide](docs/VPS_DEPLOYMENT.md)** - Déploiement sur VPS low-cost (Hetzner, OVH, DigitalOcean)
- **[Infrastructure Roadmap](docs/INFRASTRUCTURE_ROADMAP.md)** - Roadmap d'évolution (5€/mois → 270€/mois)
- **[Infrastructure K3s](infrastructure/README.md)** - Kubernetes sur OVH Cloud

### Monitoring & Opérations
- **[Monitoring Guide](monitoring/README.md)** - Scripts de monitoring VPS (RAM, CPU, PostgreSQL, capacité)
- **[Capacity Calculator](monitoring/scripts/capacity_calculator.sh)** - Estimation du nombre de copropriétés supportées

### Business & Marché
- **[Business Plan](docs/BUSINESS_PLAN.md)** - Plan d'affaires complet 2025-2028
  - Executive Summary & Vision
  - Projections financières détaillées (120 copros → 3,000 copros)
  - Stratégie commerciale dual-market (Europe + Maghreb)
  - Modèle économique (LTV/CAC, Unit Economics)
  - Besoins financement (Seed 50k€, Series A 500k€-1M€)
  - Roadmap produit, géographique, équipe
- **[Market Analysis](docs/MARKET_ANALYSIS.md)** - Analyse marché Europe & Afrique du Nord (Belgique, France, Espagne, Italie, Allemagne, Tunisie)
  - Réglementation par pays (syndic obligatoire, seuils)
  - Concurrence et opportunités
  - Pricing recommandé (Europe vs Maghreb)
  - Stratégie expansion géographique
  - Règles métier à implémenter

### Guides Techniques
- **[CLAUDE.md](CLAUDE.md)** - Instructions pour développeurs (Architecture, Commandes, TDD)
- **[E2E Testing Guide](E2E_TESTING_GUIDE.md)** - Tests End-to-End
- **[Makefile Guide](MAKEFILE_GUIDE.md)** - Commandes disponibles

## 📝 Contribuer

Contributions bienvenues ! Voir [CONTRIBUTING.md](CONTRIBUTING.md) (à venir).

### Workflow

1. Fork le projet
2. Créer une branche feature (`git checkout -b feature/amazing-feature`)
3. TDD : Tests d'abord !
4. Commit (`git commit -m 'Add amazing feature'`)
5. Push (`git push origin feature/amazing-feature`)
6. Ouvrir une Pull Request

## 📄 Licence

Ce projet est sous licence MIT. Voir [LICENSE](LICENSE) pour plus de détails.

## 👥 Auteurs

- **Votre Nom** - *Initial work*

## 🙏 Remerciements

- Architecture hexagonale inspirée par Alistair Cockburn
- DDD patterns par Eric Evans
- Actix-web team
- Astro team
- Rust community

---

**KoproGo** - Gestion de copropriété moderne, performante et écologique 🌱
