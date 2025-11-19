======================================
Documentation KoproGo
======================================

**Plateforme open-source de gestion de copropriété pour la Belgique**

.. note::
   📺 **Tutoriels Vidéo** : `Chaîne YouTube @koprogo <https://www.youtube.com/@koprogo>`_

   Retrouvez des tutoriels vidéo pour démarrer avec KoproGo, comprendre l'architecture hexagonale, et découvrir les fonctionnalités avancées de la plateforme.

Introduction
============

**En Belgique, les 200 000 copropriétés dépensent collectivement 70 millions d'euros par an en frais de gestion logicielle.** Les solutions propriétaires facturent entre 200 et 500€/mois, avec des marges importantes, tout en générant une empreinte carbone élevée.

**KoproGo propose une alternative radicalement différente :** une plateforme open-source, développée par une ASBL belge, offrant une gestion complète pour **5€/mois** en cloud managé ou **gratuite en self-hosted**.

Cette différence de prix n'est pas un miracle : elle résulte d'une **architecture ultra-optimisée en Rust**, d'une **infrastructure mutualisée** et d'une **gouvernance sans actionnaires** où chaque euro économisé profite à la communauté.

Le Problème
===========

Les copropriétés belges font face à plusieurs défis :

.. attention::
   **Coûts Prohibitifs**

   * **200-500€/mois** par copropriété pour les solutions propriétaires
   * **70M€/an** dépensés collectivement en Belgique
   * Petites copropriétés exclues par les prix élevés
   * Dépendance à des acteurs privés avec marges importantes

.. attention::
   **Impact Environnemental**

   * Solutions SaaS surdimensionnées : **11,5g CO₂/requête** en moyenne
   * Datacenters énergivores et technologies inefficaces
   * Aucune optimisation écologique

.. attention::
   **Manque de Transparence**

   * Calculs de charges opaques
   * Pas de souveraineté des données (GDPR)
   * Litiges fréquents faute de traçabilité

La Solution KoproGo
===================

**KoproGo résout ces problèmes avec une approche innovante :**

.. tip::
   **99% d'Économies**

   * **5€/mois** en cloud managé vs 200-500€/mois des concurrents
   * **Gratuit** en self-hosted (licence AGPL-3.0)
   * **70M€/an** économisables pour les copropriétés belges
   * Prix démocratique : voté par l'Assemblée Générale (modèle ASBL)

.. tip::
   **96% de Réduction CO₂**

   * **0,12g CO₂/requête** grâce à l'architecture Rust
   * Datacenter bas carbone OVH France (60g CO₂/kWh)
   * **840 tonnes CO₂/an évitées** à 5 000 copropriétés
   * Infrastructure mutualisée ultra-économique

.. tip::
   **Open-Source & Souveraineté**

   * Code public sur GitHub (AGPL-3.0)
   * Hébergement Europe (GDPR strict)
   * Aucune dépendance aux GAFAM
   * Contributions communautaires bienvenues

.. tip::
   **Gouvernance Démocratique**

   * ASBL belge sans actionnaires
   * 1 membre = 1 voix (Assemblée Générale)
   * Transparence comptable totale
   * Fonds de solidarité pour membres en difficulté

Vision 2025-2030
================

**Notre objectif : 5 000 copropriétés belges utilisant KoproGo**

.. list-table:: Progression par Paliers Mesurables
   :header-rows: 1
   :widths: 20 20 20 20 20

   * - Palier
     - Copropriétés
     - Économies/an
     - CO₂ évité/an
     - Impact Social
   * - **Validation**
     - 100
     - 80k€
     - -2 tonnes
     - Beta publique
   * - **Viabilité**
     - 500
     - 400k€
     - -15 tonnes
     - Production ouverte
   * - **Impact**
     - 1 000
     - 800k€
     - -107 tonnes
     - Communauté active
   * - **Leadership**
     - 2 000
     - 1,6M€
     - -214 tonnes
     - Référence belge
   * - **Référence**
     - 5 000
     - **4M€**
     - **-840 tonnes**
     - Leadership EU

**Philosophie** : Nous livrons quand les **capacités sont atteintes**, pas selon des dates arbitraires. Chaque palier débloque le suivant.

Parcours Guidé du Nouveau Contributeur
=======================================

.. important::
   **Vous découvrez KoproGo ? Suivez ce parcours étape par étape !**

