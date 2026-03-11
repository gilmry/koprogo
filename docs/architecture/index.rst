==========================
Architecture Technique
==========================

Cette section présente l'architecture technique de KoproGo : choix technologiques, patterns, et organisation du code.

.. note::
   **Architecture Hexagonale (Ports & Adapters)** : KoproGo suit les principes du Domain-Driven Design avec une séparation stricte des couches.

Contenu de cette Section
=========================

.. toctree::
   :maxdepth: 2

   vue-ensemble
   choix-technologiques

Vue d'Ensemble
==============

**Stack Technique**

* **Backend** : Rust 1.83 + Actix-web 4.9 + PostgreSQL 15
* **Frontend** : Astro 4.x + Svelte 4.x (PWA offline-first)
* **Infrastructure** : Terraform + Ansible + GitOps
* **Hébergement** : OVH France (Gravelines, bas carbone)

**Architecture Hexagonale**

.. code-block:: text

   Domain (Core)
     ↑ defines interfaces
   Application (Use Cases + Ports)
     ↑ implements ports
   Infrastructure (Adapters: Web, Database)

**Performance**

* **287 req/s** soutenus (charge réelle)
* **752ms** latence P99 (1 vCPU)
* **0,12g CO₂/requête** (96% réduction vs concurrence)
* **128MB RAM** par instance

Documents
=========

:doc:`vue-ensemble`
   Vue d'ensemble de l'architecture hexagonale, flux de données, et séparation des couches.

:doc:`choix-technologiques`
   Explication détaillée des choix technologiques : pourquoi Rust, PostgreSQL, Astro, Svelte, et architecture hexagonale.

:doc:`../backend/index`
   Documentation complète du backend (domaine, application, infrastructure, tests).

:doc:`../frontend/index`
   Documentation complète du frontend (Astro, Svelte, components, i18n).

:doc:`../infrastructure/index`
   Documentation infrastructure (Terraform, Ansible, Docker, Kubernetes).

----

*Section Architecture Technique - Documentation KoproGo ASBL*
