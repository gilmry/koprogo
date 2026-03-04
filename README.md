# 🏢 KoproGo - Plateforme Communautaire pour l'Habitat Collectif

> **Trois façons d'utiliser KoproGo** : Gestion complète de copropriété • Modules communautaires seuls (SEL, partage) • Soutien aux valeurs (écologie, opensource, solidarité)

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
[![YouTube](https://img.shields.io/badge/YouTube-@koprogo-red?logo=youtube)](https://www.youtube.com/@koprogo)

**Performance validée** (1 vCPU / 2GB RAM) :
[![Success Rate](https://img.shields.io/badge/Success%20Rate-99.74%25-success)](docs/PERFORMANCE_REPORT.rst)
[![Throughput](https://img.shields.io/badge/Throughput-287%20req%2Fs-blue)](docs/PERFORMANCE_REPORT.rst)
[![P50 Latency](https://img.shields.io/badge/P50-69ms-green)](docs/PERFORMANCE_REPORT.rst)
[![P99 Latency](https://img.shields.io/badge/P99-752ms-yellow)](docs/PERFORMANCE_REPORT.rst)
[![CO2 Impact](https://img.shields.io/badge/CO2-0.12g%2Freq-brightgreen)](docs/PERFORMANCE_REPORT.rst)

---

## 🎬 Tutoriels Vidéo

[![Playlist YouTube KoproGo](https://img.shields.io/badge/YouTube-Playlist%20Tutoriels-red?style=for-the-badge&logo=youtube)](https://www.youtube.com/playlist?list=PLBxLivSdjkRtGNabcvJuV68_9_VFmGVc-)

> 📺 **[Voir la playlist complète sur YouTube](https://www.youtube.com/playlist?list=PLBxLivSdjkRtGNabcvJuV68_9_VFmGVc-)**

---

## 🎯 KoproGo Complète Votre Immeuble

**KoproGo s'adapte à votre situation** - Vous n'avez pas besoin de tout changer pour bénéficier de KoproGo.

### 🧩 Trois Façons d'Utiliser KoproGo

#### **1. Modules Communautaires Seuls** 🤝 (Le Plus Populaire)

**Gardez votre syndic actuel, ajoutez le lien social :**

- ✅ **SEL** (Système d'Échange Local) : Troquez compétences entre voisins (jardinage, bricolage, cours)
- ✅ **Partage d'Objets** : Prêtez outils, tondeuse, échelle entre habitants
- ✅ **Bazar de Troc** : Échangez ou donnez objets inutilisés
- ✅ **Annuaire Compétences** : Qui sait faire quoi dans l'immeuble ?
- ✅ **Covoiturage & Garde** : Petites annonces locales

**Installation** : 15 minutes • **Coût** : 0€ (self-hosted) ou 5€/mois (cloud avec support)
**Compatible avec** : Solutions SaaS existantes, tableurs Excel, n'importe quel outil de gestion

**Cas d'usage** : *"Mon immeuble de 30 lots utilise un logiciel SaaS pour la gestion officielle. On a installé KoproGo juste pour le SEL et le partage d'objets. Ça marche parfaitement ensemble !"*

---

#### **2. Gestion de Copropriété Complète** 🏗️

**Solution complète pour remplacer vos outils actuels :**

- ✅ **CRUD Complet** : Immeubles, lots, copropriétaires, charges, documents
- ✅ **Multi-propriété Native** : Quote-parts, contact principal, historique complet
- ✅ **Multi-rôles** : Syndic, comptable, superadmin, switch instantané
- ✅ **Workflow Facturation** : TVA belge (6%, 12%, 21%), recouvrement automatisé
- ✅ **Comptabilité Belge** : PCMN (Plan Comptable Normalisé), AR 12/07/2012
- ✅ **Conseil de Copropriété** : Obligatoire >20 lots (Article 577-8/4)
- ✅ **Assemblées Générales** : Convocations, PV, votes électroniques

**Installation** : 20 minutes • **Coût** : 0€ (self-hosted) ou 5€/mois (cloud)
**Target** : Petites copropriétés (< 50 lots), syndics bénévoles, techniciens autonomes

**Cas d'usage** : *"Petite copro 12 lots, on cherchait une alternative gratuite aux solutions SaaS classiques (200€/mois). Self-hosted KoproGo depuis 6 mois, ça fonctionne parfaitement. Économie : 2,400€/an."*

---

#### **3. Soutien aux Valeurs** 💚 (Sympathisants)

**Soutenez l'écologie, l'opensource, la solidarité sans utiliser les outils :**

- ✅ **Écologie** : 0.12g CO₂/req (96% réduction vs solutions SaaS classiques)
- ✅ **Opensource** : Code AGPL-3.0, auditable, transparent
- ✅ **Solidarité** : Fonds de Solidarité pour membres en difficulté
- ✅ **Démocratie** : 1 membre = 1 voix, prix voté en AG
- ✅ **ASBL Non-Profit** : Surplus réinvesti dans le projet

**Action** : Devenir membre cotisant (5€/mois) = 1 voix en Assemblée Générale
**Bonus** : Influencez la roadmap, votez l'allocation du surplus, participez aux décisions

**Cas d'usage** : *"Je n'ai pas de copropriété (locataire), mais je soutiens l'opensource et l'écologie. Je cotise 5€/mois et je vote en AG sur la roadmap. C'est ma façon de contribuer."*

---

## 💡 Le Modèle Démocratique ASBL - Prix Fixe Solidaire

**Principe fondamental** : Contrairement aux SaaS classiques où l'échelle enrichit les actionnaires, chez KoproGo **le prix est fixe et démocratiquement décidé**.

```
Prix fixe 5€/mois → Surplus réinvesti → AG vote baisse
         ↑                                        ↓
    Confiance ←── Communauté grandit ←── Transparence totale
```

### Coûts Infrastructure Réels (Transparence Totale)

| Palier | Copros | Infra/mois | Coût réel/copro | Prix facturé | Surplus/mois |
|--------|--------|------------|-----------------|--------------|--------------|
| **Validation** | 100 | 8€ | 0.08€ | **5€** | 492€ |
| **Viabilité** | 500 | 13€ | 0.03€ | **5€** | 2,487€ |
| **Impact** | 1,000 | 18€ | 0.02€ | **5€** | 4,982€ |
| **Leadership** | 2,000 | 29€ | 0.01€ | **5€** | 9,971€ |
| **Référence** | 5,000 | 163€ | 0.03€ | **5€** | 16,037€ |

**Marge : 96-99%** → Surplus réinvesti dans développement, salaires dev, Fonds de Solidarité, ou baisse prix (vote AG)

### Allocation du Surplus (Votée en AG)

**Exemple AG 2028** (1,500 copros, surplus 72,000€/an) :

- **30% Fonds de Solidarité** (21,600€) : Aide financière membres en difficulté
- **25% Baisse tarifaire** (18,000€) : Prix 5€ → 4€/mois
- **25% Features prioritaires** (18,000€) : Vote communauté
- **15% Réserve légale** (10,800€) : Sécurité
- **5% R&D PropTech** (3,600€) : IA, Blockchain, IoT

**Vote AG** (1 membre = 1 voix) : ✅ Adopté 87% pour, 13% abstention

### 🤝 Fonds de Solidarité (Nouveau !)

**Objectif** : Garantir l'accès à la justice et prévenir l'exclusion financière.

**Types d'aides** :
- **Aide Litiges AG** (500-2,000€) : Avocat pour contester vote illégal
- **Prêts 0% Frais Admin** (jusqu'à 5,000€) : Charges impayées, huissiers
- **Crédits Travaux Solidaires** (5,000-50,000€) : Quote-part travaux urgents (taux 1-2%)
- **Subventions Urgence** (max 3,000€) : Précarité extrême, non-remboursable

**Impact projeté 2030** : 40-60 personnes aidées/an, 20 litiges évités, 5-8 familles sauvées expulsion

📖 **[Documentation complète](docs/FONDS_SOLIDARITE.rst)**

---

## 🗺️ Roadmap par Capacités

**🗺️ [Roadmap complète par jalons](docs/ROADMAP_PAR_CAPACITES.rst)**

**Philosophie** : KoproGo progresse par **jalons de capacités**, pas par dates fixes. Chaque jalon débloque le suivant quand les fonctionnalités sont validées.

- **Jalon 0** ✅ : 10-20 early adopters (Architecture hexagonale, 73 endpoints API)
- **Jalon 1** : 50-100 copros (LUKS, backups GPG, GDPR basique)
- **Jalon 2** : 200-500 copros (Facturation TVA, recouvrement, K3s)
- **Jalon 3** : 500-1,000 copros (IA, Blockchain, IoT → Constitution ASBL)
- **Jalons 4-5** : 1,000-5,000 copros (Mobile app, API publique, K8s)
- **Jalons 6-7** : 5,000+ copros (PropTech 2.0 complet, multi-région)

Projets GitHub: [Software (#2)](https://github.com/users/gilmry/projects/2) | [Infrastructure (#3)](https://github.com/users/gilmry/projects/3)

---

## 📚 Documentation

**Documentation complète disponible sur [gilmry.github.io/koprogo](https://gilmry.github.io/koprogo)**

### 🎯 Parcours Dédiés (Choisissez le Vôtre)

La documentation propose **4 parcours de lecture** adaptés à votre profil :

- 💼 **[Parcours Investisseur](docs/parcours-investisseur.rst)** (5-10 min) - Évaluation impact social, viabilité financière, opportunités
- 🧑‍💻 **[Parcours Développeur](docs/parcours-contributeur.rst)** (15-30 min) - Architecture hexagonale, stack technique, contribuer
- 🏢 **[Parcours Syndic/Copropriétaire](docs/index.rst)** (10-20 min) - Fonctionnalités, mise en route, support
- 🤔 **[Parcours Curieux](docs/vision-strategie/de-gestion-a-symbiose.rst)** (20-30 min) - Vision long terme, impact sociétal

**Documents clés** :
- [De la Gestion à la Symbiose](docs/vision-strategie/de-gestion-a-symbiose.rst) - Comment un outil de gestion devient un réseau symbiotique
- [Roadmap par Jalons](docs/roadmap/roadmap-2025-2030.rst) - Progression par capacités 2025-2030
- [Index complet](docs/index.rst) - Point d'entrée pour toute la documentation

### 🎥 Tutoriels Vidéo

**Chaîne YouTube** : [@koprogo](https://www.youtube.com/@koprogo)

Retrouvez des tutoriels vidéo pour démarrer avec KoproGo, comprendre l'architecture, et découvrir les fonctionnalités avancées

---

## ✨ Caractéristiques Principales

### Core Features
- ⚡ **Performance Prouvée** : 99.74% uptime, 287 req/s sur 1 vCPU, P50=69ms, P99=752ms
- 🌱 **Ultra-Écologique** : 0.12g CO₂/requête (7-25x mieux que solutions SaaS classiques)
- 💰 **Prix Fixe Démocratique** : 5€/mois cloud, self-hosted 0€, baisse par vote AG
- 🏗️ **Architecture Hexagonale** : Séparation stricte des couches (Domain, Application, Infrastructure)
- 🧪 **Tests Complets** : Unitaires, Intégration, BDD (Cucumber), E2E, Load tests
- 🧑‍🤝‍🧑 **Multi-propriété native** : quote-parts cumulées, contact principal, historique complet
- 🧠 **Multi-rôles utilisateurs** : syndic/comptable/superadmin, switch rôle instantané
- 🔒 **Sécurité Production** : LUKS encryption, backups GPG+S3, IDS Suricata, CrowdSec WAF, fail2ban
- 🛡️ **GDPR Compliant** : Articles 15, 16, 17 implémentés, audit logging complet
- 📦 **Stack Moderne** : Rust + Actix-web + Astro + Svelte + PostgreSQL 15

### 🤝 Modules Communautaires (Standalone)

**Utilisables sans changer votre gestion actuelle :**

- ✅ **SEL** (Système d'Échange Local) : Échange de compétences entre voisins
- ✅ **Partage d'Objets** : Prêt d'outils, tondeuse, échelle
- ✅ **Bazar de Troc** : Échange/don d'objets inutilisés
- ✅ **Annuaire Compétences** : Qui sait faire quoi ?
- ✅ **Covoiturage & Garde** : Petites annonces locales
- ✅ **Tableau d'affichage numérique** : Communication résidents

**Impact social** (30% adoption, 5,000 copros) :
- 750k€ économie circulaire (SEL)
- 600k€ consommation évitée (partage)
- 12,000 objets partagés
- -790 tonnes CO₂/an (partage + réduction consommation)

### 🚀 PropTech 2.0 Features (Jalons 3-7)

**🤖 Intelligence Artificielle Éthique**
- **Assistant conversationnel** pour syndics (réponses FAQ, aide calculs)
- **OCR automatique** factures et PV d'assemblée générale
- **Prévisions budget** par machine learning sur historique charges
- **Maintenance prédictive** (alertes équipements avant panne)
- **Éthique** : Code open source, GDPR strict, Comité d'Éthique IA, pas de surveillance

**🔗 Blockchain & Transparence Radicale**
- **Votes AG immuables** sur blockchain (Polygon/Avalanche, Layer 2 écologique)
- **Smart contracts** pour décisions automatiques approuvées en AG
- **Audit trail complet** et vérifiable publiquement
- **Pas de crypto-spéculation** : Technologie au service de la gouvernance uniquement
- **Interface simplifiée** : Aucune connaissance crypto requise

**📡 IoT & Smart Buildings**
- **Capteurs intelligents** : eau, électricité, gaz, température, humidité
- **DPE automatisé** : diagnostic performance énergétique en temps réel
- **Monitoring temps réel** consommations et anomalies
- **Alertes automatiques** : fuites, surconsommations, pannes équipements
- **ROI 24 mois** : économies énergétiques -15 à -25%

**⚡ Achats Groupés Énergie**
- **Plateforme neutre** d'orchestration (rôle facilitateur)
- **Intégration partenaires** : Energie2030, Wikipower, autres fournisseurs
- **Gouvernance démocratique** : vote AG obligatoire avant adhésion
- **GDPR opt-in individuel** : consentement explicite de chaque copropriétaire
- **Modèle 0€** : mission ASBL, aucune commission sur contrats
- **Économies cibles** : -20% facture énergie via pouvoir négociation collectif

### 💶 Tarification Transparente

**Tarif base** (inclus) :
- **Self-hosted** : **0€** (toujours gratuit, accès code source)
- **Cloud géré** : **5€/mois** fixe (baisse démocratique par vote AG)

**Add-ons PropTech** (optionnels, prix coûtant) :
- **AI Assistant Pack** : +2€/mois (chatbot, OCR, prévisions ML)
- **IoT Sensors** : Hardware 15-45€/capteur + 1€/mois/capteur service
- **Blockchain Voting** : +1€/mois (50 votes/mois inclus ≈ 2 AG/an)
- **Energy Buying Groups** : **0€** (mission ASBL, financé par partenariats)

**Mutualisation** : 1 GPU IA sert 5,000 copros = 0.01€/copro/mois coût réel. L'add-on +2€ finance la R&D modèles et rémunération dev.

---

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
│   ├── FONDS_SOLIDARITE.rst
│   └── ...
│
├── docker-compose.yml        # Environnement de développement
├── Makefile                  # Commandes utilitaires
└── README.md
```

---

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

---

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

---

## 🧪 Tests

KoproGo dispose d'une suite de tests complète suivant la pyramide de tests :

### Tests Unitaires (100% coverage Domain)

```bash
# Tous les tests unitaires
cargo test --lib

# Tests avec coverage
make coverage
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
→ Voir [docs/PERFORMANCE_REPORT.rst](docs/PERFORMANCE_REPORT.rst) pour détails complets

---

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
| **CO₂ Impact** | **0.12g/req** | **7-25x mieux que solutions SaaS classiques** |
| **RAM** | 128MB max | Sans swap |
| **CPU** | 8% moyen | Pic à 25% |

**📈 Rapport détaillé** : [docs/PERFORMANCE_REPORT.rst](docs/PERFORMANCE_REPORT.rst)

---

## 🔒 Sécurité & Conformité

### Production Security (Phase 1 - Implémenté)

- ✅ **LUKS Encryption at Rest** : Full-disk encryption PostgreSQL + uploads (AES-XTS-512)
- ✅ **Encrypted Backups** : Daily GPG-encrypted backups + S3 off-site (7d local, configurable S3 lifecycle)
- ✅ **Monitoring Stack** : Prometheus + Grafana + Loki + Alertmanager (30d metrics, 7d logs)
- ✅ **Intrusion Detection** : Suricata IDS avec règles SQL injection, XSS, path traversal
- ✅ **WAF Protection** : CrowdSec community threat intelligence
- ✅ **fail2ban** : Jails SSH, Traefik, API abuse, PostgreSQL brute-force
- ✅ **SSH Hardening** : Key-only, modern ciphers, reduced attack surface
- ✅ **Kernel Hardening** : sysctl security (SYN cookies, IP spoofing protection, ASLR)
- ✅ **Security Auditing** : Lynis (weekly), rkhunter (daily), AIDE file integrity

**Documentation** : [infrastructure/SECURITY.md](infrastructure/SECURITY.md)

### GDPR (Règlement Général sur la Protection des Données)

- ✅ Chiffrement des données sensibles (AES-256)
- ✅ Droit d'accès, rectification, effacement (Articles 15, 16, 17)
- ✅ Portabilité des données (Article 20)
- ✅ Audit logging complet
- ✅ Consentement explicite
- 🔄 Articles 18, 21 en cours (Phase 1)

---

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

---

## 📚 Documentation Complète

Documentation complète disponible dans le dossier `docs/` :

### Stratégie & Vision
- **[Vision](docs/VISION.rst)** ⭐ - Vision technologie au service du bien commun + Modèle démocratique ASBL
- **[Mission](docs/MISSION.rst)** ⭐ - 7 piliers mission ASBL incluant IA éthique, Blockchain, IoT/Énergie
- **[Modèle Économique](docs/ECONOMIC_MODEL.rst)** ⭐⭐ - Prix fixe 5€/mois, transparence comptable, démocratie AG (RECOMMANDÉ)
- **[Fonds de Solidarité](docs/FONDS_SOLIDARITE.rst)** 🆕 - Aide financière membres en difficulté (litiges, prêts 0%, travaux)
- **[Gouvernance](docs/GOVERNANCE.rst)** - Structure ASBL évolutive par jalons, processus décision, Comité d'Éthique IA
- **[Roadmap par Capacités](docs/ROADMAP_PAR_CAPACITES.rst)** ⭐ - Progression par jalons de capacités (pas de dates fixes)

### Performance & Tests de Charge
- **[Performance Report](docs/PERFORMANCE_REPORT.rst)** ⭐ - Tests charge production : 99.74% success, 287 req/s, 0.12g CO₂/req
- **[Infrastructure Costs](docs/INFRASTRUCTURE_COST_SIMULATIONS_2025.rst)** - Simulations coûts infrastructure 2025-2030

### Guides Techniques
- **[CLAUDE.md](CLAUDE.md)** - Instructions développeurs (Architecture hexagonale, TDD, Commandes)
- **[Multi-owner Support](docs/MULTI_OWNER_SUPPORT.md)** - Fonctionnement quotes-parts et API multi-copropriétaires
- **[Multi-role Support](docs/MULTI_ROLE_SUPPORT.md)** - Gestion utilisateurs multi-rôles
- **[Deployment Guide](docs/deployment/index.rst)** - Déploiement (Terraform, Ansible, GitOps)
- **[Security](infrastructure/SECURITY.md)** - Sécurité production (LUKS, IDS, WAF, backups)

### Monitoring & Opérations
- **[Monitoring Guide](monitoring/README.md)** - Scripts monitoring VPS (RAM, CPU, PostgreSQL, capacité)
- **[Capacity Calculator](monitoring/scripts/capacity_calculator.sh)** - Estimation nombre copropriétés supportées

---

## 📝 Contribuer

Contributions bienvenues ! Nous suivons le [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md) pour maintenir une communauté bienveillante.

Le guide [CONTRIBUTING.md](CONTRIBUTING.md) détaille les conventions de branches, les hooks Git et la checklist qualité.

### Modèle Participatif

**Vos contributions ont de la valeur** :
- Contributeur code/docs/traductions : **-50%** tarif cloud
- Mainteneur actif : **100% gratuit** à vie
- Plus de contributeurs = Meilleur produit = Bénéfice pour tous

### Gouvernance ASBL

KoproGo est organisé en **ASBL (Association Sans But Lucratif)** belge garantissant transparence et démocratie. Consultez [docs/GOVERNANCE.rst](docs/GOVERNANCE.rst) pour comprendre :
- Structure de l'ASBL (constitution déclenchée au Jalon 3 : 500-1,000 copros)
- Processus de décision démocratique (CA + AG + communauté, 1 membre = 1 vote)
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

- 📚 **Documentation** : [gilmry.github.io/koprogo](https://gilmry.github.io/koprogo)
- 🎥 **Tutoriels vidéo** : [YouTube @koprogo](https://www.youtube.com/@koprogo)
- 💬 **Discussions** : [GitHub Discussions](https://github.com/gilmry/koprogo/discussions)
- 🐛 **Bugs** : [Créer une issue](https://github.com/gilmry/koprogo/issues/new/choose)
- 🔒 **Sécurité** : Voir [SECURITY.md](SECURITY.md)

---

## 📄 Licence

Ce projet est sous licence AGPL 3.0 (GNU Affero General Public License v3.0). Voir [LICENSE](LICENSE) pour plus de détails.

Cette licence copyleft forte garantit que toute modification du code source, y compris les versions utilisées pour fournir des services réseau, doit être rendue disponible sous la même licence.

---

## 👥 Auteur

- **Gilles Maury** - Fondateur KoproGo ASBL - *contact@koprogo.com*
  - 25 ans d'expérience en informatique
  - Admirateur de la cause libre et des modèles économiques démocratiques, sociaux et solidaires résilients
  - 🔍 **Recrute** : Co-fondateur(trice) Product avec expertise syndic/copropriété

---

## 🙏 Remerciements

- Architecture hexagonale inspirée par Alistair Cockburn
- DDD patterns par Eric Evans
- Actix-web team
- Astro team
- Rust community
- Contributeurs open source du projet

---

**KoproGo** - Complète votre immeuble : Gestion • Communauté • Valeurs 🌱

**L'engouement est notre moteur** : Gestion performante + Modules communautaires + Valeurs partagées = Croissance organique et durable 🔄

**Rejoignez le mouvement** : Prenez ce qui vous manque, gardez ce qui marche. C'est flexible, c'est ouvert, c'est ensemble. 🤝
