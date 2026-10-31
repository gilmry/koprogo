===============================================
Nexus Framework pour KoproGo
===============================================

:Auteur: KoproGo ASBL
:Date: 2025-01-15
:Version: 1.0
:Statut: Actif

.. contents:: Table des matières
   :depth: 3
   :local:

Introduction
============

**Nexus** est un framework de scaling Scrum développé par Scrum.org pour coordonner 3 à 9 équipes Scrum travaillant sur le même Product Backlog.

KoproGo adopte Nexus pour coordonner ses 4 équipes techniques tout en maintenant l'agilité et la qualité du code.

Pourquoi Nexus pour KoproGo ?
==============================

Avantages vs alternatives
--------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 25 25 25

   * - Framework
     - Complexité
     - Adapté ASBL ?
     - Décision
   * - **Nexus**
     - Faible (sur-couche Scrum)
     - ✅ Oui
     - **CHOISI**
   * - **SAFe**
     - Élevée (lourd, corporate)
     - ❌ Non
     - Rejeté
   * - **LeSS**
     - Moyenne (radical)
     - ⚠️ Moyen
     - Rejeté
   * - **Scrum of Scrums**
     - Faible (informel)
     - ⚠️ Trop léger
     - Rejeté

**Justification Nexus** :

1. **Lightweight** : Ajoute seulement 3 événements (Nexus Planning, Daily, Sprint Review)
2. **Communautaire** : Gratuit, open-source (Scrum.org)
3. **Scalable** : 3-9 équipes (KoproGo prévoit 4-6 équipes max)
4. **Compatible Scrum** : Pas de réinvention, sur-couche seulement

Structure Équipes KoproGo
==========================

Organisation 4 équipes
----------------------

.. code-block:: text

   ┌─────────────────────────────────────────────────┐
   │  Nexus Integration Team (NIT)                   │
   │  - Product Owner (PO ASBL)                      │
   │  - Scrum Master Nexus                           │
   │  - Tech Leads (1 par équipe)                    │
   └────────────────┬────────────────────────────────┘
                    │
        ┌───────────┼───────────┬───────────┐
        │           │           │           │
   ┌────▼────┐ ┌───▼────┐ ┌────▼────┐ ┌───▼────┐
   │ Backend │ │Frontend│ │  Infra  │ │IA/Grid │
   │  Team   │ │  Team  │ │  Team   │ │  Team  │
   └─────────┘ └────────┘ └─────────┘ └────────┘
     3-5 dev    3-5 dev     2-4 ops    2-3 ML

Équipe 1 : Backend (Rust)
--------------------------

**Mission** : API REST, domain logic, sécurité, performance

**Stack** :

- Rust 1.83 + Actix-web 4.9
- PostgreSQL 15 (SQLx)
- Redis 7 (future)

**Responsabilités** :

- Architecture hexagonale (domain, application, infrastructure)
- Use cases métier (buildings, expenses, meetings, accounts)
- Repositories PostgreSQL
- Tests unitaires + intégration (> 90% coverage)
- Benchmarks performance (Criterion)

**Compétences requises** :

- Rust (ownership, traits, async)
- PostgreSQL (indexes, EXPLAIN ANALYZE)
- Architecture DDD
- Tests (unit, integration, BDD Cucumber)

**Taille** : 3-5 développeurs

Équipe 2 : Frontend (Svelte)
-----------------------------

**Mission** : UI/UX, PWA, accessibilité, mobile

**Stack** :

- Astro 4.x (SSG, Islands)
- Svelte 4.x (components)
- TailwindCSS 3.x
- Playwright (E2E tests)

**Responsabilités** :

- Composants Svelte réutilisables
- Pages Astro (SSG)
- PWA offline-first (ServiceWorker)
- Tests E2E Playwright
- Accessibilité WCAG 2.1 AA

**Compétences requises** :

- TypeScript
- Svelte/Astro
- TailwindCSS
- Tests E2E (Playwright)
- UX/UI design

**Taille** : 3-5 développeurs

Équipe 3 : Infrastructure (GitOps)
-----------------------------------

**Mission** : Déploiement, monitoring, sécurité, SRE

**Stack** :

- Terraform (IaC)
- Ansible (config management)
- Traefik 3.0 (reverse proxy)
- Prometheus + Grafana (monitoring)
- Suricata + CrowdSec (sécurité)

**Responsabilités** :

- Infrastructure as Code (Terraform)
- CI/CD GitHub Actions
- Monitoring stack (Prometheus, Loki, Grafana)
- Sécurité (IDS, WAF, fail2ban)
- Backups chiffrés (GPG, S3)
- Runbooks SRE

