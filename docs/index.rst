======================================
Documentation KoproGo
======================================

**Plateforme open-source de gestion de copropriété pour la Belgique**

.. note::
   📺 **Tutoriels Vidéo** : `Chaîne YouTube @koprogo <https://www.youtube.com/@koprogo>`_

   Retrouvez des tutoriels vidéo pour démarrer avec KoproGo, comprendre l'architecture hexagonale, et découvrir les fonctionnalités avancées de la plateforme.

L'Histoire Humaine Derrière KoproGo
====================================

**Marie, 72 ans, retraitée à Bruxelles**

Son syndic lance des travaux de façade : 15 000€ sa quote-part. Elle conteste les devis qui lui semblent excessifs. Un avocat coûte 2 000€. Sa pension : 1 200€/mois.

**Ahmed, 35 ans, intérimaire**

Trois mois de chômage technique en 2024. Résultat : 3 200€ d'impayés de charges. Les huissiers interviennent. Les pénalités s'accumulent.

**Sofiane, 40 ans, auto-entrepreneur**

La toiture de sa copropriété doit être refaite : 12 000€ sa quote-part. Les banques refusent le prêt (pas de CDI). Les travaux sont bloqués. L'immeuble se dégrade.

----

**Ces situations reflètent les défis quotidiens de milliers de copropriétaires en Belgique.**

**KoproGo apporte des solutions concrètes** : plateforme de gestion accessible (5€/mois en cloud, gratuite en self-hosted), Fonds de Solidarité pour membres en difficulté, gouvernance démocratique (ASBL), et architecture optimisée réduisant coûts et empreinte carbone.

Qui Êtes-Vous ? (Choisissez Votre Parcours)
============================================

KoproGo s'adresse à différents profils. Choisissez le parcours qui vous correspond :

.. grid:: 2
   :gutter: 3

   .. grid-item-card:: 💼 Investisseur / Fondation
      :link: parcours-investisseur
      :link-type: doc

      **Vous évaluez KoproGo pour un investissement ou subvention ?**

      Découvrez le modèle économique, les projections financières,
      et l'impact sociétal attendu.

      ⏱️ **5-10 min**

   .. grid-item-card:: 👨‍💻 Développeur
      :link: contribuer/index
      :link-type: doc

      **Vous voulez contribuer au projet ?**

      Installez le projet, comprenez l'architecture,
      et faites votre première contribution.

      ⏱️ **30-60 min**

   .. grid-item-card:: 🏘️ Syndic / Copropriétaire
      :link: user-guides/syndic-guide
      :link-type: doc

      **Vous cherchez un outil de gestion ?**

      Découvrez les fonctionnalités et cas d'usage.

      ⏱️ **10-15 min**

   .. grid-item-card:: 🌍 Curieux du Projet
      :link: vision-strategie/pourquoi-koprogo
      :link-type: doc

      **Vous découvrez KoproGo ?**

      Comprenez pourquoi ce projet existe.

      ⏱️ **15-20 min**

Introduction Rapide
====================

**Le Contexte**

En Belgique, 200 000 copropriétés font face à des coûts de gestion élevés (200-500€/mois pour les solutions logicielles existantes), une empreinte carbone importante, et un manque de transparence dans les calculs de charges.

**La Solution KoproGo**

Une plateforme open-source développée par une ASBL belge :

.. grid:: 2
   :gutter: 2

   .. grid-item-card:: 💰 Économies Substantielles

      * **5€/mois** en cloud managé
      * **Gratuit** en self-hosted (AGPL-3.0)
      * **70M€/an** économisables collectivement en Belgique

   .. grid-item-card:: 🌱 Impact Écologique

      * **0,12g CO₂/requête** (architecture Rust)
      * **96% de réduction** vs solutions SaaS classiques
      * **840 tonnes CO₂/an évitées** à 5 000 copropriétés

   .. grid-item-card:: 🤝 Gouvernance Démocratique

      * ASBL belge sans actionnaires
      * 1 membre = 1 voix (AG)
      * Prix voté démocratiquement

   .. grid-item-card:: 💙 Solidarité Intégrée

      * Fonds de Solidarité pour membres en difficulté
      * Prêts à taux 0% pour impayés
      * Aide aux litiges démocratiques

**L'Impact Attendu** (5 000 copropriétés)

* **4M€/an économisés** collectivement
* **840 tonnes CO₂/an évitées**
* **40-60 copropriétaires aidés/an** financièrement

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

**Chaque fonctionnalité d'aujourd'hui prépare la vision de demain.**

→ Découvrez comment : :doc:`vision-strategie/de-gestion-a-symbiose`

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

**Ou suivez le parcours complet** : :doc:`parcours-contributeur`

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
* **0,12g CO₂/requête** (96% réduction vs solutions SaaS classiques)
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

   parcours-investisseur
   vision-strategie/index
   vision-strategie/pourquoi-koprogo
   vision-strategie/de-gestion-a-symbiose
   vision-strategie/vision
   vision-strategie/mission
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