**Étape 1 : Comprendre le Projet (15 min)**

1. Lisez :doc:`vision-strategie/pourquoi-koprogo` - Pourquoi KoproGo existe
2. Découvrez :doc:`vision-strategie/vision` - La vision à long terme
3. Explorez :doc:`vision-strategie/mission` - La mission et les valeurs

**Étape 2 : Découvrir la Roadmap (20 min)**

4. Consultez :doc:`roadmap/roadmap-2025-2030` - La roadmap 2025-2030
5. Voyez :doc:`roadmap/jalons-atteints` - Ce qui est déjà fait
6. Explorez :doc:`roadmap/jalons-a-venir` - Ce qui vient ensuite

**Étape 3 : Comprendre l'Architecture (30 min)**

7. Lisez :doc:`architecture/vue-ensemble` - Vue d'ensemble de l'architecture
8. Découvrez :doc:`architecture/choix-technologiques` - Pourquoi Rust, PostgreSQL, etc.

**Étape 4 : Commencer à Contribuer (1-2h)**

9. Suivez :doc:`contribuer/premiers-pas` - Premiers pas
10. Installez le projet : :doc:`contribuer/installer-projet`
11. Faites votre première contribution : :doc:`contribuer/faire-premiere-contribution`

**Besoin d'Aide ?**

* Consultez :doc:`contribuer/index` - Guide complet du contributeur
* Rejoignez `GitHub Discussions <https://github.com/gilmry/koprogo/discussions>`_
* Regardez les `Tutoriels YouTube <https://www.youtube.com/@koprogo>`_

Chiffres Clés (État Actuel)
============================

**Architecture & Code**

* **73 endpoints REST API** opérationnels
* **11 entités du domaine** (Organization, Building, Unit, Owner, Expense, etc.)
* **Architecture hexagonale** (Domain-Driven Design)
* **Tests E2E automatisés** avec Playwright
* **100% open-source** (AGPL-3.0)

**Performance Technique**

* **287 req/s** soutenus (charge réelle)
* **752ms** latence P99 (1 vCPU)
* **0,12g CO₂/requête** (96% réduction vs concurrence)
* **99,74% uptime** (infrastructure OVH)
* **128MB RAM** par instance (ultra-léger)

**Stack Technique**

* **Backend** : Rust 1.83 + Actix-web 4.9 + PostgreSQL 15
* **Frontend** : Astro 4.x + Svelte 4.x (PWA offline-first)
* **Infrastructure** : Terraform + Ansible + GitOps
* **Hébergement** : OVH France (Gravelines, bas carbone)

Table des Matières
==================

📖 Vision & Stratégie
---------------------

Comprenez pourquoi KoproGo existe et quelle est sa vision à long terme.

.. toctree::
   :maxdepth: 2

   vision-strategie/index
   vision-strategie/pourquoi-koprogo
   vision-strategie/vision
   vision-strategie/mission
   vision-strategie/impact
   vision-strategie/fonds-solidarite

🗺️ Roadmap 2025-2030
--------------------

Découvrez le chemin vers 5 000 copropriétés et l'impact sociétal prévu.

.. toctree::
   :maxdepth: 2

   roadmap/index
   roadmap/roadmap-2025-2030
   roadmap/jalons-atteints
   roadmap/jalons-a-venir
   ROADMAP_PAR_CAPACITES

🏗️ Architecture Technique
-------------------------

Explorez l'architecture hexagonale, les choix technologiques et les patterns utilisés.

.. toctree::
   :maxdepth: 2

   architecture/index
   architecture/vue-ensemble
   architecture/choix-technologiques
   backend/index
   frontend/index
   infrastructure/index

🤝 Guide du Contributeur
------------------------

Apprenez à contribuer au projet, de l'installation à votre première pull request.

.. toctree::
   :maxdepth: 2

   contribuer/index
   contribuer/premiers-pas
   contribuer/installer-projet
   contribuer/faire-premiere-contribution
   contribuer/comprendre-rfc-adr
   contribuer/standards-code

📜 Gouvernance & Décisions
--------------------------

Comprenez la gouvernance ASBL, les RFC/ADR et le processus de décision.

.. toctree::
   :maxdepth: 2

   gouvernance/index
   gouvernance/modele-asbl
   GOVERNANCE
   governance/togaf/adm
   governance/nexus/framework
   governance/scrum/ceremonies
   governance/rfc/template
   governance/adr/0001-mcp-integration