**Compétences requises** :

- Linux (sysctl, iptables, systemd)
- Docker/K8s
- Terraform/Ansible
- Monitoring (Prometheus, Grafana)
- Sécurité (IDS, WAF)

**Taille** : 2-4 ops

Équipe 4 : IA & Grid Computing (Jalon 6)
-----------------------------------------

**Mission** : MCP, edge computing, IA locale, revenus distribués

**Stack** :

- MCP v1 (Model Context Protocol)
- llama.cpp (LLM local)
- Rust (koprogo-mcp server)
- Raspberry Pi (edge nodes)

**Responsabilités** :

- Serveur MCP Rust (koprogo-mcp)
- Client edge (koprogo-node sur Raspberry Pi)
- Distribution tasks (grid computing)
- Monétisation compute (revenus partagés)
- Tests edge (ARM, x86, macOS)

**Compétences requises** :

- Rust (MCP SDK)
- Machine Learning (LLM, embedding)
- Edge computing (Raspberry Pi, WASM)
- Réseau P2P (libp2p future)

**Taille** : 2-3 développeurs ML

**Note** : Équipe 4 démarre au Jalon 6 (2026+)

Nexus Integration Team (NIT)
=============================

Rôle et responsabilités
-----------------------

Le **NIT** est responsable de :

1. **Intégration technique** : Résoudre dépendances cross-équipes
2. **Product Backlog** : Maintenir backlog unifié (GitHub Projects)
3. **Definition of Done** : Garantir DoD respectée par toutes équipes
4. **Nexus Sprint Goal** : Définir objectif commun sprint

Composition NIT
---------------

.. list-table::
   :header-rows: 1
   :widths: 30 50 20

   * - Rôle
     - Responsabilités NIT
     - Temps dédié
   * - **Product Owner (PO)**
     - Vision produit, priorisation backlog
     - 100%
   * - **Scrum Master Nexus**
     - Facilitation Nexus events, résolution blocages
     - 100%
   * - **Tech Lead Backend**
     - Intégration API, architecture hexagonale
     - 20%
   * - **Tech Lead Frontend**
     - Intégration UI, contrats API
     - 20%
   * - **Tech Lead Infra**
     - CI/CD, déploiements, monitoring
     - 20%
   * - **Tech Lead IA** (Jalon 6)
     - Intégration MCP, edge computing
     - 20%

**Total NIT** : 2 full-time (PO + SM) + 4 tech leads part-time (20%)

Événements Nexus
=================

Nexus Sprint Planning
---------------------

**Objectif** : Planifier sprint cross-équipes, identifier dépendances

**Durée** : 4h (sprint 2 semaines)

**Participants** : NIT + représentants équipes (SM + 1-2 devs)

**Déroulement** :

1. **Part 1 (1h)** : NIT présente Nexus Sprint Goal + top backlog items
2. **Part 2 (2h)** : Équipes planifient en parallèle (4 salles Zoom breakout)
3. **Part 3 (1h)** : Plénière - Partage plans, identification dépendances

**Outputs** :

- Nexus Sprint Goal (objectif commun)
- 4 Sprint Backlogs (1 par équipe)
- Nexus Sprint Backlog (items cross-équipes + intégration)
- Tableau dépendances (qui bloque qui)

**Exemple Nexus Sprint Goal (Sprint 12)** :

   *"Implémenter workflow factures avec approbation conseil syndical + frontend mobile-ready + monitoring Prometheus production"*

   - **Backend** : Expense workflow (Draft → PendingApproval → Approved)
   - **Frontend** : UI approbation + responsive mobile
   - **Infra** : Prometheus metrics backend + alertes Slack
   - **Dépendance** : Frontend attend API backend (POST /expenses/:id/approve)

Nexus Daily Scrum
-----------------

**Objectif** : Synchroniser équipes, détecter blocages d'intégration

**Durée** : 15 min

**Participants** : NIT (obligatoire) + représentants équipes (optionnel)

**Format** :

Chaque tech lead répond :

1. **Intégration livrée hier** : Qu'est-ce qui est intégré/déployable ?
2. **Intégration prévue aujourd'hui** : Qu'est-ce qui sera prêt ?
3. **Blocages cross-équipes** : Qui bloque qui ?

**Exemple** :

