# 🏢 KoproGo - Green SaaS Property Management

> Plateforme SaaS de gestion de copropriété construite avec une architecture hexagonale, optimisée pour la performance et la conformité.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Actix-web](https://img.shields.io/badge/Actix--web-4.9-blue.svg)](https://actix.rs/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-blue.svg)](https://www.postgresql.org/)
[![Astro](https://img.shields.io/badge/Astro-4.0-purple.svg)](https://astro.build/)
[![License](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-gilmry.github.io%2Fkoprogo-blue)](https://gilmry.github.io/koprogo)

[![CI Pipeline](https://github.com/gilmry/koprogo/actions/workflows/ci.yml/badge.svg)](https://github.com/gilmry/koprogo/actions/workflows/ci.yml)
[![GDPR Compliance](https://img.shields.io/badge/GDPR-Partial%20Compliance-yellow)](docs/GDPR_COMPLIANCE_CHECKLIST.md)
![Security Audit](https://github.com/gilmry/koprogo/actions/workflows/security.yml/badge.svg)
![Documentation](https://github.com/gilmry/koprogo/actions/workflows/docs.yml/badge.svg)

[![GitHub Stars](https://img.shields.io/github/stars/gilmry/koprogo?style=social)](https://github.com/gilmry/koprogo/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/gilmry/koprogo?style=social)](https://github.com/gilmry/koprogo/network/members)
[![GitHub Issues](https://img.shields.io/github/issues/gilmry/koprogo)](https://github.com/gilmry/koprogo/issues)
[![GitHub Discussions](https://img.shields.io/github/discussions/gilmry/koprogo)](https://github.com/gilmry/koprogo/discussions)
[![Contributors](https://img.shields.io/github/contributors/gilmry/koprogo)](https://github.com/gilmry/koprogo/graphs/contributors)

**Performance validée** (1 vCPU / 2GB RAM) :
[![Success Rate](https://img.shields.io/badge/Success%20Rate-99.74%25-success)](docs/PERFORMANCE_REPORT.md)
[![Throughput](https://img.shields.io/badge/Throughput-287%20req%2Fs-blue)](docs/PERFORMANCE_REPORT.md)
[![P50 Latency](https://img.shields.io/badge/P50-69ms-green)](docs/PERFORMANCE_REPORT.md)
[![P99 Latency](https://img.shields.io/badge/P99-752ms-yellow)](docs/PERFORMANCE_REPORT.md)
[![CO2 Impact](https://img.shields.io/badge/CO2-0.12g%2Freq-brightgreen)](docs/PERFORMANCE_REPORT.md)

## 🎯 Vue d'ensemble

KoproGo est une solution complète de gestion de copropriété construite avec une **architecture hexagonale** (Ports & Adapters) et une approche **Domain-Driven Design (DDD)**. Le projet met l'accent sur la performance, la testabilité, la sécurité et la conformité.

### 🗺️ Roadmap

**📅 [Plan de développement Nov 2025 - Août 2026](docs/ROADMAP.md)**

- **Phase 1 (VPS MVP)**: Sécurité, GDPR, Backups, Board Tools (Nov 2025 - Fév 2026)
- **Phase 2 (K3s)**: Voting, Community Features, Contractor Backoffice (Mar - Mai 2026)
- **Phase 3 (K8s)**: Performance, Real-time, Mobile App (Jun - Août 2026)

Projets GitHub: [Software (#2)](https://github.com/users/gilmry/projects/2) | [Infrastructure (#3)](https://github.com/users/gilmry/projects/3)

### 📚 Documentation

**Documentation complète disponible sur [gilmry.github.io/koprogo](https://gilmry.github.io/koprogo)**
- Documentation Sphinx (guides, architecture, déploiement)
- Documentation Rust API (documentation technique du backend)
- [Guide de documentation complète](docs/README.md)

### Caractéristiques Principales

- ⚡ **Performance Prouvée** : 99.74% uptime, 287 req/s sur 1 vCPU, P50=69ms, P99=752ms
- 🌱 **Ultra-Écologique** : 0.12g CO₂/requête (7-25x mieux que la concurrence)
- 💰 **Économique** : 1€/copro/mois, 1,000-1,500 copropriétés sur 5€/mois infra
- 🏗️ **Architecture Hexagonale** : Séparation stricte des couches (Domain, Application, Infrastructure)
- 🧪 **Tests Complets** : Unitaires, Intégration, BDD (Cucumber), E2E, Load tests
- 🧑‍🤝‍🧑 **Multi-propriété native** : quote-parts cumulées, contact principal, historique complet des copropriétaires
- 🧠 **Multi-rôles utilisateurs** : syndic/comptable/superadmin sur un seul compte, switch rôle instantané
- 🔒 **Sécurité** : Conforme GDPR, ISO 27001 ready
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
# Load tests réalistes (wrk2 + Lua scripts)
cd load-tests
export BASE_URL=https://api2.koprogo.com  # ou http://localhost:8080
./scripts/realistic-load.sh

# Benchmarks Criterion (micro-benchmarks)
cargo bench

# Ou via Makefile
make bench
```

**Résultats validés** : 99.74% success rate, 287 req/s, P50=69ms sur 1 vCPU
→ Voir [docs/PERFORMANCE_REPORT.md](docs/PERFORMANCE_REPORT.md) pour détails complets

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

## 📊 Performances Validées (Load Tests Production)

### Résultats Réels (1 vCPU / 2GB RAM - OVH Cloud)

**Test de charge réaliste** : 3 minutes, 70% GET / 30% POST, 4 threads, 20 connexions

| Métrique | Valeur | Note |
|----------|--------|------|
| **Success Rate** | 99.74% | 47,681 requêtes, 125 erreurs |
| **Throughput** | 287 req/s | Soutenu sur 3 minutes |
| **Latence P50** | 69ms | Médiane |
| **Latence P90** | 130ms | 90e percentile |
| **Latence P99** | 752ms | Requêtes POST lourdes |
| **CO₂ Impact** | **0.12g/req** | **7-25x mieux que concurrents** |
| **RAM** | 128MB max | Sans swap |
| **CPU** | 8% moyen | Pic à 25% |

### Capacité & Économie

**Infrastructure Tier 1** (5€/mois OVH Cloud VPS) :
- **Capacité** : 1,000-1,500 copropriétés
- **Pricing** : 1€/copro/mois
- **Revenu** : 1,000-1,500€/mois
- **Marge brute** : **99%+** (5€ coûts / 1,000€+ revenus)

**Comparaison CO₂** (par requête) :
- KoproGo (OVH France) : **0.12g CO₂** ⭐
- SaaS cloud Europe (AWS/Azure) : 0.8-1.2g CO₂ (7-10x plus)
- SaaS cloud US (AWS/Azure) : 1.5-2g CO₂ (12-17x plus)
- Solutions legacy on-premise : 2-3g CO₂ (17-25x plus)

**Avantage France** : Mix énergétique ultra-bas carbone (60g CO₂/kWh grâce au nucléaire + renouvelables) vs 350g en Allemagne, 400g+ aux USA. L'hébergement OVH France divise les émissions serveur par **5.8x**.

### Optimisations Appliquées

- Rust natif avec compilation LTO (`opt-level=3`)
- Infrastructure OVH Cloud (datacenter européen)
- Connection pooling PostgreSQL (max 10 connections)
- Async/await non-blocking (Tokio runtime)
- Indexes PostgreSQL optimisés
- Minimal allocations dans hot paths

### Monitoring Production

Ressources pendant le test (45,070 requêtes en 3 minutes) :
```
CPU Usage:     8% average, 25% peak
RAM Usage:     128MB/2GB (6.3%)
Disk I/O:      Minimal
PostgreSQL:    < 10 connections, queries < 5ms
Network:       1.06MB/s transfer
```

**📈 Rapport détaillé** : [docs/PERFORMANCE_REPORT.md](docs/PERFORMANCE_REPORT.md)

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
make install-hooks     # Installer/mettre à jour les hooks Git (pre-commit, pre-push)
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

### Performance & Tests de Charge
- **[Performance Report](docs/PERFORMANCE_REPORT.md)** ⭐ - Rapport détaillé des tests de charge production (RECOMMANDÉ)
  - Tests réalistes : 99.74% success, 287 req/s soutenu
  - Monitoring serveur (CPU, RAM, PostgreSQL)
  - Calculs CO₂ réels : 0.12g/req (7-25x mieux que concurrents)
  - Capacité validée : 1,000-1,500 copropriétés sur 5€/mois
  - Modèle économique : 1€/copro/mois, 99%+ marge brute
  - Projections 5 ans avec données réelles

### Guides de Déploiement
- **[VPS Deployment Guide](docs/VPS_DEPLOYMENT.md)** - Déploiement sur VPS low-cost (Hetzner, OVH, DigitalOcean)
- **[Infrastructure Roadmap](docs/INFRASTRUCTURE_ROADMAP.md)** - Roadmap d'évolution (5€/mois → 270€/mois)
- **[Infrastructure K3s](infrastructure/README.md)** - Kubernetes sur OVH Cloud

### Monitoring & Opérations
- **[Monitoring Guide](monitoring/README.md)** - Scripts de monitoring VPS (RAM, CPU, PostgreSQL, capacité)
- **[Capacity Calculator](monitoring/scripts/capacity_calculator.sh)** - Estimation du nombre de copropriétés supportées

### Business & Marché
- **[Modèle Économique](docs/ECONOMIC_MODEL.md)** ⭐ - Modèle économique complet ASBL 2025-2030 (RECOMMANDÉ)
  - Structure juridique ASBL belge et gouvernance
  - Modèle OpenCore hybride (Cloud 1€/copro + Self-hosted gratuit)
  - Transparence comptable et prix coûtant
  - Viabilité financière et projections 2025-2030
  - Exemples open source réussis (Red Hat, GitLab, Mozilla)
  - Équipe 2 personnes (0.25 FTE), bénévolat puis rémunération
  - Croissance organique (SEO, communauté, bouche-à-oreille)
  - Rentable dès mois 2, 0€ financement externe
  - LTV/CAC exceptionnel : 48:1 (vs 3:1 target SaaS)
  - Opportunités de soutien (partenariats, subventions, sponsoring)

### Guides Techniques
- **[CLAUDE.md](CLAUDE.md)** - Instructions pour développeurs (Architecture, Commandes, TDD)
- **[Multi-owner Support](docs/MULTI_OWNER_SUPPORT.md)** - Fonctionnement des quotes-parts et API multi-copropriétaires
- **[Multi-role Support](docs/MULTI_ROLE_SUPPORT.md)** - Gestion des utilisateurs multi-rôles (issue #28)
- **[E2E Testing Guide](E2E_TESTING_GUIDE.md)** - Tests End-to-End
- **[Makefile Guide](MAKEFILE_GUIDE.md)** - Commandes disponibles

## 📝 Contribuer

Contributions bienvenues ! Nous suivons le [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md) pour maintenir une communauté bienveillante.

Le guide [CONTRIBUTING.md](CONTRIBUTING.md) détaille les conventions de branches, les hooks Git et la checklist qualité.

### Gouvernance

KoproGo est organisé en **ASBL (Association Sans But Lucratif)** belge garantissant transparence et démocratie. Consultez [GOVERNANCE.md](GOVERNANCE.md) pour comprendre :
- Structure de l'ASBL (création prévue mi-2026)
- Processus de décision (CA + AG + communauté)
- Comment devenir membre ou mainteneur
- Modèle économique et transparence financière

### Workflow

1. Fork le projet
2. Créer une branche dédiée (`feature/`, `fix/`, `docs/`, `chore/`…)
3. Installer les hooks Git si nécessaire (`make install-hooks`)
4. TDD : Tests d'abord !
5. Commit (`git commit -m 'feat: add amazing feature'`)
6. Push (`git push origin feature/amazing-feature`)
7. Ouvrir une Pull Request (référence à l'issue, checklist PR)

### Obtenir de l'Aide

- 📚 **Documentation**: [gilmry.github.io/koprogo](https://gilmry.github.io/koprogo)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/gilmry/koprogo/discussions)
- 🐛 **Bugs**: [Créer une issue](https://github.com/gilmry/koprogo/issues/new/choose)
- 🔒 **Sécurité**: Voir [SECURITY.md](SECURITY.md)

## 📄 Licence

Ce projet est sous licence AGPL 3.0 (GNU Affero General Public License v3.0). Voir [LICENSE](LICENSE) pour plus de détails.

Cette licence copyleft forte garantit que toute modification du code source, y compris les versions utilisées pour fournir des services réseau, doit être rendue disponible sous la même licence.

## 👥 Auteurs

- **L'équipe Koprogo ASBL** - *contact@koprogo.com*

## 🙏 Remerciements

- Architecture hexagonale inspirée par Alistair Cockburn
- DDD patterns par Eric Evans
- Actix-web team
- Astro team
- Rust community

---

**KoproGo** - Gestion de copropriété moderne, performante et écologique 🌱
