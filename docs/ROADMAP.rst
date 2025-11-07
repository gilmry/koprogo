
KoproGo - Roadmap 2025-2026
===========================

**Date de mise à jour**\ : 7 novembre 2025
**Début effectif**\ : Novembre 2025
**Version**\ : 2.1 (Gap Analysis + Énergie/IoT Features)
**Durée totale estimée**\ : 27-35 semaines (6.5-8.5 mois)
**Issues totales**\ : 28 (358-456 heures)

----

📋 Table des Matières
---------------------


#. `Vue d'ensemble <#-vue-densemble>`_
#. `Architecture & Stratégie <#-architecture--stratégie>`_
#. `Phase 1: VPS MVP <#-phase-1-vps-mvp-novembre-2025---février-2026>`_
#. `Phase 2: K3s <#-phase-2-k3s-mars---mai-2026>`_
#. `Phase 3: K8s Production <#️-phase-3-k8s-production-juin---août-2026>`_
#. `Timeline Globale <#-timeline-globale>`_
#. `Dépendances Critiques <#-dépendances-critiques>`_
#. `Ressources & Liens <#-ressources--liens>`_

----

⚠️ IMPORTANT: Modèle Économique Participatif
---------------------------------------------

**KoproGo repose sur les économies d'échelle inversées**: Plus de participants = Prix baisse pour tous.

**Votre participation compte** :

* ✅ Chaque nouvelle copropriété **dilue les coûts** pour toutes les autres
* ✅ Paliers dégressifs **automatiques** (500/1k/2k/5k copros)
* ✅ Surplus **réinvesti** selon vote AG annuel
* ✅ Contributions **valorisées** (-50% tarif pour contributeurs)

**Effet concret** :

.. code-block:: text

   100 copros:   1.00€/mois (prix lancement)
   500 copros:   0.80€/mois (-20% automatique)
   1,000 copros: 0.60€/mois (-40% automatique)
   5,000 copros: 0.40€/mois (-60% automatique)

   Même infrastructure, meilleur service, prix divisé par 2.5

**Rejoindre KoproGo = Contribuer au bien commun + Économiser**