.. code-block:: text

   Tech Lead Backend:
   - Hier: API POST /expenses/:id/approve merged + déployé staging
   - Aujourd'hui: Tests E2E workflow complet
   - Blocage: Aucun

   Tech Lead Frontend:
   - Hier: UI approbation 80% (mockup)
   - Aujourd'hui: Intégration API POST /expenses/:id/approve
   - Blocage: ⚠️ API retourne 500 si expense déjà approved (unexpected)

   → Action NIT: Backend fix edge case (idempotence)

**Note** : Équipes font AUSSI leur Daily Scrum local (voir :doc:`/governance/scrum/ceremonies`)

Nexus Sprint Review
-------------------

**Objectif** : Démontrer Increment intégré aux stakeholders

**Durée** : 2h

**Participants** : NIT + équipes + stakeholders (copropriétés, syndics, ASBL CA)

**Déroulement** :

1. **Démo Increment intégré (1h)** : Démo cross-équipes (pas 4 démos séparées !)
2. **Feedback stakeholders (30min)** : Questions, suggestions
3. **Ajustement backlog (30min)** : PO ajuste priorités selon feedback

**Exemple démo Sprint 12** :

   *"Workflow factures end-to-end : création facture (backend) → UI liste factures (frontend) → soumission approbation → email notification conseil syndical → approbation/rejet → monitoring Prometheus (alerte si > 10 factures pending)"*

   - Démo sur environnement **staging** (pas local !)
   - Données réalistes (seed script)
   - Monitoring Grafana projeté (dashboard temps réel)

Nexus Sprint Retrospective
---------------------------

**Objectif** : Améliorer collaboration cross-équipes

**Durée** : 1h30

**Participants** : NIT + équipes

**Déroulement** :

1. **Retros locales (45min)** : Chaque équipe fait sa retro (4 salles parallèles)
2. **Retro Nexus (45min)** : Plénière NIT + représentants équipes

**Thèmes Nexus Retro** :

- Qualité intégration (bugs cross-équipes)
- Communication (Slack, GitHub, wiki)
- Dépendances (bloquées trop longtemps ?)
- Tooling (CI/CD, testcontainers)
- DoD (respectée ?)

**Exemple amélioration** :

   *"Backend et Frontend perdent temps sur contrats API (types divergents). Action : Adopter OpenAPI 3.0 + génération TypeScript auto (openapi-generator)"*

**Note** : Équipes font AUSSI leur Retro locale (voir :doc:`/governance/scrum/ceremonies`)

Product Backlog Unifié
======================

GitHub Projects
---------------

**Outil** : GitHub Projects (Kanban board unifié)

**URL** : https://github.com/users/gilmry/projects

**Structure** :

.. code-block:: text

   Colonnes:
   - 📥 Backlog (triées par priorité PO)
   - 🎯 Sprint N (items sprint en cours)
   - 🔨 In Progress (WIP limit = 2 par dev)
   - 👀 Review (attente code review)
   - ✅ Done (merged + déployé staging)

**Labels** :

- **Équipe** : ``team:backend``, ``team:frontend``, ``team:infra``, ``team:ia``
- **Priorité** : ``P0`` (blocker), ``P1`` (high), ``P2`` (medium), ``P3`` (low)
- **Type** : ``feature``, ``bug``, ``refactor``, ``docs``, ``test``
- **Jalon** : ``milestone:1``, ``milestone:2``, ..., ``milestone:6``
- **Cross-équipe** : ``nexus`` (item nécessite coordination)

Refinement Backlog
------------------

**Fréquence** : Mid-sprint (1x par sprint)

**Durée** : 1h

**Participants** : PO + NIT + tech leads

**Objectif** :

- Affiner top 20 items backlog
- Décomposer epics en user stories
- Estimer (Planning Poker, Fibonacci)
- Identifier dépendances cross-équipes

**Critères "Ready"** (item prêt pour sprint) :

1. ✅ User story claire (format : *As X, I want Y, so that Z*)
2. ✅ Critères acceptation définis (Given/When/Then)
3. ✅ Estimée (story points Fibonacci)
4. ✅ Dépendances identifiées (équipes impliquées)
5. ✅ Testable (critères automatisables)

**Exemple item Ready** :

.. code-block:: text

   **User Story**:
   As a syndic, I want to submit an expense for approval,
   so that the conseil syndical can review it before payment.

   **Critères acceptation**:
   - Given: expense in Draft state
   - When: syndic clicks "Submit for approval"
   - Then: expense state = PendingApproval
   - And: email sent to conseil syndical members
   - And: audit log recorded

   **Estimation**: 5 points

   **Équipes**: Backend (3 pts), Frontend (2 pts)

   **Dépendances**: Needs email service (Infra team)

Definition of Done (DoD) Nexus
===============================

