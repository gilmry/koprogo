=======================================
Documentation Miroir Complète - KoproGo
=======================================

Cette documentation **miroir** du projet KoproGo couvre l'intégralité du code source :

- ✅ **Backend** (Rust + Actix-web) - 321 fichiers
- ✅ **Frontend** (Astro + Svelte + TypeScript)
- ✅ **Infrastructure** (Ansible, Docker, Scripts)

**Principe** : Chaque fichier source a un fichier .rst correspondant avec **explication en français** de son rôle.

Architecture
============

Le projet suit une **architecture hexagonale** (Ports & Adapters) avec **Domain-Driven Design (DDD)**.

.. toctree::
   :maxdepth: 2
   :caption: Documentation par Composant

   backend/index
   frontend/index
   infrastructure/index

Couches Backend
===============

1. **Domain** (Métier)
   - Entités de domaine avec validation métier
   - Services de domaine (logique complexe)
   - Aucune dépendance externe

2. **Application** (Use Cases)
   - Use Cases (orchestration)
   - Ports (interfaces/traits)
   - DTOs (contrats API)

3. **Infrastructure** (Adaptateurs)
   - Repositories PostgreSQL
   - Handlers HTTP (Actix-web)
   - Clients API externes (Stripe, Linky)

Frontend
========

- **Astro** : SSG (Static Site Generation)
- **Svelte** : Composants interactifs (Islands)
- **TypeScript** : Type-safety

Infrastructure
==============

- **Ansible** : Déploiement VPS automatisé
- **Docker** : Conteneurisation services
- **GitHub Actions** : CI/CD pipelines

Liens Rapides
=============

- :doc:`/CLAUDE` - Guide développeur
- :doc:`/ARCHITECTURE` - Architecture détaillée
- :doc:`/NOUVELLES_FONCTIONNALITES_2025` - Features 2025
- :doc:`/IOT_INTEGRATION` - Intégration IoT

