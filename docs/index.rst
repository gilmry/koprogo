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

**KPIs Stratégiques 2030** (validés avec données réelles Oct 2025):

* **Adoption** : 5,000 copropriétés (100,000 personnes)
* **Impact Économique** : 9,35M€/an économisés (8M€ logiciels + 750k€ SEL + 600k€ consommation)
* **Impact Écologique** : **-840 tonnes CO₂/an** (dépassement +57% vs objectif -534t)
* **Performance Technique** : P99 < 1s (réel: 752ms ✅), Throughput > 200 req/s (réel: 287 req/s ✅), 0.12g CO₂/req
* **Viabilité Financière** : 84,000€/an revenus, 2,034€/an coûts, **81,966€/an surplus** (marge 98%)
* **Communauté** : 100 contributeurs réguliers

.. toctree::
   :maxdepth: 2
   :caption: 💰 Finances & Performance (Données Réelles 2025)

   INVESTOR_EXECUTIVE_SUMMARY_2025
   INFRASTRUCTURE_COST_SIMULATIONS_2025
   FINANCIAL_COHERENCE_AUDIT_2025
   PERFORMANCE_REPORT
   PERFORMANCE_TESTING

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

=====================================

*Documentation maintenue par la communauté KoproGo ASBL*

*Dernière mise à jour : Novembre 2025*