DoD Cross-équipes
-----------------

Un increment est **Done** si et seulement si :

**Code** :

1. ✅ Merged dans ``main`` (via PR approuvée)
2. ✅ Tests passent (unit + integration + E2E)
3. ✅ Coverage > 90% (backend), > 80% (frontend)
4. ✅ Linting OK (clippy, prettier)
5. ✅ Code reviewed (2+ reviewers si cross-équipe)

**Documentation** :

6. ✅ RFC approuvé (si changement majeur)
7. ✅ ADR rédigé (si décision technique)
8. ✅ Docs Sphinx mise à jour (API, guides)
9. ✅ CHANGELOG.md mis à jour

**Tests** :

10. ✅ Tests unitaires (domain logic)
11. ✅ Tests intégration (PostgreSQL testcontainers)
12. ✅ Tests E2E (Playwright full workflow)
13. ✅ Tests BDD (Cucumber scenarios)

**Performance** :

14. ✅ Benchmarks passent (P99 < 5ms)
15. ✅ Impact CO₂ mesuré et publié
16. ✅ Load tests OK (> 287 req/s)

**Déploiement** :

17. ✅ Déployé staging (smoke tests OK)
18. ✅ Rollback plan documenté
19. ✅ Monitoring Prometheus (métriques exposées)
20. ✅ Alertes configurées (Grafana)

**Sécurité & Conformité** :

21. ✅ Scan sécurité OK (cargo audit, npm audit)
22. ✅ RGPD compliant (si données sensibles)
23. ✅ Logs anonymisés (PII removed)

DoD par Équipe
--------------

**Backend** (ajoute à DoD Nexus) :

- Contrats API documentés (OpenAPI future)
- Migration DB testée (rollback + rollforward)
- Ports & Adapters respectés (hexagonal architecture)

**Frontend** (ajoute à DoD Nexus) :

- Responsive mobile (< 768px)
- Accessibilité WCAG 2.1 AA (axe-core tests)
- PWA fonctionne offline

**Infra** (ajoute à DoD Nexus) :

- Terraform plan OK (dry-run)
- Ansible playbook idempotent
- Runbook SRE rédigé (incident response)

**IA/Grid** (ajoute à DoD Nexus) :

- Tests edge (Raspberry Pi, ARM64)
- Modèle quantized (< 2GB RAM)
- Revenus compute trackés (blockchain future)

Gestion Dépendances
===================

Tableau Dépendances Sprint
---------------------------

Maintenu dans **Nexus Sprint Backlog** (Google Sheets ou GitHub Projects custom fields)

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 15 15

   * - Item Backlog
     - Équipe Owner
     - Dépend de
     - Statut
     - Blocage ?
   * - POST /expenses/:id/approve
     - Backend
     - —
     - ✅ Done
     - Non
   * - UI approbation factures
     - Frontend
     - Backend API
     - 🔨 In Progress
     - Non (API ready)
   * - Email notification approbation
     - Backend
     - Infra (SendGrid)
     - 👀 Review
     - ⚠️ Oui (SendGrid quota)
   * - Prometheus metrics expenses
     - Infra
     - Backend (metrics endpoint)
     - 📥 Backlog
     - Non

**Résolution blocages** :

- **Détection** : Nexus Daily Scrum
- **Escalade** : SM Nexus (immédiat)
- **Résolution** : NIT + équipes concernées (< 24h)

**Exemple résolution** :

   *Blocage: SendGrid quota dev dépassé (100 emails/jour).*

   *Résolution NIT: Upgrade plan SendGrid (19$/mois), ou mock emails en dev (tests E2E).*

   *Décision PO: Mock emails dev, budget SendGrid production seulement.*

Risques Cross-équipes
----------------------

**Risque 1 : API contract mismatch** (Backend ↔ Frontend)

**Mitigation** :

- OpenAPI 3.0 spec (future)
- Génération TypeScript auto (openapi-generator)
- Tests E2E contract (Playwright)

**Risque 2 : Database migration failure** (Backend ↔ Infra)

**Mitigation** :

- Testcontainers (migration tests)
- Rollback plan obligatoire
- Blue/green deployment (zero downtime)

**Risque 3 : Performance regression** (Backend ↔ Infra)

**Mitigation** :

- Benchmarks CI (Criterion)
- Load tests staging (k6 future)
- Monitoring Prometheus (alertes P99 > 5ms)

Outils Collaboration
====================

Communication
-------------

