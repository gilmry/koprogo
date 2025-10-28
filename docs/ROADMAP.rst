
KoproGo - Roadmap 2025-2026
===========================

**Date de mise à jour**\ : 27 octobre 2024, 17:15
**Début effectif**\ : Novembre 2025
**Version**\ : 1.0
**Durée totale estimée**\ : 21-29 semaines (6-7 mois)

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

État Actuel (Octobre 2025)
^^^^^^^^^^^^^^^^^^^^^^^^^^

**✅ Implémenté**\ :


* 73 endpoints API REST
* 11 entités domain (Building, Unit, Owner, Expense, Meeting, etc.)
* Auth JWT + Refresh Tokens (4 rôles: SuperAdmin, Syndic, Accountant, Owner)
* Multi-tenancy (Organization)
* 26 pages frontend + 39 composants Svelte
* PWA + offline mode (IndexedDB, Service Worker)
* i18n (4 langues: NL, FR, DE, EN)
* Terraform + Ansible (VPS OVH)
* Docker Compose production avec Traefik
* GitOps auto-deploy (systemd service)
* CI/CD complet (6 workflows GitHub Actions)

**🚧 Gaps Identifiés**\ : 16 issues créés (voir phases ci-dessous)

----

🚀 Phase 1: VPS MVP (Novembre 2025 - Février 2026)
--------------------------------------------------

**Durée estimée**\ : 9-13 semaines
**Objectif**\ : Production-ready sur VPS OVH avec sécurité, GDPR, backups

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

Software Critique/High (26-35 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

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

Recap Phase 1
^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Catégorie
     - Issues
     - Effort Total
   * - **Infrastructure**
     - #39, #40, #41, #43
     - 16-24 jours
   * - **Software**
     - #44, #45, #48, #42, #51
     - 26-35 jours
   * - **Total Phase 1**
     - 9 issues
     - **42-59 jours** (9-13 semaines)


**Note**\ : Registration itsme® (#48) prend 2-4 semaines (processus externe), mais peut être faite en parallèle du développement.

----

🚀 Phase 2: K3s (Mars - Mai 2026)
---------------------------------

**Durée estimée**\ : 6-8 semaines
**Objectif**\ : Migration K3s avec ArgoCD, features communautaires avancées

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

Software Features (31-39 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

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

Recap Phase 2
^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Catégorie
     - Issues
     - Effort Total
   * - **Infrastructure**
     - K3s setup
     - ~15 jours
   * - **Software**
     - #47, #46, #49, #52
     - 31-39 jours
   * - **Total Phase 2**
     - 4 issues + infra
     - **46-54 jours** (6-8 semaines)


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

Software Advanced (30-40 jours)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Features**\ :


#. **ScyllaDB/DragonflyDB Integration**\ : NoSQL pour performance lectures (sessions, cache)
#. **Real-time Notifications**\ : WebSocket avec Actix pour notifications temps réel
#. **Advanced Analytics Dashboard**\ : Métriques métier (occupancy rate, expense trends, meeting attendance)
#. **Mobile App**\ : React Native ou Flutter (offline-first)
#. **Advanced Search**\ : ElasticSearch/MeiliSearch pour recherche full-text
#. **Audit Dashboard**\ : Visualisation audit logs pour SuperAdmin

**Livrables**\ :


* 6 nouvelles features majeures
* Tests performance (benchmarks Criterion)
* Documentation scalabilité
* Mobile app (MVP)

----

Recap Phase 3
^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Catégorie
     - Effort Total
   * - **Infrastructure**
     - ~15 jours
   * - **Software**
     - 30-40 jours
   * - **Total Phase 3**
     - **45-55 jours** (6-8 semaines)


----

📅 Timeline Globale
-------------------

.. code-block::

   Nov 2025          Fév 2026          Mai 2026          Août 2026
      |                 |                 |                 |
      v                 v                 v                 v
   ┌──────────────────┐ ┌───────────────┐ ┌───────────────┐
   │   VPS MVP        │ │     K3s       │ │  K8s Prod     │
   │  (9-13 sem.)     │ │  (6-8 sem.)   │ │  (6-8 sem.)   │
   └──────────────────┘ └───────────────┘ └───────────────┘
    Security, GDPR,     Voting, PDF,      Performance,
    Storage, Backups,   Community,        Real-time,
    Monitoring,         Contractor        Analytics,
    Board Tools         Backoffice        Mobile App

Dates Clés
^^^^^^^^^^


* **Novembre 2025**\ : Début Phase 1 (VPS MVP)
* **Février 2026**\ : Fin Phase 1, début Phase 2 (K3s)
* **Mai 2026**\ : Fin Phase 2, début Phase 3 (K8s)
* **Août 2026**\ : KoproGo 1.0 Production-Ready

Effort Total
^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Phase
     - Durée
     - Fin Prévue
   * - **Phase 1 (VPS MVP)**
     - 9-13 semaines
     - Février 2026
   * - **Phase 2 (K3s)**
     - 6-8 semaines
     - Mai 2026
   * - **Phase 3 (K8s)**
     - 6-8 semaines
     - Août 2026
   * - **TOTAL**
     - **21-29 semaines**
     - **Août 2026**


----

🔗 Dépendances Critiques
------------------------

Chaînes de Dépendances
^^^^^^^^^^^^^^^^^^^^^^

.. code-block::

   #44 (Storage Strategy) ──▶ #45 (File Upload UI)
   #48 (Strong Auth)      ──▶ #46 (Voting System)
   #39-41 (Security/Backup/Monitoring) ──▶ Production VPS
   Phase 1 Complete      ──▶ Phase 2 (K3s)
   Phase 2 Complete      ──▶ Phase 3 (K8s)

Risques & Mitigations
^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Risque
     - Impact
     - Probabilité
     - Mitigation
   * - **itsme® registration delay**
     - Bloque #48 → #46
     - Moyenne
     - Démarrer registration immédiatement (Nov 2025)
   * - **Storage strategy indecision**
     - Bloque #45
     - Faible
     - Decision meeting semaine 1
   * - **K3s migration complexity**
     - Retard Phase 2
     - Moyenne
     - Tests migration sur env staging
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

**Dernière mise à jour**\ : 27 octobre 2024, 17:15
**Maintenu par**\ : KoproGo ASBL
**Contact**\ : `GitHub Issues <https://github.com/gilmry/koprogo/issues>`_
