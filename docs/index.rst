===================================
Documentation KoproGo ASBL
===================================

**KoproGo** : Plateforme opensource de gestion de copropriété développée par une ASBL belge, utilisant des technologies de pointe pour résoudre un problème sociétal avec un impact écologique minimal.

Introduction
============

KoproGo est un projet **holistique** qui combine :

✅ **Résolution d'un problème sociétal** (gestion copropriétés en Belgique et Europe)
✅ **Technologies de pointe** (Rust, GitOps, IA, Architecture Hexagonale)
✅ **Écologie** (0.12g CO2/requête, 96% réduction vs solutions actuelles)
✅ **Opensource et communautaire** (AGPL-3.0, ASBL, partage des recettes IA)
✅ **Sécurité et conformité** (RGPD, souveraineté des données, GitOps)
✅ **Pédagogie** (documentation exhaustive, onboarding facilité)
✅ **Progression mesurable** (jalons basés sur capacités, pas sur dates)

**Stack Technique** :

- **Backend**: Rust 1.83 + Actix-web 4.9 + PostgreSQL 15
- **Frontend**: Astro 4.x + Svelte 4.x (PWA offline-first)
- **Infrastructure**: Terraform + Ansible + GitOps (OVH Cloud)
- **Architecture**: Hexagonale (DDD) avec tests exhaustifs (Pyramid Strategy)

Ressources
==========

📺 **Tutoriels Vidéo** : `Chaîne YouTube @koprogo <https://www.youtube.com/@koprogo>`_

Retrouvez des tutoriels vidéo pour démarrer avec KoproGo, comprendre l'architecture hexagonale, et découvrir les fonctionnalités avancées de la plateforme.

=====================================
Documentation
=====================================

.. toctree::
   :maxdepth: 2
   :caption: 📘 Vision & Stratégie (Hiérarchie Stratégique)

   VISION
   MISSION
   GOVERNANCE
   ECONOMIC_MODEL
   FONDS_SOLIDARITE

**Hiérarchie de lecture recommandée** :

1. **VISION** : Vision macro sociétale et problème à résoudre (pourquoi KoproGo existe)
2. **MISSION** : Solutions concrètes et valeurs fondamentales (comment nous résolvons le problème)
3. **GOVERNANCE** : Structure organisationnelle évolutive Solo → Fondateurs → ASBL → Coopérative
4. **ECONOMIC_MODEL** : Modèle économique ASBL et viabilité financière à long terme
5. **FONDS_SOLIDARITE** : Mécanisme d'aide financière aux membres en difficulté (solidarité concrète)

**Métriques de Succès par Paliers** (progression mesurable):

.. list-table:: Progression par Capacités
   :header-rows: 1
   :widths: 20 20 20 20 20

   * - Palier
     - Copropriétés
     - Impact Économique
     - CO₂ évité/an
     - Participants
   * - **Validation**
     - 100
     - 80k€
     - -2 tonnes
     - 10
   * - **Viabilité**
     - 500
     - 400k€
     - -15 tonnes
     - 50
   * - **Impact**
     - 1.000
     - 800k€
     - -107 tonnes
     - 100
   * - **Leadership**
     - 2.000
     - 1,6M€
     - -214 tonnes
     - 200
   * - **Référence**
     - 5.000
     - 4M€
     - **-840 tonnes**
     - 500

**Performance Technique Validée** :

* Latence P99: 752ms (charge soutenue, 1 vCPU) ✅
* Throughput: 287 req/s soutenu ✅
* Consommation: 0.12g CO₂/req (96% réduction vs marché) ✅
* RAM: 128MB utilisée sur 2GB (5% seulement) ✅
* Viabilité: Marge 98% maintenue à tous les paliers ✅

**Chaque palier débloque le suivant. Pas de dates fixes, mais des conditions mesurables.**

.. toctree::
   :maxdepth: 2
   :caption: 💰 Finances & Performance (Données Réelles 2025)

   INVESTOR_EXECUTIVE_SUMMARY_2025
   INFRASTRUCTURE_COST_SIMULATIONS_2025
   PERFORMANCE_REPORT
   PERFORMANCE_TESTING

.. toctree::
   :maxdepth: 2
   :caption: 🗺️ Roadmap

   ROADMAP_PAR_CAPACITES
   roadmap/agile-journey

.. toctree::
   :maxdepth: 2
   :caption: ⚡ Gouvernance Agile

   governance/togaf/adm
   governance/nexus/framework
   governance/scrum/ceremonies
   governance/rfc/template
   governance/adr/0001-mcp-integration

.. toctree::
   :maxdepth: 2
   :caption: 📊 GitHub Project Management

   github-export/index

.. toctree::
   :maxdepth: 2
   :caption: 💻 Documentation Technique

   backend/index
   frontend/index
   infrastructure/index
   deployment/index

.. toctree::
   :maxdepth: 2
   :caption: 🛠️ Guides Développeurs

   MAKEFILE_GUIDE
   E2E_TESTING_GUIDE
   e2e-videos
   PROJECT_STRUCTURE
   GIT_HOOKS
   ROLE_PERMISSIONS_MATRIX
   MULTI_OWNER_SUPPORT
   MULTI_ROLE_SUPPORT
   OWNER_MODEL_REFACTORING
   RELEASE_PROCESS
   PERFORMANCE_TUNING
   DATABASE_ADMIN
   INTEGRATION_GUIDES

.. toctree::
   :maxdepth: 2
   :caption: 👥 Guides Utilisateurs

   user-guides/syndic-guide
   user-guides/owner-guide
   user-guides/accountant-guide
   user-guides/board-member-guide

.. toctree::
   :maxdepth: 2
   :caption: 🔧 API Documentation

   api/README

.. toctree::
   :maxdepth: 2
   :caption: 🏗️ Architecture Decision Records (ADR)

   adr/0001-rust-actix-web-backend
   adr/0002-hexagonal-architecture
   adr/0003-postgresql-database
   adr/0004-astro-svelte-frontend
   adr/0005-jwt-authentication
   adr/0006-agpl-license
   adr/0044-document-storage-strategy

.. toctree::
   :maxdepth: 2
   :caption: 🚨 Operations & SRE

   operations/disaster-recovery
   operations/monitoring-runbook
   operations/backup-recovery
   operations/incident-response

.. toctree::
   :maxdepth: 2
   :caption: 🎨 Frontend & Internationalization

   FRONTEND_COMPONENTS
   EMAIL_TEMPLATES
   I18N_GUIDE

.. toctree::
   :maxdepth: 2
   :caption: 🔒 Sécurité & Conformité

   BELGIAN_ACCOUNTING_PCMN
   INVOICE_WORKFLOW
   PAYMENT_RECOVERY_WORKFLOW
   GDPR_COMPLIANCE_CHECKLIST
   GDPR_IMPLEMENTATION_STATUS
   GDPR_ADDITIONAL_RIGHTS
   BOARD_OF_DIRECTORS_GUIDE

=====================================

*Documentation maintenue par la communauté KoproGo ASBL*

*Modèle de progression: Capacités et métriques, pas dates fixes*