.. note::
   **Détails complets** : Voir :doc:`ECONOMIC_MODEL` (Section Économies d'Échelle Participatives) pour projections 2025-2030, grille tarifaire, et transparence comptable.

----

🆕 Nouveautés Version 2.0 (Gap Analysis)
----------------------------------------

Cette version 2.0 intègre une **analyse complète des gaps** de conformité légale belge réalisée le 1er novembre 2025.

**Résultat Gap Analysis**\ :


* **93 features analysées** pour plateforme copropriété conforme législation belge
* **29% de complétude actuelle** (27 features implémentées, 14 partielles, 52 manquantes)
* **12 nouvelles issues créées** pour combler les gaps critiques
* **2 issues obsolètes supprimées** (#004 Pagination, #007 Work Management)
* **Total: 25 issues** couvrant tous les besoins (320-408 heures effort)

**Gaps Critiques Identifiés**\ :

#. ❌ **Conseil de Copropriété** (0% implémenté) - **OBLIGATION LÉGALE >20 lots** (Issue #022)
#. ❌ **Plan Comptable Normalisé Belge** (AR 12/07/2012) - 0% implémenté (Issue #016)
#. ❌ **État Daté** (Article 577-2) - BLOQUE toutes ventes immobilières (Issue #017)
#. ❌ **Budget Prévisionnel Annuel** - Requis légalement (Issue #018)
#. ❌ **Workflow Recouvrement** - Pas d'automatisation (Issue #023)

**Impact Roadmap**\ :


* Phase 1 étendue: +3-4 semaines (conformité légale prioritaire)
* Livraison finale: Septembre 2026 (vs Août 2026 précédemment)
* **PRIORITÉ #1**: Issue #022 (Conseil) - Bloque production >20 lots

**Documents**\ :


* Gap Analysis complète: `docs/GAP_ANALYSIS_KoproGov.md <./GAP_ANALYSIS_KoproGov.md>`_
* Issues détaillées: `issues/README.md <../issues/README.md>`_

----

🆕 Nouveautés Version 2.1 (Features Énergie & IoT)
---------------------------------------------------

Cette version 2.1 intègre **3 nouvelles features stratégiques** pour le marché belge :

**Nouvelles Issues**\ :

#. **Issue #028: Commande Groupée Énergie** (10-12h) - Phase 2

   * Agrégation consommations pour négociation collective électricité + gaz
   * Économies estimées : 15-30% sur factures énergétiques
   * Conforme réglementation belge (CWaPE, VREG, BRUGEL)

#. **Issue #029: Import Relevés ISTA** (6-8h) - Phase 2

   * Import automatique relevés compteurs ISTA (CSV/XML)
   * Historisation consommations eau/chauffage par unité
   * Détection anomalies et fuites

#. **Issue #030: Intégration Sondes IoT** (18-24h) - Phase 3

   * Monitoring temps réel via MQTT (eau froide/chaude, gaz, électricité, cogénération)
   * Alertes instantanées (fuites, surconsommations)
   * Architecture événementielle avec TimescaleDB

**Impact Roadmap**\ :

* Phase 2 : +2 issues (16-20h) → Total Phase 2 : **12 issues** (65-78 jours)
* Phase 3 : +1 issue (18-24h) → Total Phase 3 : **2 issues + 6 features** (50-60 jours)
* **Durée totale** : +2-3 semaines (features énergie différenciateurs marché)

**Contexte Stratégique**\ :

Ces 3 features ciblent un besoin critique du marché belge des copropriétés :

* **Augmentation factures énergétiques** : +300% depuis 2021 (crise énergie)
* **Législation belge** : Libéralisation marché énergie 2007 → achats groupés légaux
* **Standard ISTA** : Leader européen sous-comptage (majorité copropriétés belges)
* **IoT monitoring** : Tendance émergente (détection fuites, économies 10-20%)

**Documents**\ :

* `Issue #028 <../issues/important/028-commande-groupee-energie.md>`_ (Commande Groupée Énergie)
* `Issue #029 <../issues/important/029-import-releves-ista.md>`_ (Import Relevés ISTA)
* `Issue #030 <../issues/important/030-integration-sondes-iot.md>`_ (Intégration Sondes IoT)

----

🎯 Vue d'ensemble
-----------------

KoproGo suit une approche progressive d'infrastructure avec développement logiciel parallèle :

.. code-block::

   VPS (Docker Compose) → K3s (Lightweight K8s) → K8s (Production)
            ↓                    ↓                      ↓
        GitOps              GitOps + ArgoCD        GitOps + ArgoCD
        Traefik             Traefik                Traefik

Objectifs par Phase
^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Phase
     - Infrastructure
     - Software Focus
     - Durée
   * - **VPS MVP**
     - Docker Compose + GitOps
     - Sécurité, GDPR, Storage, Board Tools
     - 9-13 semaines
   * - **K3s**
     - K3s + ArgoCD
     - Voting, PDF, Community, Contractor
     - 6-8 semaines
   * - **K8s**
     - Multi-node K8s + HA
     - Performance, Real-time, Analytics
     - 6-8 semaines


GitHub Projects
^^^^^^^^^^^^^^^


* **Software Roadmap**\ : `Project #2 <https://github.com/users/gilmry/projects/2>`_
* **Infrastructure Roadmap**\ : `Project #3 <https://github.com/users/gilmry/projects/3>`_

----

🏗️ Architecture & Stratégie
---------------------------

Stack Technique
^^^^^^^^^^^^^^^

**Backend**\ : Rust + Actix-web (Hexagonal Architecture)
**Frontend**\ : Astro + Svelte (SSG + Islands)
**Database**\ : PostgreSQL 15
**Reverse Proxy**\ : Traefik (toutes phases)
**GitOps**\ : Ansible + Terraform (toutes phases), ArgoCD (K3s/K8s)

Principes de Développement
^^^^^^^^^^^^^^^^^^^^^^^^^^


* **Hexagonal Architecture** (Ports & Adapters)
* **Domain-Driven Design** (DDD)
* **Test-Driven Development** (TDD)
* **Infrastructure as Code** (IaC)
* **GitOps Continuous Deployment**

État Actuel (Novembre 2025) - Gap Analysis
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**✅ Implémenté (29% complétude légale belge)**\ :


* 73 endpoints API REST
* 11 entités domain (Building, Unit, Owner, Expense, Meeting, etc.)
* Auth JWT + Refresh Tokens + Multi-rôles (SuperAdmin, Syndic, Accountant, Owner)
* Multi-tenancy complet (Organization + isolation données)
* Multi-owner support (junction table unit_owners avec quotités)
* 26 pages frontend + 49 composants Svelte
* PWA + offline mode (IndexedDB, Service Worker)
* i18n (4 langues: NL, FR, DE, EN)
* Terraform + Ansible (VPS OVH)
* Docker Compose production avec Traefik
* GitOps auto-deploy (systemd service)
* CI/CD complet (6 workflows GitHub Actions)
* GDPR Articles 15 & 17 (export + effacement)

**🚧 Gaps Identifiés (Gap Analysis complète)**\ :

**Voir**\ : `docs/GAP_ANALYSIS_KoproGov.md <./GAP_ANALYSIS_KoproGov.md>`_ (93 features analysées)

.. list-table::
   :header-rows: 1

   * - Statut
     - Nombre Features
     - % Complétion
   * - ✅ Implémenté
     - 27/93
     - 29%
   * - 🟡 Partiel
     - 14/93
     - 15%
   * - ❌ Manquant
     - 52/93
     - 56%

**Gaps Critiques Identifiés**\ :

* ❌ **Plan comptable normalisé belge** (AR 12/07/2012) - 0% implémenté
* ❌ **État daté** (Article 577-2 Code Civil) - BLOQUE ventes immobilières
* ❌ **Conseil de Copropriété** (Article 577-8/4) - OBLIGATOIRE >20 lots - 0% implémenté
* ❌ **Budget prévisionnel annuel** - Requis légalement
* ❌ **Workflow recouvrement** - Pas d'automatisation
* ❌ **Carnet d'entretien digital** - 0% implémenté
* ❌ **Convocations AG automatiques** - Workflow manuel
* ❌ **Génération PDF étendue** (PCN, états datés, PV) - Partiel
* ❌ **GDPR Articles 16, 18, 21** - Manquants
* ❌ **Accessibilité WCAG 2.1 AA** - 0% implémenté

**25 issues créées** pour combler ces gaps (voir phases ci-dessous)

----

🚀 Phase 1: VPS MVP + Conformité Légale Belge (Novembre 2025 - Mars 2026)
--------------------------------------------------------------------------

**Durée estimée**\ : 12-16 semaines
**Objectif**\ : Production-ready sur VPS OVH avec conformité légale belge complète

**PRIORITÉ ABSOLUE**\ : Conformité législation belge (Conseil, Budget, État daté, Plan comptable)

Infrastructure Critique (16-24 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

#39: LUKS Encryption at Rest ⏱️ 3-5 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Infrastructure | **Effort**\ : Medium

**Description**\ : Full-disk encryption avec LUKS pour données sensibles (GDPR).

**Tâches**\ :


* Configuration LUKS sur volumes Docker
* Cryptsetup automation dans Ansible
* Key management sécurisé (Vault ou secrets chiffrés)
* Documentation récupération en cas de perte clé

**Livrables**\ :


* Playbook Ansible avec LUKS setup
* Guide de récupération d'urgence
* Tests de restauration

----

#40: Encrypted Backups (GPG + S3) ⏱️ 5-7 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Infrastructure | **Effort**\ : Large

**Description**\ : Backups PostgreSQL automatisés, chiffrés GPG, stockés sur S3 OVH.

**Tâches**\ :


* Script backup PostgreSQL (pg_dump)
* Chiffrement GPG avant upload S3
* Cron job quotidien (2h du matin)
* Rétention: 7 daily, 4 weekly, 12 monthly
* Tests de restauration automatisés

**Livrables**\ :


* Script ``backup.sh`` avec GPG + S3
* Cron job configuré
* Documentation restauration
* Alertes en cas d'échec

----

#41: Monitoring Stack (Prometheus/Grafana/Loki) ⏱️ 5-7 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Infrastructure | **Effort**\ : Large

**Description**\ : Observabilité complète avec métriques, logs, dashboards.

**Tâches**\ :


* Docker Compose: Prometheus, Grafana, Loki, Promtail
* Exporters: Node Exporter, PostgreSQL Exporter, cAdvisor
* Dashboards Grafana (CPU, RAM, disk, PostgreSQL, containers)
* Alertes: disk > 80%, RAM > 90%, PostgreSQL down
* Log aggregation avec Loki

**Livrables**\ :


* Stack monitoring complète
* 5+ dashboards Grafana préconfigurés
* Alert Manager configuré
* Documentation accès & usage

----

#43: Security Hardening ⏱️ 3-5 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Infrastructure | **Effort**\ : Medium

**Description**\ : Durcissement sécurité production (fail2ban, CrowdSec, Suricata).

**Tâches**\ :


* fail2ban pour SSH et API endpoints
* CrowdSec WAF avec bouncer Traefik
* Suricata IDS (detection intrusions réseau)
* Automatic security updates (unattended-upgrades)
* Auditd pour logs système

**Livrables**\ :


* Playbook Ansible avec tous les outils
* Configuration fail2ban + CrowdSec
* Dashboards sécurité dans Grafana
* Documentation incidents & réponse

----

Software Critique - Conformité Légale Belge (40-51 heures) 🆕
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**NOUVEAU (Gap Analysis)**\ : Ces 5 issues comblent les gaps critiques de conformité légale.

#016: Plan Comptable Normalisé Belge ⏱️ 8-10h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Labels**\ : ``finance``, ``legal-compliance``

**Description**\ : Implémenter plan comptable conforme arrêté royal 12/07/2012 (classes 4, 5, 6, 7).

**Tâches**\ :


* Enum ``AccountCode`` avec 24+ codes (6000-7999)
* Migration SQL pour account_code dans expenses table
* Use cases génération bilan comptable + compte de résultat
* Endpoints ``GET /api/v1/financial/balance-sheet``, ``/income-statement``
* Frontend: rapports comptables avec drill-down par compte

**Livrables**\ :


* Entity ``Account`` + enum ``AccountCode``
* Génération bilan + compte de résultat conformes PCN belge
* Tests unitaires + E2E comptabilité
* Documentation PCN pour utilisateurs

**Bloque**\ : #017 (État daté), #018 (Budget), #003 (Rapports financiers)

**Voir**\ : `issues/critical/016-plan-comptable-belge.md <../issues/critical/016-plan-comptable-belge.md>`_

----

#017: État Daté Génération ⏱️ 6-8h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Labels**\ : ``legal-compliance``, ``pdf``

**Description**\ : Génération états datés pour mutations immobilières (Article 577-2 Code Civil).

**Impact**\ : **BLOQUE TOUTES LES VENTES DE LOTS** sans ce document légal.

**Tâches**\ :


* Entity ``EtatDate`` (building_id, unit_id, reference_date, data JSONB, status)
* Génération PDF conforme (16 sections légales requises)
* Workflow: demande → génération (max 15 jours) → délivrance
* Endpoints: ``POST /api/v1/units/:id/etat-date``, ``GET /api/v1/etat-dates/:id/pdf``
* Historique complet: appels de fonds, paiements, travaux votés, litiges

**Livrables**\ :


* Template PDF état daté conforme législation
* Workflow avec rappels si délai > 10 jours
* Tests E2E génération + validation contenu
* Documentation procédure notaires

**Dépend de**\ : #016 (Plan Comptable pour section financière)

**Voir**\ : `issues/critical/017-etat-date-generation.md <../issues/critical/017-etat-date-generation.md>`_

----

#018: Budget Prévisionnel Annuel ⏱️ 8-10h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Labels**\ : ``finance``, ``legal-compliance``

**Description**\ : Système budget annuel (ordinaire + extraordinaire) avec variance analysis.

**Tâches**\ :


* Entity ``Budget`` (fiscal_year, ordinary_budget, extraordinary_budget, status)
* Calcul automatique provisions mensuelles
* Variance analysis (budget vs actual) mensuelle
* Vote AG obligatoire avant exercice fiscal
* Endpoints: ``POST /api/v1/buildings/:id/budget``, ``GET /budget/:year/variance``
* Dashboard syndic: alertes dépassements budgétaires

**Livrables**\ :


* Système budget complet avec projections
* Génération PDF budget pour vote AG
* Alertes dépassements > 10%
* Rapports variance trimestriels

**Dépend de**\ : #016 (Plan Comptable pour catégorisation)

**Voir**\ : `issues/critical/018-budget-previsionnel.md <../issues/critical/018-budget-previsionnel.md>`_

----

#022: Conseil de Copropriété ⏱️ 12-15h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Labels**\ : ``legal-compliance``, ``governance``

**Description**\ : **OBLIGATION LÉGALE** pour immeubles >20 lots (Article 577-8/4 Code Civil).

**Gap Critique**\ : **0% implémenté actuellement** - Bloque production pour copropriétés >20 lots.

**Tâches**\ :


* **Nouveau rôle**\ : ``BoardMember`` avec permissions spéciales
* Entity ``BoardMember`` (user_id, building_id, position, mandate_start/end)
* Entity ``BoardDecision`` (subject, decision_text, deadline, status)
* Élections conseil (vote AG) avec mandats 1 an renouvelables
* Dashboard conseil: suivi décisions AG + alertes retards syndic
* Tracking délais: devis (30j), travaux votés (60j), PV (30j)
* Rapports automatiques: semestriel + annuel pour AG
* Trigger SQL: vérification incompatibilité syndic ≠ conseil

**Livrables**\ :


* Rôle ``BoardMember`` opérationnel
* Workflow élections + mandats
* Dashboard suivi + alertes
* Rapports semestriels/annuels automatiques
* Tests BDD scenarios complets

**Bloque**\ : Production pour tout immeuble >20 lots (majorité du marché belge)

**Voir**\ : `issues/critical/022-conseil-copropriete.md <../issues/critical/022-conseil-copropriete.md>`_

----

#023: Workflow Recouvrement Impayés ⏱️ 6-8h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Labels**\ : ``finance``, ``automation``

**Description**\ : Workflow automatisé relances 3 niveaux (J+15, J+30, J+60 mise en demeure).

**Impact Business**\ : Réduction impayés 30-50% via automatisation.

**Tâches**\ :


* Entity ``PaymentReminder`` (expense_id, owner_id, level, sent_date, status)
* 3 niveaux: FirstReminder (J+15 aimable), SecondReminder (J+30 ferme), FormalNotice (J+60 légale)
* Génération PDF lettres (templates par niveau + langue)
* Cron job quotidien: détection impayés + envoi automatique
* Calcul pénalités retard (taux légal belge 8% annuel)
* Workflow: email → PDF lettre recommandée → procédure huissier
* Dashboard syndic: vue impayés + historique relances

**Livrables**\ :


* 3 templates PDF lettres (FR/NL/DE/EN)
* Cron job relances automatique
* Calcul pénalités conforme législation
* Tests E2E workflow complet

**Voir**\ : `issues/critical/023-workflow-recouvrement.md <../issues/critical/023-workflow-recouvrement.md>`_

----

Software Critique/High - Core Features (26-35 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

#44: Document Storage Strategy ⏱️ 2-3 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Effort**\ : Small

**Description**\ : Décision architecture stockage documents (local volume vs MinIO vs S3).

**Options**\ :


#. **Local volume Docker** (simple, pas de coût supplémentaire)
#. **MinIO container** (S3-compatible, self-hosted)
#. **S3 externe OVH** (managed, coût ~€0.01/GB/mois)

**Tâches**\ :


* Analyser pros/cons de chaque option
* Tester MinIO si choisi
* Implémenter abstraction storage dans backend (trait ``StorageProvider``\ )
* Migrer ``FileStorage`` pour utiliser la solution choisie

**Livrables**\ :


* Decision document (ADR - Architecture Decision Record)
* Implémentation backend avec abstraction
* Tests unitaires + intégration
* Documentation configuration

**Bloque**\ : #45 (File Upload UI)

----

#45: File Upload UI ⏱️ 3-5 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Effort**\ : Medium

**Description**\ : Interface upload documents avec preview, drag-drop, progress.

**Tâches**\ :


* Composant Svelte ``FileUploader.svelte``
* Drag & drop + file picker
* Progress bar upload
* Preview images/PDFs
* Validation côté client (type, size max 10MB)
* Liste documents avec download/delete

**Livrables**\ :


* Composant réutilisable
* Intégration pages Documents
* Tests E2E upload/download
* Documentation usage

**Dépend de**\ : #44 (storage backend doit être choisi)

----

#48: Strong Authentication (itsme®/eID) ⏱️ 8-10 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 Critical | **Track**\ : Software | **Effort**\ : Large

**Description**\ : Authentification forte OIDC pour votes légaux (itsme® Belgique, eID).

**Tâches**\ :


* Registration itsme® (2-4 semaines délai externe, parallèle)
* Intégration OIDC backend (crate ``openidconnect``\ )
* Nouveau endpoint ``/auth/itsme/callback``
* Frontend: bouton "Se connecter avec itsme®"
* Lien compte existant avec identité forte
* Audit trail votes avec signature OIDC

**Livrables**\ :


* Integration itsme® fonctionnelle
* Tests E2E authentification forte
* Documentation compliance légale
* Guide utilisateur

**Bloque**\ : #46 (Voting System - requis pour validité légale)

----

#42: GDPR Data Export & Deletion ⏱️ 5-7 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Effort**\ : Large

**Description**\ : Endpoints GDPR pour export données personnelles + droit à l'oubli.

**Tâches**\ :


* Endpoint ``GET /api/v1/users/me/export`` (JSON complet)
* Endpoint ``DELETE /api/v1/users/me`` (anonymisation cascade)
* Anonymisation vs suppression réelle (constraints légales)
* UI: page "Mes données" avec boutons Export/Delete
* Logs audit pour toute demande GDPR
* Email confirmation avant suppression

**Livrables**\ :


* 2 nouveaux endpoints
* Tests unitaires + E2E
* Page frontend GDPR
* Documentation compliance

----

#51: Board of Directors Tools ⏱️ 8-10 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Effort**\ : Large

**Description**\ : Outils conseil de copropriété (sondages, tâches, rapports).

**Tâches**\ :


* **Sondages/Polls**\ : 4 types (yes/no, multiple choice, rating, text)

  * Création, édition, publication
  * Notification propriétaires
  * Résultats temps réel + export PDF

* **Task Management**\ : Kanban pour conseil (Todo/InProgress/Done)
* **Issue Reporting**\ : Signalement problèmes bâtiment avec photos
* **Decision Log**\ : Historique décisions importantes avec contexte

**Nouveau rôle**\ : ``BoardMember`` (permissions spéciales)

**Livrables**\ :


* 4 nouvelles entités domain (Poll, Task, Issue, Decision)
* API complète + handlers
* 4 pages frontend + composants
* Tests BDD (Gherkin scenarios)

----

Recap Phase 1 - Conformité Légale Belge Prioritaire
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Catégorie
     - Issues
     - Effort Total
   * - **Infrastructure**
     - #39, #40, #41, #43
     - 16-24 jours
   * - **🆕 Conformité Légale Belge**
     - #016, #017, #018, #022, #023
     - 40-51 heures (5-6 jours)
   * - **Software Core**
     - #44, #45, #48, #42, #51
     - 26-35 jours
   * - **Total Phase 1**
     - **14 issues**
     - **47-65 jours** (12-16 semaines)


**Priorités Critiques Phase 1**\ :

#. 🔴 **#022 (Conseil)** + **#016 (PCN)** + **#017 (État daté)** - Bloquants légaux
#. 🔴 **#39-41** (Infrastructure sécurisée) - Requis GDPR
#. 🟡 **#48** (Strong Auth) → #46 (Voting) Phase 2
#. 🟡 Autres features automation (#018, #023, #42, #51)

**Notes**\ :

* **Conseil Copropriété (#022)**\ : PRIORITÉ #1 - Bloque >20 lots (majorité marché)
* **itsme® registration (#48)**\ : 2-4 semaines (externe), démarrer immédiatement en parallèle
* **Plan Comptable (#016)**\ : Bloque #017, #018, #003 - Démarrer semaine 1

----

🚀 Phase 2: K3s + Automation & Community (Mars - Juin 2026)
------------------------------------------------------------

**Durée estimée**\ : 8-11 semaines
**Objectif**\ : Migration K3s + Automation workflow + Features communautaires

Infrastructure K3s (~15 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Tâches**\ :


* Terraform: Provisionning cluster K3s (multi-node ou single-node HA)
* Ansible: Configuration K3s + Traefik ingress
* ArgoCD setup (GitOps CD)
* Cert-manager (Let's Encrypt automatique)
* Monitoring adapté K3s (ServiceMonitor Prometheus Operator)
* Migration données VPS → K3s

**Livrables**\ :


* Cluster K3s opérationnel
* ArgoCD configuré avec app definitions
* Playbooks Ansible K3s
* Documentation migration

----

Software Features - Automation & GDPR (27-35 heures) 🆕
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**NOUVEAU (Gap Analysis)**\ : Automation workflow + GDPR compliance complète.

#019: Convocations AG Automatiques ⏱️ 5-7h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``automation``, ``legal-compliance``

**Description**\ : Génération automatique convocations AG avec PDF + email + vérification délais légaux.

**Tâches**\ :


* Templates PDF convocations (FR/NL/DE/EN)
* Vérification délais: 15 jours (AG ordinaire), 8 jours (extraordinaire)
* Génération automatique: ordre du jour + annexes
* Envoi email automatique avec PDF attaché
* Accusés réception + relance J-3 si non ouvert
* Tracking présences prévues vs effectives

**Livrables**\ :


* Templates multi-langue conformes législation
* Workflow automatique complet
* Tests E2E convocation → réception
* Dashboard syndic: statut convocations

**Dépend de**\ : #001 (Meeting API doit être complète)

**Voir**\ : `issues/important/019-convocations-ag-automatiques.md <../issues/important/019-convocations-ag-automatiques.md>`_

----

#020: Carnet d'Entretien Digital ⏱️ 10-12h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``maintenance``, ``legal-compliance``

**Description**\ : Carnet d'entretien digital avec rapports travaux, inspections techniques, garanties.

**Gap**\ : 0% implémenté - Obligation légale belge pour suivi maintenance.

**Tâches**\ :


* Entity ``WorkReport`` (contractor, date, description, photos, cost)
* Entity ``TechnicalInspection`` (type, inspector, date, report, next_due)
* Gestion garanties: 2 ans (défauts apparents), 10 ans (décennale)
* Alertes inspections obligatoires: ascenseur, chaudière, électricité
* Upload photos avec métadonnées EXIF
* Historique complet interventions par équipement
* Export PDF carnet pour vente/audit

**Livrables**\ :


* Carnet digital complet
* Alertes inspections automatiques
* Export PDF conforme pour notaires
* Tests E2E workflow maintenance

**Voir**\ : `issues/important/020-carnet-entretien.md <../issues/important/020-carnet-entretien.md>`_

----

#021: GDPR Articles Complémentaires ⏱️ 5-7h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``gdpr``, ``legal-compliance``

**Description**\ : Compléter GDPR avec Articles 16 (Rectification), 18 (Restriction), 21 (Objection).

**État actuel**\ : Articles 15 & 17 implémentés, manque 16, 18, 21.

**Tâches**\ :


* **Article 16 (Rectification)**\ : Endpoint ``PUT /api/v1/users/me/data`` + UI correction données
* **Article 18 (Restriction)**\ : Flag ``processing_restricted`` + freeze processing partiel
* **Article 21 (Objection)**\ : Opt-out marketing + traitements automatisés
* Audit logs pour toutes demandes GDPR
* Page frontend "Mes droits GDPR" complète
* Tests unitaires + E2E compliance

**Livrables**\ :


* 3 nouveaux endpoints GDPR
* UI complète droits utilisateurs
* Documentation compliance GDPR 100%
* Tests conformité

**Voir**\ : `issues/important/021-gdpr-articles-complementaires.md <../issues/important/021-gdpr-articles-complementaires.md>`_

----

#024: Module Devis Travaux ⏱️ 8-10h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``finance``, ``quotes``

**Description**\ : Gestion devis avec comparaison multi-entrepreneurs + scoring automatique.

**Obligation légale**\ : 3 devis obligatoires pour travaux >€5000.

**Tâches**\ :


* Entity ``Quote`` (contractor, work_description, amount, validity_date, status)
* Comparaison multi-devis: tableau prix + délais + conditions
* Scoring automatique: prix (40%), délai (30%), garanties (20%), réputation (10%)
* Workflow: demande → réception → comparaison → vote AG → attribution
* Tracking: devis acceptés → WorkReport (carnet #020)
* Historique contractors: notes, délais respectés, qualité

**Livrables**\ :


* Système devis complet
* Algorithme scoring automatique
* Dashboard comparaison visuelle
* Tests E2E workflow

**Voir**\ : `issues/important/024-module-devis-travaux.md <../issues/important/024-module-devis-travaux.md>`_

----

#025: Affichage Public Syndic ⏱️ 3-4h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``frontend``, ``legal-compliance``

**Description**\ : Page publique (non authentifiée) affichant coordonnées syndic (obligation légale belge).

**Tâches**\ :


* Route ``/public/buildings/:slug/syndic`` (accessible sans auth)
* Affichage: nom syndic, adresse, téléphone, email, horaires permanence
* Option QR code pour accès mobile rapide
* SEO optimisé pour recherche "syndic [adresse immeuble]"
* Composant Svelte réutilisable

**Livrables**\ :


* Page publique syndic opérationnelle
* Tests E2E accessibilité publique
* Documentation SEO

**Voir**\ : `issues/important/025-affichage-public-syndic.md <../issues/important/025-affichage-public-syndic.md>`_

----

Software Features - Énergie & IoT (16-20 heures) 🆕
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**NOUVEAU (Version 2.1)**\ : Features stratégiques pour économies d'énergie et monitoring consommations.

#028: Commande Groupée Énergie (Électricité + Gaz) ⏱️ 10-12h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``energy``, ``finance``, ``belgium``

**Description**\ : Système d'agrégation consommations pour négociation collective avec fournisseurs énergie.

**Impact Business**\ : Économies 15-30% sur factures électricité/gaz pour copropriétés.

**Tâches**\ :


* 4 nouvelles entités : ``EnergyContract``, ``GroupPurchaseCampaign``, ``GroupPurchaseParticipant``, ``EnergyQuote``
* Workflow complet : Lancement campagne → Agrégation conso → Appel d'offres → Vote → Attribution
* Algorithme scoring fournisseurs (prix, services, origine énergie verte)
* Calcul économies prévisionnelles vs contrats actuels
* Intégration régulateurs belges (CWaPE, VREG, BRUGEL)
* Dashboard syndic : suivi campagnes + ROI
* API fournisseurs : envoi cahier des charges automatique
* Génération PDF comparatif offres pour AG

**Livrables**\ :


* Système achat groupé complet
* 8 endpoints API
* Dashboard comparaison fournisseurs
* Tests E2E workflow
* Documentation conformité légale belge

**Synergie**\ : Issue #029 (relevés ISTA) pour historique conso précis

**Voir**\ : `issues/important/028-commande-groupee-energie.md <../issues/important/028-commande-groupee-energie.md>`_

----

#029: Import Relevés ISTA (Historique Consommations) ⏱️ 6-8h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Labels**\ : ``automation``, ``energy``, ``belgium``

**Description**\ : Import automatique relevés compteurs ISTA (eau froide/chaude, chauffage) pour historisation.

**Contexte**\ : ISTA = leader européen sous-comptage, majorité copropriétés belges équipées.

**Tâches**\ :


* 3 nouvelles entités : ``ISTAReading``, ``ISTAImportBatch``, ``MeterUnitMapping``
* Parsers CSV + XML pour fichiers ISTA
* Auto-mapping compteurs → unités (numéro série)
* Détection anomalies consommation (variation >30%)
* Génération rapports consommation par période
* Export Excel pour comptabilité
* Validation croisée avec factures fournisseurs

**Livrables**\ :


* Système import multi-format (CSV/XML)
* Rapports analyse consommation
* Détection anomalies + alertes
* Tests intégration PostgreSQL
* Documentation guide import

**Synergie**\ : Issue #030 (IoT) pour validation croisée capteurs temps réel vs relevés manuels

**Voir**\ : `issues/important/029-import-releves-ista.md <../issues/important/029-import-releves-ista.md>`_

----

#027: Accessibilité WCAG 2.1 AA ⏱️ 8-10h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Frontend | **Labels**\ : ``accessibility``, ``wcag``

**Description**\ : Conformité WCAG 2.1 Level AA complète.

**Gap**\ : 0% implémenté - Audit accessibilité nécessaire.

**Tâches**\ :


* Audit accessibilité complet (automated + manual)
* Ratios contraste conformes (4.5:1 texte, 3:1 large)
* Navigation clavier complète (focus visible, tab order logique)
* ARIA labels sur tous composants interactifs
* Landmarks ARIA (navigation, main, complementary)
* Tests screen readers (NVDA, VoiceOver)
* Skip links + page titles descriptifs
* Forms: labels explicites + messages erreur clairs

**Livrables**\ :


* Conformité WCAG 2.1 AA validée
* Documentation accessibilité
* Tests automatisés (axe-core)
* Guide développeurs a11y

**Voir**\ : `issues/important/027-accessibilite-wcag.md <../issues/important/027-accessibilite-wcag.md>`_

----

Software Features - Voting & PDF (31-39 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

#47: PDF Generation Extended ⏱️ 5-7 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Effort**\ : Large

**Description**\ : Extension génération PDF (PCN, procès-verbaux, résultats votes).

**Tâches**\ :


* Templates PDF pour PCN (Précompte charges)
* Template procès-verbal assemblée générale
* Template résultats votes avec signatures
* Multi-langue (FR/NL/DE/EN)
* Watermark officiel + timestamps

**Livrables**\ :


* 3 nouveaux templates PDF
* Tests génération + assertions contenu
* Documentation templates

----

#46: Meeting Voting System ⏱️ 8-10 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟡 High | **Track**\ : Software | **Effort**\ : Large

**Description**\ : Système votes assemblées générales avec authentification forte.

**Tâches**\ :


* Entité ``Vote`` (meeting_id, user_id, option, signature_oidc)
* Endpoints: create vote, get results, close voting
* UI: Page vote avec countdown
* Validation: 1 vote par propriétaire (pondération tantièmes)
* Résultats temps réel (WebSocket ou polling)
* Audit trail complet avec signature itsme®

**Livrables**\ :


* Système voting complet
* Tests BDD scenarios
* Page frontend + composant
* Export PDF résultats

**Dépend de**\ : #48 (Strong Auth requis pour validité légale)

----

#49: Community Features ⏱️ 10-12 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟢 Medium | **Track**\ : Software | **Effort**\ : X-Large

**Description**\ : Fonctionnalités communautaires pour dynamique sociale (mission ASBL).

**Modules**\ :


#. **SEL (Système d'Échange Local)**\ : Troc compétences entre habitants
#. **Skills Directory**\ : Annuaire compétences (bricolage, jardinage, cours, etc.)
#. **Object Sharing**\ : Prêt objets (outils, échelles, tondeuse)
#. **Notice Board**\ : Tableau d'affichage numérique (petites annonces)
#. **Swap Shop (Bazar de Troc)**\ : Échange/don objets entre habitants

**Tâches**\ :


* 5 nouvelles entités domain (SkillOffer, ObjectLoan, Notice, SwapItem, Transaction)
* API complète pour chaque module
* Frontend: 5 pages dédiées + composants
* Notifications (email/push)
* Moderation tools (signalement contenu inapproprié)

**Livrables**\ :


* 5 modules fonctionnels
* Tests E2E pour chaque module
* Documentation usage communauté
* Guide modération

----

#52: Contractor Backoffice ⏱️ 8-10 jours
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟢 Medium | **Track**\ : Software | **Effort**\ : Large

**Description**\ : Backoffice léger prestataires (rapports travaux, photos, paiement).

**Tâches**\ :


* Rôle ``Contractor`` avec auth simplifiée (PIN ou lien magique)
* Page rapport travaux: description, photos, pièces changées
* Upload photos avec métadonnées (date, lieu, intervention)
* Soumission facture avec montant
* Workflow validation syndic → paiement
* Historique interventions par prestataire

**Livrables**\ :


* Entité ``WorkReport`` + ``ContractorInvoice``
* API + handlers
* Backoffice frontend (mobile-friendly)
* Tests E2E workflow complet

----

Recap Phase 2 - Automation & Community + Énergie
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Catégorie
     - Issues
     - Effort Total
   * - **Infrastructure**
     - K3s setup + ArgoCD
     - ~15 jours
   * - **🆕 Automation & GDPR**
     - #019, #020, #021, #024, #025, #027
     - 39-50 heures (5-6 jours)
   * - **🆕 Énergie & IoT**
     - #028, #029
     - 16-20 heures (2-3 jours)
   * - **Software Voting & PDF**
     - #47, #46
     - 13-17 jours
   * - **Software Community**
     - #49, #52
     - 18-22 jours
   * - **Total Phase 2**
     - **12 issues + infra**
     - **54-63 jours** (8.5-12 semaines)


**Priorités Phase 2**\ :

#. 🔴 K3s migration (bloque Phase 3)
#. 🟡 #019, #020, #021, #024, #027 (automation + compliance)
#. 🟡 #028, #029 (énergie + ISTA) - différenciateurs marché + ROI direct
#. 🟡 #46 (Voting) + #47 (PDF Extended)
#. 🟢 #49 (Community) + #52 (Contractor) - différenciateurs marché


----

☸️ Phase 3: K8s Production (Juin - Août 2026)
---------------------------------------------

**Durée estimée**\ : 6-8 semaines
**Objectif**\ : K8s multi-node, HA, performance, features avancées

Infrastructure K8s (~15 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Tâches**\ :


* Terraform: Multi-node K8s cluster (3+ nodes)
* Ansible: Configuration HA (etcd, control plane)
* PostgreSQL HA (Patroni ou CloudNativePG operator)
* Redis/Valkey distributed cache
* Advanced monitoring (distributed tracing: Jaeger/Tempo)
* Horizontal Pod Autoscaling (HPA)
* Network policies (sécurité inter-pods)

**Livrables**\ :


* Cluster K8s production-grade
* HA PostgreSQL opérationnel
* Cache distribué
* Documentation architecture K8s

----

Software Advanced Features (30-40 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

#026: Modules Communautaires (Mission ASBL) ⏱️ 15-20h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🟢 Nice-to-Have | **Track**\ : Software | **Labels**\ : ``community``, ``asbl-mission``

**Description**\ : 5 modules communautaires pour dynamique sociale et impact environnemental.

**Mission ASBL**\ : Résolution "phénomènes sociétés" via partage et solidarité.

**Modules**\ :

#. **SEL (Système d'Échange Local)**\ : Troc compétences (monnaie locale virtuelle)
#. **Swap Shop (Bazar de Troc)**\ : Échange/don objets entre habitants
#. **Object Lending Library**\ : Prêt outils, échelles, tondeuse, etc.
#. **Skills Directory**\ : Annuaire compétences (bricolage, cours, jardinage)
#. **Digital Notice Board**\ : Tableau affichage petites annonces

**Tâches**\ :


* 5 entities: ``SelOffer``, ``SwapItem``, ``LendableObject``, ``SkillOffer``, ``Notice``
* Gamification: badges (Super Partageur, Éco-Héros), leaderboard copropriété
* Modération: signalement contenu + workflow validation
* Notifications: email + push pour nouveaux items
* Tracking impact: CO2 économisé, objets réutilisés, compétences partagées
* Rapport annuel impact social (export PDF pour AG)

**Livrables**\ :


* 5 modules fonctionnels complets
* Système gamification + badges
* Outil modération
* Dashboard impact social
* Tests E2E chaque module

**Voir**\ : `issues/nice-to-have/026-modules-communautaires.md <../issues/nice-to-have/026-modules-communautaires.md>`_

----

#030: Intégration Sondes IoT (Monitoring Temps Réel) ⏱️ 18-24h 🆕
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Priority**\ : 🔴 High | **Track**\ : Software | **Labels**\ : ``iot``, ``real-time``, ``energy``, ``monitoring``

**Description**\ : Monitoring temps réel consommations via capteurs IoT (eau froide/chaude, gaz, électricité, cogénération).

**Impact**\ : Détection immédiate fuites + économies 10-20% via optimisation consommations.

**Architecture**\ :


* **MQTT Broker** (Mosquitto) pour ingestion capteurs
* **Service Ingestion IoT** autonome (subscribe MQTT → TimescaleDB)
* **TimescaleDB** (hypertable) pour time-series optimisé
* **WebSocket** (Actix) pour streaming temps réel frontend
* **Redis Streams** pour buffering et résilience

**Tâches**\ :


* 3 nouvelles entités : ``IoTSensor``, ``IoTReading`` (time-series), ``IoTAlert``
* Service ingestion MQTT (Rust async avec rumqttc)
* Rule engine détection anomalies (seuils configurables)
* Service alertes (email/SMS/push) en temps réel
* API endpoints : CRUD sensors, time-series queries, WebSocket stream
* Dashboard frontend temps réel (Chart.js Streaming)
* Déploiement K8s : StatefulSet + Mosquitto Broker
* Tests charge (10k mesures/seconde)

**Livrables**\ :


* Architecture IoT complète (MQTT → TimescaleDB → WebSocket)
* 3 entités domain + repositories
* Service ingestion indépendant
* Dashboard temps réel avec graphiques live
* Alertes automatiques anomalies
* Tests E2E + tests charge
* Documentation installation capteurs

**Synergie**\ : Issue #029 (ISTA) pour validation croisée relevés manuels vs capteurs temps réel

**Dépendances Infrastructure**\ : TimescaleDB extension PostgreSQL, MQTT Broker, K8s StatefulSets

**Voir**\ : `issues/important/030-integration-sondes-iot.md <../issues/important/030-integration-sondes-iot.md>`_

----

**Features Performance & Analytics**\ :


#. **ScyllaDB/DragonflyDB Integration**\ : NoSQL pour performance lectures (sessions, cache)
#. **Real-time Notifications**\ : WebSocket avec Actix pour notifications temps réel
#. **Advanced Analytics Dashboard**\ : Métriques métier (occupancy rate, expense trends, meeting attendance)
#. **Mobile App**\ : React Native ou Flutter (offline-first)
#. **Advanced Search**\ : ElasticSearch/MeiliSearch pour recherche full-text
#. **Audit Dashboard**\ : Visualisation audit logs pour SuperAdmin

**Livrables**\ :


* #026 (Modules communautaires) + 6 features performance
* Tests performance (benchmarks Criterion)
* Documentation scalabilité
* Mobile app (MVP)

----

Recap Phase 3 - Performance & Community + IoT
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Catégorie
     - Issues/Features
     - Effort Total
   * - **Infrastructure K8s**
     - Multi-node HA + monitoring
     - ~15 jours
   * - **🆕 Community ASBL**
     - #026 (5 modules)
     - 15-20 heures (2 jours)
   * - **🆕 IoT Monitoring**
     - #030 (capteurs temps réel)
     - 18-24 heures (3 jours)
   * - **Software Performance**
     - 6 features (NoSQL, WebSocket, Analytics, Mobile, Search, Audit)
     - 30-40 jours
   * - **Total Phase 3**
     - **2 issues + 6 features + infra**
     - **50-60 jours** (7.5-10 semaines)


----

📅 Timeline Globale (Gap Analysis Intégrée)
-------------------------------------------

.. code-block::

   Nov 2025          Mars 2026         Juin 2026         Sept 2026
      |                 |                 |                 |
      v                 v                 v                 v
   ┌──────────────────┐ ┌───────────────┐ ┌───────────────┐
   │   VPS MVP +      │ │   K3s +       │ │  K8s Prod +   │
   │ Legal Compliance │ │  Automation   │ │  Performance  │
   │  (12-16 sem.)    │ │  (8-11 sem.)  │ │  (7-9 sem.)   │
   └──────────────────┘ └───────────────┘ └───────────────┘
   🔴 PRIORITÉ:         Voting, PDF,      Performance,
   Conseil Copropriété, Automation,       Real-time,
   Plan Comptable,      Community,        Analytics,
   État Daté,           Contractor,       Mobile App,
   Budget, Security,    GDPR Complete,    ASBL Community
   GDPR, Backups        Maintenance       Modules

Dates Clés (Mises à Jour)
^^^^^^^^^^^^^^^^^^^^^^^^^^


* **Novembre 2025**\ : Début Phase 1 (VPS MVP + Legal Compliance)
* **Mars 2026**\ : Fin Phase 1 (conformité légale belge complète), début Phase 2 (K3s)
* **Juin 2026**\ : Fin Phase 2 (automation + community), début Phase 3 (K8s)
* **Septembre 2026**\ : KoproGo 1.0 Production-Ready avec conformité légale 100%

Effort Total Mis à Jour
^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Phase
     - Issues/Features
     - Durée
     - Fin Prévue
   * - **Phase 1 (VPS + Legal)**
     - 14 issues
     - 12-16 semaines
     - Mars 2026
   * - **Phase 2 (K3s + Automation + Énergie)**
     - 12 issues
     - 8.5-12 semaines
     - Juin 2026
   * - **Phase 3 (K8s + Performance + IoT)**
     - 2 issues + 6 features
     - 7.5-10 semaines
     - Septembre 2026
   * - **TOTAL**
     - **28 issues + features**
     - **28-38 semaines** (~7-9.5 mois)
     - **Septembre 2026**


**Changements vs Version 2.0**\ :

* +3 issues Énergie & IoT (Version 2.1): #028, #029 (Phase 2), #030 (Phase 3)
* +5 issues Belgian Legal Compliance (Version 2.0 - Phase 1): #016, #017, #018, #022, #023
* +6 issues Automation & Accessibility (Version 2.0 - Phase 2): #019, #020, #021, #024, #025, #027
* +1 issue Community Modules (Version 2.0 - Phase 3): #026
* -2 issues obsolètes supprimés (Version 2.0): #004 (Pagination), #007 (Work Management)
* **Durée totale**: +8-10 semaines vs Version 1.0 (conformité légale + énergie prioritaires)


----

🔗 Dépendances Critiques (Mises à Jour)
---------------------------------------

Chaînes de Dépendances - Phase 1
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block::

   🆕 #016 (Plan Comptable) ──▶ #017 (État Daté), #018 (Budget), #003 (Rapports)
   🆕 #022 (Conseil Copropriété) ──▶ BLOQUE production >20 lots (PRIORITÉ #1)

   #44 (Storage Strategy) ──▶ #45 (File Upload UI) ──▶ #002 (Documents)
   #48 (Strong Auth)      ──▶ #46 (Voting System - Phase 2)
   #39-41 (Security/Backup/Monitoring) ──▶ Production VPS

   Phase 1 Complete ──▶ Phase 2 (K3s)

Chaînes de Dépendances - Phase 2
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block::

   🆕 #001 (Meeting API) ──▶ #019 (Convocations AG), #022 (Conseil)
   🆕 #002 (Documents) ──▶ #017 (État Daté), #020 (Carnet), #024 (Devis)

   #48 (Strong Auth - Phase 1) ──▶ #46 (Voting)
   #020 (Carnet Entretien) ──▶ #024 (Devis Travaux - tracking)

   Phase 2 Complete ──▶ Phase 3 (K8s)

Ordre de Développement Optimal
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Phase 1 - Semaine 1-2 (Critique)**\ :

#. **#022 Conseil Copropriété** (12-15h) - BLOQUANT >20 lots
#. **#016 Plan Comptable** (8-10h) - Bloque #017, #018, #003
#. **#039-041 Infrastructure** (16-24 jours en parallèle)

**Phase 1 - Semaine 3-4**\ :

#. **#017 État Daté** (6-8h) - Dépend de #016
#. **#018 Budget Prévisionnel** (6-8h) - Dépend de #016
#. **#023 Workflow Recouvrement** (6-8h) - Indépendant
#. **#044-045 Storage + Upload** (5-8 jours)

**Phase 1 - Semaine 5-12**\ :

#. **#048 Strong Auth** (8-10 jours) - Bloque #046 Phase 2
#. **#042 GDPR Export/Deletion** (5-7 jours)
#. **#051 Board Tools** (8-10 jours)
#. **#001, #002, #003, #005** (Core features)

Risques & Mitigations (Mis à Jour)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Risque
     - Impact
     - Probabilité
     - Mitigation
   * - 🆕 **Conseil Copropriété non implémenté**
     - BLOQUE production >20 lots (majorité marché)
     - Haute (0% actuellement)
     - **PRIORITÉ #1** - Démarrer semaine 1
   * - 🆕 **Plan Comptable bloque chaîne**
     - Bloque #017, #018, #003 (État daté critique)
     - Moyenne
     - Démarrer semaine 1 en parallèle #022
   * - **itsme® registration delay**
     - Bloque #48 → #46 (Voting Phase 2)
     - Moyenne
     - Démarrer registration immédiatement (Nov 2025)
   * - 🆕 **Sous-estimation effort Belgian Compliance**
     - Retard Phase 1 (40-51h nouvelles)
     - Moyenne
     - Buffer +3 semaines dans Phase 1
   * - **Storage strategy indecision**
     - Bloque #45 → #002
     - Faible
     - Decision meeting semaine 1
   * - **K3s migration complexity**
     - Retard Phase 2
     - Moyenne
     - Tests migration sur env staging
   * - 🆕 **Accessibilité WCAG manuelle**
     - Retard Phase 2 (tests screen readers)
     - Moyenne
     - Audit externe + tests automatisés continus
   * - **Performance K8s**
     - Retard Phase 3
     - Faible
     - Benchmarks continus dès Phase 1


Dépendances Externes
^^^^^^^^^^^^^^^^^^^^


* **itsme® registration**\ : 2-4 semaines (processus externe Belgique)
* **OVH VPS/K3s/K8s**\ : Dispo immédiate (Terraform automation)
* **Let's Encrypt certificates**\ : Automatique (cert-manager)
* **S3 OVH**\ : Activation immédiate

----

📚 Ressources & Liens
---------------------

GitHub
^^^^^^


* **Repository**\ : https://github.com/gilmry/koprogo
* **Software Roadmap (Project #2)**\ : https://github.com/users/gilmry/projects/2
* **Infrastructure Roadmap (Project #3)**\ : https://github.com/users/gilmry/projects/3

Documentation Interne
^^^^^^^^^^^^^^^^^^^^^


* **CLAUDE.md**\ : Guide complet projet (architecture, commandes, API)
* **docs/deployment/**\ : Documentation infrastructure (Terraform, Ansible, GitOps)
* **docs/GIT_HOOKS.md**\ : Hooks pre-commit/pre-push
* **docs/unit_owners/**\ : Documentation multi-ownership

Issues par Phase
^^^^^^^^^^^^^^^^

**Phase 1 (VPS MVP)**\ :


* Infrastructure: `#39 <https://github.com/gilmry/koprogo/issues/39>`_\ , `#40 <https://github.com/gilmry/koprogo/issues/40>`_\ , `#41 <https://github.com/gilmry/koprogo/issues/41>`_\ , `#43 <https://github.com/gilmry/koprogo/issues/43>`_
* Software: `#44 <https://github.com/gilmry/koprogo/issues/44>`_\ , `#45 <https://github.com/gilmry/koprogo/issues/45>`_\ , `#48 <https://github.com/gilmry/koprogo/issues/48>`_\ , `#42 <https://github.com/gilmry/koprogo/issues/42>`_\ , `#51 <https://github.com/gilmry/koprogo/issues/51>`_

**Phase 2 (K3s)**\ :


* Software: `#47 <https://github.com/gilmry/koprogo/issues/47>`_\ , `#46 <https://github.com/gilmry/koprogo/issues/46>`_\ , `#49 <https://github.com/gilmry/koprogo/issues/49>`_\ , `#52 <https://github.com/gilmry/koprogo/issues/52>`_

Labels GitHub
^^^^^^^^^^^^^


* **Phases**\ : ``phase:vps``\ , ``phase:k3s``\ , ``phase:k8s``
* **Tracks**\ : ``track:software``\ , ``track:infrastructure``
* **Priority**\ : ``priority:critical``\ , ``priority:high``\ , ``priority:medium``\ , ``priority:low``

Technologies Clés
^^^^^^^^^^^^^^^^^


* **Backend**\ : Rust, Actix-web, SQLx, PostgreSQL 15
* **Frontend**\ : Astro, Svelte, Tailwind CSS
* **Infrastructure**\ : Terraform, Ansible, Docker Compose, K3s, K8s
* **GitOps**\ : ArgoCD, systemd service (VPS)
* **Monitoring**\ : Prometheus, Grafana, Loki
* **Security**\ : LUKS, GPG, fail2ban, CrowdSec, Suricata
* **Auth**\ : JWT, itsme® (OIDC)

----

🎯 Principes Directeurs
-----------------------

Performance Targets
^^^^^^^^^^^^^^^^^^^


* **Latency P99**\ : < 5ms
* **Throughput**\ : > 100k req/s (K8s phase)
* **Memory**\ : < 128MB per instance
* **Database pool**\ : Max 10 connections

Compliance & Security
^^^^^^^^^^^^^^^^^^^^^


* **GDPR**\ : Export/deletion, encryption at rest, audit logs
* **Legal voting**\ : Strong authentication (itsme®/eID)
* **Data protection**\ : LUKS + GPG backups
* **Security hardening**\ : fail2ban, CrowdSec, Suricata IDS

Sustainability (Mission ASBL)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


* **CO2 target**\ : < 0.5g CO2/request
* **Community features**\ : SEL, sharing, swap shop (résolution phénomènes sociétés)
* **Efficient infrastructure**\ : Progressive scaling (VPS → K3s → K8s)
* **Open source**\ : Contribution à l'écosystème Rust/Actix

----

**Dernière mise à jour**\ : 7 novembre 2025
**Version**\ : 2.1 (Gap Analysis + Énergie/IoT Features)
**Maintenu par**\ : KoproGo ASBL
**Contact**\ : `GitHub Issues <https://github.com/gilmry/koprogo/issues>`_

----

📊 Documents Associés
---------------------

* **Gap Analysis Complète**\ : `docs/GAP_ANALYSIS_KoproGov.md <./GAP_ANALYSIS_KoproGov.md>`_ (93 features analysées)
* **Issues Détaillées**\ : `issues/README.md <../issues/README.md>`_ (28 issues avec cahiers des charges)
* **Belgian Legal Compliance**\ :

  * `#016 Plan Comptable <../issues/critical/016-plan-comptable-belge.md>`_
  * `#017 État Daté <../issues/critical/017-etat-date-generation.md>`_
  * `#018 Budget Prévisionnel <../issues/critical/018-budget-previsionnel.md>`_
  * `#022 Conseil Copropriété <../issues/critical/022-conseil-copropriete.md>`_ (PRIORITÉ #1)
  * `#023 Workflow Recouvrement <../issues/critical/023-workflow-recouvrement.md>`_

* **🆕 Énergie & IoT Features (Version 2.1)**\ :

  * `#028 Commande Groupée Énergie <../issues/important/028-commande-groupee-energie.md>`_
  * `#029 Import Relevés ISTA <../issues/important/029-import-releves-ista.md>`_
  * `#030 Intégration Sondes IoT <../issues/important/030-integration-sondes-iot.md>`_

* **Architecture**\ : `CLAUDE.md <../CLAUDE.md>`_ (Hexagonal Architecture + DDD)
* **GDPR Compliance**\ : `docs/GDPR_COMPLIANCE_CHECKLIST.md <./GDPR_COMPLIANCE_CHECKLIST.md>`_
* **Multi-Owner Support**\ : `docs/unit_owners/ <./unit_owners/>`_
* **Deployment**\ : `docs/deployment/ <./deployment/>`_ (Terraform + Ansible)