💰 Modèle Économique
--------------------

Découvrez le modèle économique OpenCore et la transparence financière.

.. toctree::
   :maxdepth: 2

   economic-model/index
   economic-model/modele-economique
   economic-model/transparence-comptable
   ECONOMIC_MODEL

💻 Documentation Technique
--------------------------

Documentation technique détaillée pour développeurs.

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
   :caption: 🔧 API & Déploiement

   api/README
   deployment/index

🏗️ Architecture Decision Records (ADR)
---------------------------------------

Historique des décisions d'architecture importantes.

.. toctree::
   :maxdepth: 2

   adr/0001-rust-actix-web-backend
   adr/0002-hexagonal-architecture
   adr/0003-postgresql-database
   adr/0004-astro-svelte-frontend
   adr/0005-jwt-authentication
   adr/0006-agpl-license
   adr/0044-document-storage-strategy

📊 GitHub Project Management
-----------------------------

Suivi du projet via GitHub Issues, Milestones et Projects.

.. toctree::
   :maxdepth: 2

   github-export/index

🚨 Operations & SRE
-------------------

Guides d'exploitation, monitoring, backups et incidents.

.. toctree::
   :maxdepth: 2

   operations/disaster-recovery
   operations/monitoring-runbook
   operations/backup-recovery
   operations/incident-response

🔒 Sécurité & Conformité
-------------------------

GDPR, comptabilité belge (PCMN), workflow de facturation et recouvrement.

.. toctree::
   :maxdepth: 2

   BELGIAN_ACCOUNTING_PCMN
   INVOICE_WORKFLOW
   PAYMENT_RECOVERY_WORKFLOW
   GDPR_COMPLIANCE_CHECKLIST
   GDPR_IMPLEMENTATION_STATUS
   GDPR_ADDITIONAL_RIGHTS
   BOARD_OF_DIRECTORS_GUIDE

💰 Finances & Performance
--------------------------

Rapports de performance, simulations de coûts et données financières.

.. toctree::
   :maxdepth: 2

   INVESTOR_EXECUTIVE_SUMMARY_2025
   INFRASTRUCTURE_COST_SIMULATIONS_2025
   PERFORMANCE_REPORT
   PERFORMANCE_TESTING

🎨 Frontend & Internationalisation
-----------------------------------

Composants frontend, templates email et guide i18n.

.. toctree::
   :maxdepth: 2

   FRONTEND_COMPONENTS
   EMAIL_TEMPLATES
   I18N_GUIDE

Rejoignez la Communauté
=======================

**KoproGo est un projet collaboratif et ouvert. Votre contribution compte !**

.. tip::
   **Comment Contribuer ?**

   * **Code** : Consultez les `issues GitHub <https://github.com/gilmry/koprogo/issues>`_ étiquetées "good first issue"
   * **Documentation** : Améliorez cette documentation via pull requests
   * **Traductions** : Ajoutez le support pour d'autres langues (NL, DE, EN)
   * **Tests** : Rejoignez le programme beta et testez la plateforme
   * **Feedback** : Partagez vos idées sur `GitHub Discussions <https://github.com/gilmry/koprogo/discussions>`_

**Liens Utiles**

* Code source : https://github.com/gilmry/koprogo
* Discussions : https://github.com/gilmry/koprogo/discussions
* Tutoriels vidéo : https://www.youtube.com/@koprogo
* Documentation : https://koprogo.readthedocs.io (à venir)

Principes Fondamentaux
======================

.. note::
   **Nos Valeurs**

   ✅ **Open-Source d'abord** : Code public, auditable, contributible

   ✅ **Démocratie tarifaire** : 1 membre = 1 voix, prix voté en AG

   ✅ **Qualité avant vitesse** : Livraison quand c'est prêt, pas selon un calendrier

   ✅ **Écologie par design** : Architecture optimisée pour réduire l'empreinte carbone

   ✅ **Transparence totale** : Comptabilité publique, décisions ouvertes

   ✅ **Solidarité intégrée** : Fonds de solidarité pour membres en difficulté

   ✅ **Souveraineté des données** : Hébergement Europe, conformité GDPR stricte

----

*Documentation maintenue par la communauté KoproGo ASBL*

*Modèle de progression : Capacités et métriques, pas dates fixes*

*Contact : contact@koprogo.com - GitHub : github.com/gilmry/koprogo*