.. list-table::
   :header-rows: 1
   :widths: 25 35 40

   * - Outil
     - Usage
     - Règles
   * - **GitHub Issues**
     - Backlog items, bugs
     - 1 issue = 1 user story
   * - **GitHub PRs**
     - Code reviews
     - 2+ reviewers si cross-équipe
   * - **GitHub Discussions**
     - RFCs, ADRs, design docs
     - Commentaires asynchrones
   * - **Slack** (future)
     - Chat temps réel
     - Channels: #backend, #frontend, #infra, #nexus
   * - **Zoom** (future)
     - Nexus events
     - Enregistrés + publiés (transparence)

Documentation
-------------

**Sphinx RST** :

- Architecture (ADM, Nexus, Scrum)
- Guides développeurs (setup, tests, benchmarks)
- RFCs (propositions majeures)
- ADRs (décisions techniques)

**GitHub Wiki** (future) :

- Runbooks SRE
- Troubleshooting
- Onboarding contributeurs

**Miro** (future) :

- Architecture diagrams
- Event storming (DDD)

Métriques Nexus
===============

Vélocité Cross-équipes
----------------------

**Vélocité sprint** : Somme story points **Done** (toutes équipes)

**Objectif** : Stabilité vélocité (± 10% sprint N vs N-1)

**Exemple Sprint 10-12** :

.. code-block:: text

   Sprint 10: 52 points (Backend 20, Frontend 18, Infra 14)
   Sprint 11: 48 points (Backend 18, Frontend 16, Infra 14)
   Sprint 12: 55 points (Backend 22, Frontend 20, Infra 13)

   Moyenne: 51,7 points ± 3,5 (stable ✅)

**Alarmes** :

- Vélocité baisse > 20% → Retro focus (burnout ? dépendances ?)
- Vélocité monte > 30% → Sous-estimation ? DoD non respectée ?

Lead Time Intégration
---------------------

**Définition** : Temps entre "PR merged" équipe A et "Intégré/testé" équipe B

**Objectif** : < 24h (1 working day)

**Exemple** :

.. code-block:: text

   Backend merge API POST /expenses/:id/approve: Lundi 10h
   Frontend intègre + teste API: Lundi 16h
   → Lead time: 6h ✅

**Alarmes** :

- Lead time > 48h → Blocage process (NIT investigate)

Bugs Cross-équipes
------------------

**Définition** : Bugs découverts APRÈS intégration (pas détectés par tests unitaires/équipe)

**Objectif** : < 5 bugs/sprint

**Exemple Sprint 12** :

- 2 bugs intégration (API contract mismatch)
- 1 bug performance (N+1 queries PostgreSQL)
- Total: 3 bugs ✅

**Actions si > 5 bugs** :

- Revoir DoD (tests E2E insuffisants ?)
- Améliorer communication (Nexus Daily ?)
- Pair programming cross-équipes

Évolution Nexus
===============

Phase Solo → ASBL → Coopérative
--------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 30 30 20

   * - Phase
     - Équipes
     - Framework
     - Participants
   * - **Solo** (Nov 2025)
     - 1 (full-stack)
     - Scrum local
     - 1 dev
   * - **Fondateurs** (Déc 2025 - Fév 2026)
     - 2 (backend, frontend)
     - Scrum of Scrums
     - 3-5 devs
   * - **ASBL** (Mar - Mai 2026+)
     - 4 (back, front, infra, IA)
     - **Nexus**
     - 10-20 devs
   * - **Coopérative** (2027+)
     - 6-9 (+ mobile, data, sec)
     - Nexus
     - 50-100 devs

**Trigger adoption Nexus** : 3+ équipes (≥ 10 devs total)

Scaling au-delà de 9 équipes
-----------------------------

Si KoproGo dépasse 9 équipes (> 100 devs), considérer :

1. **Nexus+ (multi-Nexus)** : 2+ Nexus en parallèle (ex: Nexus Europe, Nexus Belgique)
2. **LeSS (Large-Scale Scrum)** : Alternative radicale (1 PO, 1 backlog, 8+ équipes)
3. **SAFe (Scaled Agile)** : Si gouvernance corporate nécessaire (peu probable ASBL)

**Décision** : Évaluer en 2026+ (après Jalon 5)

Voir aussi
==========

- :doc:`/governance/scrum/ceremonies` : Scrum local par équipe
- :doc:`/governance/togaf/adm` : Architecture d'entreprise TOGAF
- :doc:`/governance/rfc/template` : Template RFC
- :doc:`/ROADMAP_PAR_CAPACITES` : Roadmap par jalons

---

*Document maintenu par KoproGo ASBL - Nexus Framework adapté pour l'open-source*
