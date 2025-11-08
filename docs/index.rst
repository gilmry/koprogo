===================================
Documentation KoproGo ASBL
===================================

**KoproGo** : Plateforme opensource de gestion de copropriété développée par une ASBL belge, utilisant des technologies de pointe pour résoudre un problème sociétal avec un impact écologique minimal.

Introduction
============

KoproGo est un projet **holistique** qui combine :

✅ **Résolution d'un problème sociétal** (gestion copropriétés en Belgique et Europe)
✅ **Technologies de pointe** (Rust, GitOps, IA, Architecture Hexagonale)
✅ **Écologie** (< 0.5g CO2/requête, 96% réduction vs solutions actuelles)
✅ **Opensource et communautaire** (AGPL-3.0, ASBL, partage des recettes IA)
✅ **Sécurité et conformité** (RGPD, souveraineté des données, GitOps)
✅ **Pédagogie** (documentation exhaustive, onboarding facilité)

**Stack Technique** :

- **Backend**: Rust 1.83 + Actix-web 4.9 + PostgreSQL 15
- **Frontend**: Astro 4.x + Svelte 4.x (PWA offline-first)
- **Infrastructure**: Terraform + Ansible + GitOps (OVH Cloud)
- **Architecture**: Hexagonale (DDD) avec tests exhaustifs (Pyramid Strategy)

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

**Hiérarchie de lecture recommandée** :

1. **VISION** : Vision macro sociétale et problème à résoudre (pourquoi KoproGo existe)
2. **MISSION** : Solutions concrètes et valeurs fondamentales (comment nous résolvons le problème)
3. **GOVERNANCE** : Structure organisationnelle évolutive Solo → Fondateurs → ASBL → Coopérative
4. **ECONOMIC_MODEL** : Modèle économique ASBL et viabilité financière à long terme

**KPIs Stratégiques 2030** :

* **Adoption** : 5,000 copropriétés (100,000 personnes)
* **Impact Économique** : 9,35M€/an économisés (8M€ logiciels + 750k€ SEL + 600k€ consommation évitée)
* **Impact Écologique** : -534 tonnes CO₂/an (50t infrastructure + 484t features communautaires)
* **Performance Technique** : P99 < 5ms, Uptime > 99.9%, < 0.5g CO₂/req
* **Communauté** : 100 contributeurs réguliers

.. toctree::
   :maxdepth: 2
   :caption: 🗺️ Roadmaps

   ROADMAP_INTEGREE_2025_2030
   ROADMAP

.. toctree::
   :maxdepth: 2
   :caption: 📊 GitHub Project Management

   github-export/index

.. toctree::
   :maxdepth: 2
   :caption: 🦀 Backend Rust

   backend/index

.. toctree::
   :maxdepth: 2
   :caption: 🎨 Frontend Astro + Svelte

   frontend/index

.. toctree::
   :maxdepth: 2
   :caption: 🏗️ Infrastructure

   infrastructure/index

.. toctree::
   :maxdepth: 2
   :caption: 🚀 Déploiement et GitOps

   deployment/index

.. toctree::
   :maxdepth: 2
   :caption: 🔐 Sécurité et Permissions

   ROLE_PERMISSIONS_MATRIX

.. toctree::
   :maxdepth: 2
   :caption: 🛠️ Guides Développeurs

   MAKEFILE_GUIDE
   E2E_TESTING_GUIDE
   e2e-videos
   PERFORMANCE_TESTING
   PERFORMANCE_REPORT
   PROJECT_STRUCTURE
   GIT_HOOKS
   MULTI_OWNER_SUPPORT
   MULTI_ROLE_SUPPORT
   OWNER_MODEL_REFACTORING

=====================================

*Documentation maintenue par la communauté KoproGo ASBL*

*Dernière mise à jour : Novembre 2025*
