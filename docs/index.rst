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
   :caption: 🎯 Vision et Mission

   VISION
   MISSION

.. toctree::
   :maxdepth: 2
   :caption: 🗺️ Roadmap

   ROADMAP

.. toctree::
   :maxdepth: 2
   :caption: 💼 Modèle Économique

   ECONOMIC_MODEL
   BUSINESS_PLAN_BOOTSTRAP
   STAKEHOLDER_GUIDE

.. toctree::
   :maxdepth: 2
   :caption: 🦀 Backend Rust

   backend/index
   backend/src/domain/index
   backend/src/application/index
   backend/src/infrastructure/index
   backend/tests/index
   backend/benches/index

.. toctree::
   :maxdepth: 2
   :caption: 🎨 Frontend Astro + Svelte

   frontend/index
   frontend/lib/index
   frontend/components/index
   frontend/pages/index
   frontend/layouts/index
   frontend/stores/index
   frontend/locales/index

.. toctree::
   :maxdepth: 2
   :caption: 🏗️ Infrastructure (Terraform + Ansible)

   infrastructure/index
   infrastructure/terraform/index
   infrastructure/ansible/index

.. toctree::
   :maxdepth: 2
   :caption: 🚀 Déploiement et GitOps

   deployment/index
   deployment/ovh-setup
   deployment/terraform-ansible
   deployment/gitops
   deployment/troubleshooting

.. toctree::
   :maxdepth: 2
   :caption: 🛠️ Guides Développeurs

   MAKEFILE_GUIDE
   E2E_TESTING_GUIDE
   PERFORMANCE_TESTING
   PERFORMANCE_REPORT
   PROJECT_STRUCTURE

=====================================

*Documentation maintenue par la communauté KoproGo ASBL*

*Dernière mise à jour : Octobre 2025*
