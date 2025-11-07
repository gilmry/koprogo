# 🏢 KoproGo - PropTech 2.0 Platform for Property Management

> Plateforme SaaS de gestion de copropriété construite avec une architecture hexagonale, intégrant IA, IoT, Blockchain et achats groupés d'énergie. Modèle participatif ASBL où chaque nouveau participant réduit le coût pour tous.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Actix-web](https://img.shields.io/badge/Actix--web-4.9-blue.svg)](https://actix.rs/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15-blue.svg)](https://www.postgresql.org/)
[![Astro](https://img.shields.io/badge/Astro-4.0-purple.svg)](https://astro.build/)
[![License](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-gilmry.github.io%2Fkoprogo-blue)](https://gilmry.github.io/koprogo)

[![CI Pipeline](https://github.com/gilmry/koprogo/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/gilmry/koprogo/actions/workflows/ci.yml)
[![GDPR Compliance](https://img.shields.io/badge/GDPR-Partial%20Compliance-yellow)](docs/GDPR_COMPLIANCE_CHECKLIST.md)
[![Security Audit](https://github.com/gilmry/koprogo/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/gilmry/koprogo/actions/workflows/security.yml)
[![Documentation](https://github.com/gilmry/koprogo/actions/workflows/docs.yml/badge.svg?branch=main)](https://github.com/gilmry/koprogo/actions/workflows/docs.yml)

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

KoproGo est une solution complète de gestion de copropriété construite avec une **architecture hexagonale** (Ports & Adapters) et une approche **Domain-Driven Design (DDD)**. Le projet met l'accent sur la performance, la testabilité, la sécurité, la conformité GDPR, et introduit des **fonctionnalités PropTech 2.0** (IA, IoT, Blockchain, achats groupés d'énergie).

### 💡 Le Modèle Participatif - Économies d'Échelle Inversées

**Principe fondamental**: Contrairement aux SaaS classiques où l'échelle enrichit les actionnaires, chez KoproGo **chaque nouveau participant réduit le coût pour tous**.

```
Plus de participants → Coûts infra dilués → Prix baisse pour tous
         ↑                                              ↓
    Attractivité ←── Communauté grandit ←── Économies réelles
```

#### Exemples Concrets d'Économies d'Échelle

| Copropriétés | Coût serveur/mois | Coût/copro/mois | Économie vs 100 copros |
|-------------|------------------|-----------------|----------------------|
| 100 copros  | 95€              | **0.95€**       | Référence            |
| 500 copros  | 95€              | **0.19€**       | **-80%**             |
| 2,000 copros| 180€ (scale up)  | **0.09€**       | **-90%**             |
| 5,000 copros| 270€             | **0.054€**      | **-94%**             |

#### Redistribution Statutaire (ASBL)

Au-delà du prix coûtant, KoproGo applique une **grille dégressive** inscrite dans les statuts ASBL:

- **0-500 copros**: 1.00€/mois (prix lancement)
- **500-1,000**: 0.80€/mois (-20% automatique)
- **1,000-2,000**: 0.60€/mois (-40% automatique)
- **2,000-5,000**: 0.40€/mois (-60% automatique)
- **5,000+**: 0.20€/mois (-80% automatique)

**Chaque palier est automatique** dès que le nombre de participants est atteint. Les premiers utilisateurs bénéficient donc de toutes les économies générées par les suivants.

#### Impact pour les Premiers Participants

**Julie, syndic, copropriété #47 (rejoint en 2026)**:
- **Année 1** (100 copros): 1.00€/mois → 12€/an
- **Année 2** (500 copros): 0.80€/mois → 9.60€/an (**-20%** grâce aux 400 nouveaux)
- **Année 3** (1,000 copros): 0.60€/mois → 7.20€/an (**-40%** grâce aux 900 nouveaux)
- **Année 5** (5,000 copros): 0.40€/mois → 4.80€/an (**-60%** grâce aux 4,900 nouveaux)

**Économie Julie vs concurrent SaaS (50€/mois)**: 595.20€/an (**99.2%**)

### 🗺️ Roadmap

**📅 [Plan de développement Nov 2025 - Août 2026](docs/ROADMAP.rst)**

- **Phase 1 - Fondations & Légal** (Nov 2025 - Mar 2026): Conformité légale belge, sécurité production (LUKS, IDS, backups), GDPR complet
- **Phase 2 - PropTech Innovation + K3s** (Mar - Mai 2026): IA/ML, Blockchain, IoT, Energy Buying Groups + K3s deployment
- **Phase 3 - Scale, K8s & MLOps** (Jun - Août 2026): K8s multi-node, MLOps (Kubeflow, MLflow), Mobile Flutter, Performance P99 < 5ms
- **Phase 4 - Ecosystem** (Sep 2026+): Multi-region, Marketplace, Partnerships

Projets GitHub: [Software (#2)](https://github.com/users/gilmry/projects/2) | [Infrastructure (#3)](https://github.com/users/gilmry/projects/3)

### 📚 Documentation

**Documentation complète disponible sur [gilmry.github.io/koprogo](https://gilmry.github.io/koprogo)**
- Documentation Sphinx (guides, architecture, déploiement)
- Documentation Rust API (documentation technique du backend)
- [Guide de documentation complète](docs/README.md)

### ✨ Caractéristiques Principales

#### Core Features
- ⚡ **Performance Prouvée** : 99.74% uptime, 287 req/s sur 1 vCPU, P50=69ms, P99=752ms
- 🌱 **Ultra-Écologique** : 0.12g CO₂/requête (7-25x mieux que la concurrence)
- 💰 **Modèle Participatif** : 0.40-1€/copro/mois dégressif (vs 50€ concurrents), self-hosted gratuit
- 🏗️ **Architecture Hexagonale** : Séparation stricte des couches (Domain, Application, Infrastructure)
- 🧪 **Tests Complets** : Unitaires, Intégration, BDD (Cucumber), E2E, Load tests
- 🧑‍🤝‍🧑 **Multi-propriété native** : quote-parts cumulées, contact principal, historique complet
- 🧠 **Multi-rôles utilisateurs** : syndic/comptable/superadmin, switch rôle instantané
- 🔒 **Sécurité Production** : LUKS encryption, backups GPG+S3, IDS Suricata, CrowdSec WAF, fail2ban
- 🛡️ **GDPR Compliant** : Articles 15, 16, 17 implémentés, audit logging complet
- 📦 **Stack Moderne** : Rust + Actix-web + Astro + Svelte + PostgreSQL 15

#### 🚀 PropTech 2.0 Features (Phase 2 - Mar-Mai 2026)

**🤖 Intelligence Artificielle Éthique**
- **Assistant conversationnel** pour syndics (réponses FAQ, aide calculs)
- **OCR automatique** factures et PV d'assemblée générale
- **Prévisions budget** par machine learning sur historique charges
- **Maintenance prédictive** (alertes équipements avant panne)
- **Éthique**: Code open source, GDPR strict, Comité d'Éthique IA, pas de surveillance

**🔗 Blockchain & Transparence Radicale**
- **Votes AG immuables** sur blockchain (Polygon/Avalanche, Layer 2 écologique)
- **Smart contracts** pour décisions automatiques approuvées en AG
- **Audit trail complet** et vérifiable publiquement
- **Pas de crypto-spéculation**: Technologie au service de la gouvernance uniquement
- **Interface simplifiée**: Aucune connaissance crypto requise

**📡 IoT & Smart Buildings**
- **Capteurs intelligents**: eau, électricité, gaz, température, humidité
- **DPE automatisé**: diagnostic performance énergétique en temps réel
- **Monitoring temps réel** consommations et anomalies
- **Alertes automatiques**: fuites, surconsommations, pannes équipements
- **ROI 24 mois**: économies énergétiques -15 à -25%

**⚡ Achats Groupés Énergie**
- **Plateforme neutre** d'orchestration (pas de concurrent, rôle facilitateur)
- **Intégration partenaires**: Energie2030, Wikipower, et autres fournisseurs
- **Gouvernance démocratique**: vote AG obligatoire avant adhésion
- **GDPR opt-in individuel**: consentement explicite de chaque copropriétaire
- **Modèle 0€**: mission ASBL, aucune commission sur contrats
- **Économies cibles**: -20% facture énergie via pouvoir négociation collectif

#### 💶 Add-ons Optionnels (Prix Coûtant)

**Tarif base** (inclus):
- Self-hosted: **0€** (toujours gratuit)
- Cloud géré: **0.40-1€/mois** selon palier dégressif

**Add-ons PropTech** (optionnels):
- **AI Assistant Pack**: +2€/mois (chatbot, OCR, prévisions ML)
- **IoT Sensors**: Hardware 15-45€/capteur + 1€/mois/capteur service
- **Blockchain Voting**: +1€/mois (50 votes/mois inclus ≈ 2 AG/an)
- **Energy Buying Groups**: **0€** (mission ASBL, financé par partenariats)

**Mutualisation**: 1 GPU IA sert 5,000 copros = 0.01€/copro/mois coût réel. L'add-on +2€ finance la R&D modèles.

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
├── infrastructure/           # Infrastructure as Code
│   ├── terraform/           # Provisioning OVH Cloud
│   ├── ansible/             # Configuration servers
│   ├── k3s/                 # K3s manifests (Phase 2)
│   └── k8s/                 # K8s manifests (Phase 3)
│
├── docs/                     # Documentation Sphinx
│   ├── VISION.rst
│   ├── MISSION.rst
│   ├── ECONOMIC_MODEL.rst
│   ├── ROADMAP.rst
│   └── ...
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
git clone https://github.com/gilmry/koprogo.git
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

**Infrastructure Tier 1** (95€/mois OVH Cloud VPS) :
- **Capacité** : 1,000-1,500 copropriétés
- **Pricing ASBL** : 0.60€/copro/mois (palier 1k-2k)
- **Revenu** : 720€/mois (1,200 copros × 0.60€)
- **Coûts** : 95€ infra + 25€ divers = 120€/mois
- **Surplus** : 600€/mois → Réinvesti (vote AG)

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

### Production Security (Phase 1 - Implémenté)

- ✅ **LUKS Encryption at Rest**: Full-disk encryption PostgreSQL + uploads (AES-XTS-512)
- ✅ **Encrypted Backups**: Daily GPG-encrypted backups + S3 off-site (7d local, configurable S3 lifecycle)
- ✅ **Monitoring Stack**: Prometheus + Grafana + Loki + Alertmanager (30d metrics, 7d logs)
- ✅ **Intrusion Detection**: Suricata IDS avec règles SQL injection, XSS, path traversal
- ✅ **WAF Protection**: CrowdSec community threat intelligence
- ✅ **fail2ban**: Jails SSH, Traefik, API abuse, PostgreSQL brute-force
- ✅ **SSH Hardening**: Key-only, modern ciphers, reduced attack surface
- ✅ **Kernel Hardening**: sysctl security (SYN cookies, IP spoofing protection, ASLR)
- ✅ **Security Auditing**: Lynis (weekly), rkhunter (daily), AIDE file integrity

**Documentation** : [infrastructure/SECURITY.md](infrastructure/SECURITY.md)

### GDPR (Règlement Général sur la Protection des Données)

- ✅ Chiffrement des données sensibles (AES-256)
- ✅ Droit d'accès, rectification, effacement (Articles 15, 16, 17)
- ✅ Portabilité des données (Article 20)
- ✅ Audit logging complet
- ✅ Consentement explicite
- 🔄 Articles 18, 21 en cours (Phase 1)

### ISO 27001 (en préparation)

- 🔐 Authentification forte (MFA prévu Phase 2)
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
GET    /api/v1/units/:id/owners    # Propriétaires d'un lot (multi-owner)
POST   /api/v1/units/:id/owners    # Ajouter propriétaire à un lot
```

#### Owners (Copropriétaires)

```http
GET    /api/v1/owners              # Liste des copropriétaires
POST   /api/v1/owners              # Créer un copropriétaire
GET    /api/v1/owners/:id          # Détails copropriétaire
GET    /api/v1/owners/:id/units    # Lots d'un copropriétaire
```

#### Expenses (Charges)

```http
GET    /api/v1/expenses            # Liste des charges
POST   /api/v1/expenses            # Créer une charge
GET    /api/v1/buildings/:id/expenses # Charges d'un immeuble
PUT    /api/v1/expenses/:id/mark-paid # Marquer comme payée
```

#### Board of Directors (Conseil de Copropriété)

```http
GET    /api/v1/board/members       # Membres conseil
POST   /api/v1/board/members       # Élire membre
GET    /api/v1/board/decisions     # Décisions conseil
GET    /api/v1/board/dashboard     # Dashboard conseil
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

### Production

Déploiement via **Infrastructure as Code** :
- **VPS (Phase 1)**: Terraform + Ansible + Docker Compose + GitOps
- **K3s (Phase 2)**: K3s + ArgoCD + Traefik
- **K8s (Phase 3)**: Multi-node K8s + HA PostgreSQL (Patroni) + HPA

Voir [infrastructure/README.md](infrastructure/README.md)

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
make install-hooks     # Installer hooks Git (pre-commit, pre-push)
make build             # Build release
make docker-up         # Démarrer Docker
make docker-down       # Arrêter Docker
make migrate           # Lancer migrations
```

## 🗺️ Roadmap Détaillée

### Phase 1 - Fondations & Légal ✅ (Nov 2025 - Mar 2026)

**Infrastructure**:
- [x] Docker Compose production avec Traefik
- [x] GitOps auto-deploy (systemd service)
- [x] LUKS encryption at rest
- [x] Encrypted backups (GPG + S3)
- [x] Monitoring stack (Prometheus, Grafana, Loki)
- [x] IDS/WAF (Suricata, CrowdSec, fail2ban)
- [x] SSH & kernel hardening

**Software - Conformité Légale Belge**:
- [ ] #016: Plan Comptable Normalisé Belge (AR 12/07/2012)
- [ ] #017: État Daté génération (Article 577-2)
- [x] #022: Conseil de Copropriété (Article 577-8/4 - >20 lots)
- [ ] #018: Budget prévisionnel annuel
- [ ] #023: Workflow recouvrement automatisé

**GDPR**:
- [x] Articles 15, 17 (accès, effacement)
- [ ] Articles 16, 18, 21 (rectification, limitation, opposition)

### Phase 2 - PropTech Innovation + K3s 🚧 (Mar - Mai 2026)

**Infrastructure K3s**:
- [ ] Terraform: K3s cluster 1-node OVH
- [ ] ArgoCD GitOps deployment
- [ ] Traefik ingress controller
- [ ] Cert-manager (Let's Encrypt)

**PropTech Features** (10 nouvelles issues):

**🤖 Intelligence Artificielle** (4 issues, 136h):
- [ ] #100: AI Chatbot Assistant (syndics 24/7) - 40h
- [ ] #101: OCR Documents (factures, PV AG) - 32h
- [ ] #102: ML Budget Forecasting (prévisions charges) - 24h
- [ ] #103: Predictive Maintenance (alertes équipements) - 24h
- [ ] #111: AI Dashboard Analytics - 16h

**🔗 Blockchain & Gouvernance** (3 issues, 88h):
- [ ] #104: Blockchain Voting (Polygon Layer 2) - 40h
- [ ] #105: Smart Contracts AG (décisions automatiques) - 32h
- [ ] #106: Immutable Audit Trail (timestamping) - 16h

**📡 IoT & Énergie** (3 issues, 112h):
- [ ] #107: IoT Sensors Integration (MQTT broker) - 40h
- [ ] #108: Energy Monitoring Dashboard (temps réel) - 24h
- [ ] #109: Energy Buying Groups Platform (orchestration) - 48h

**Software - Automation**:
- [ ] #046: Electronic Voting System (AG online)
- [ ] #047: Extended PDF Generation (PCN, états datés)
- [ ] #049: Community Features (SEL, prêt objets)
- [ ] #052: Contractor Backoffice (prestataires)

### Phase 3 - Scale, K8s & MLOps 🎯 (Jun - Août 2026)

**Infrastructure K8s**:
- [ ] Terraform: Multi-node K8s cluster (3+ nodes)
- [ ] PostgreSQL HA (Patroni ou CloudNativePG operator)
- [ ] Redis/Valkey distributed cache
- [ ] Horizontal Pod Autoscaling (HPA)
- [ ] Network policies (sécurité inter-pods)

**MLOps Pipeline** (nouveauté):
- [ ] Kubeflow pipelines (training modèles IA)
- [ ] MLflow (versioning modèles, experiments tracking)
- [ ] Model serving (KServe ou Seldon Core)
- [ ] GPU nodes (NVIDIA operator)
- [ ] Distributed training (PyTorch DDP)
- [ ] A/B testing modèles IA
- [ ] Monitoring drift (Evidently AI)

**Performance**:
- [ ] P99 latency < 5ms (objectif)
- [ ] Cache distribué (Redis/Valkey)
- [ ] Query optimization PostgreSQL
- [ ] CDN pour assets frontend

**Mobile**:
- [ ] Application Flutter (iOS/Android)
- [ ] Push notifications
- [ ] Offline mode sync

**Analytics**:
- [ ] Real-time dashboards (websockets)
- [ ] Distributed tracing (Jaeger/Tempo)
- [ ] Advanced reporting

### Phase 4 - Ecosystem 🌍 (Sep 2026+)

- [ ] Multi-region deployment (Europe, expansion internationale)
- [ ] Marketplace add-ons (plugins tiers)
- [ ] Partner integrations (comptables, assureurs, syndics professionnels)
- [ ] API publique pour développeurs
- [ ] Community features avancées (SEL, événements voisins)
- [ ] White-label solutions

**Objectif Phase 4**: 10,000 copropriétés → Prix 0.10€/mois (-90% vs lancement)

## 📚 Documentation Complète

Documentation complète disponible dans le dossier `docs/` :

### Stratégie & Vision
- **[Vision](docs/VISION.rst)** ⭐ - Vision technologie au service du bien commun + Modèle participatif économies d'échelle
- **[Mission](docs/MISSION.rst)** ⭐ - 7 piliers mission ASBL incluant IA éthique, Blockchain, IoT/Énergie
- **[Modèle Économique](docs/ECONOMIC_MODEL.rst)** ⭐⭐ - Modèle ASBL prix coûtant, grille dégressive, transparence comptable (RECOMMANDÉ)
- **[Gouvernance](GOVERNANCE.md)** - Structure ASBL, processus décision, Comité d'Éthique IA
- **[Roadmap Intégrée](docs/ROADMAP.rst)** ⭐ - Plan détaillé 4 phases Nov 2025 - Août 2026

### Performance & Tests de Charge
- **[Performance Report](docs/PERFORMANCE_REPORT.md)** ⭐ - Tests charge production : 99.74% success, 287 req/s, 0.12g CO₂/req
- **[Infrastructure Roadmap](docs/INFRASTRUCTURE_ROADMAP.md)** - Évolution VPS → K3s → K8s

### Guides Techniques
- **[CLAUDE.md](CLAUDE.md)** - Instructions développeurs (Architecture hexagonale, TDD, Commandes)
- **[Multi-owner Support](docs/MULTI_OWNER_SUPPORT.md)** - Fonctionnement quotes-parts et API multi-copropriétaires
- **[Multi-role Support](docs/MULTI_ROLE_SUPPORT.md)** - Gestion utilisateurs multi-rôles
- **[VPS Deployment](docs/VPS_DEPLOYMENT.md)** - Déploiement VPS low-cost
- **[Security](infrastructure/SECURITY.md)** - Sécurité production (LUKS, IDS, WAF, backups)

### Monitoring & Opérations
- **[Monitoring Guide](monitoring/README.md)** - Scripts monitoring VPS (RAM, CPU, PostgreSQL, capacité)
- **[Capacity Calculator](monitoring/scripts/capacity_calculator.sh)** - Estimation nombre copropriétés supportées

## 📝 Contribuer

Contributions bienvenues ! Nous suivons le [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md) pour maintenir une communauté bienveillante.

Le guide [CONTRIBUTING.md](CONTRIBUTING.md) détaille les conventions de branches, les hooks Git et la checklist qualité.

### Modèle Participatif

**Vos contributions ont de la valeur** :
- Contributeur code/docs/traductions: **-50%** tarif cloud
- Mainteneur actif: **100% gratuit** à vie
- Plus de contributeurs = Meilleur produit = Bénéfice pour tous

### Gouvernance ASBL

KoproGo est organisé en **ASBL (Association Sans But Lucratif)** belge garantissant transparence et démocratie. Consultez [GOVERNANCE.md](GOVERNANCE.md) pour comprendre :
- Structure de l'ASBL (création prévue mi-2027)
- Processus de décision (CA + AG + communauté)
- Comment devenir membre ou mainteneur
- Modèle économique participatif et transparence financière
- Comité d'Éthique IA (audit algorithmes, prévention biais)

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
- Contributeurs open source du projet

---

**KoproGo** - PropTech 2.0 pour une gestion de copropriété moderne, participative, performante et écologique 🌱

**Rejoignez le mouvement** : Plus nous sommes nombreux, moins chacun paie. C'est mathématique. 🔄
