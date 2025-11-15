===============================================
TOGAF ADM pour KoproGo ASBL
===============================================

:Auteur: KoproGo ASBL
:Date: 2025-01-15
:Version: 1.0
:Statut: Actif

.. contents:: Table des matières
   :depth: 3
   :local:

Introduction
============

Ce document adapte le **TOGAF Architecture Development Method (ADM)** au contexte spécifique de KoproGo, une ASBL belge développant une plateforme open-source de gestion de copropriété.

L'ADM fournit un cadre itératif pour développer et gouverner l'architecture d'entreprise, adapté ici pour une organisation à but non lucratif avec des contraintes écologiques, financières et communautaires strictes.

Principes directeurs KoproGo
==============================

Avant d'appliquer l'ADM, nous définissons nos principes architecturaux non négociables :

Principes techniques
--------------------

1. **Performance écologique** : < 0,12g CO₂/requête (96% réduction vs marché)
2. **Latence P99** : < 5ms (objectif de performance)
3. **Open Source** : Licence AGPL-3.0 pour tout le code
4. **Souveraineté des données** : Hébergement Europe (OVH Cloud)
5. **RGPD natif** : Privacy by design, pas d'opt-in

Principes économiques
---------------------

1. **Viabilité financière** : Marge > 98% à tous les paliers
2. **Accessibilité** : 5€/mois/copropriété (fixe, voté en AG)
3. **Fonds de solidarité** : 20% revenus IA redistribués aux membres en difficulté
4. **Transparence** : Budgets et comptes publiés annuellement

Principes organisationnels
---------------------------

1. **Gouvernance démocratique** : 1 voix = 1 copropriété (pas de pondération capital)
2. **Communautaire** : Contribution code = droit de vote technique
3. **Progression par capacités** : Jalons basés sur métriques, pas dates fixes
4. **Documentation exhaustive** : RST, Sphinx, RFCs, ADRs

Phase Préliminaire : Cadre et Principes
========================================

Objectif
--------

Définir le cadre organisationnel et les principes de gouvernance avant d'entamer l'ADM.

Contexte organisationnel
-------------------------

**Statut juridique** : ASBL belge (Association Sans But Lucratif)

**Évolution gouvernance** :

.. list-table::
   :header-rows: 1
   :widths: 20 40 40

   * - Phase
     - Gouvernance
     - Participants
   * - **Solo** (2024-Q4)
     - Fondateur unique
     - 1 développeur
   * - **Fondateurs** (2025-Q1)
     - Conseil fondateurs
     - 3-5 fondateurs
   * - **ASBL** (2025-Q2)
     - AG + CA (3 membres)
     - 10-50 contributeurs
   * - **Coopérative** (2026+)
     - Coopérative agréée
     - 100+ membres

**Membres** :

- **Copropriétés** : Client = Membre (5€/mois = 1 voix)
- **Contributeurs** : Code accepted = Droit de vote technique
- **Partenaires** : OVH Cloud (infra), communauté open-source

Stakeholders
------------

.. list-table::
   :header-rows: 1
   :widths: 30 50 20

   * - Stakeholder
     - Intérêts
     - Influence
   * - **Copropriétés**
     - Solution simple, RGPD, pas cher
     - Haute
   * - **Syndics**
     - Efficacité, conformité légale
     - Haute
   * - **Contributeurs**
     - Apprentissage, impact, reconnaissance
     - Moyenne
   * - **RGPD/APD**
     - Conformité RGPD stricte
     - Haute
   * - **OVH Cloud**
     - Partenaire infra, uptime, SLA
     - Moyenne
   * - **Communauté Open Source**
     - Code qualité, documentation
     - Moyenne

Phase A : Vision d'Architecture
================================

Objectif
--------

Définir la vision stratégique et aligner les objectifs métier avec l'architecture technique.

Vision stratégique
------------------

**Problème adressé** :

Les copropriétés belges et européennes utilisent des solutions obsolètes (Excel, PDF, emails), coûteuses (50-200€/mois), non conformes RGPD, et écologiquement désastreuses (3g CO₂/requête).

**Solution KoproGo** :

Une plateforme SaaS open-source, écologique (0,12g CO₂/req), abordable (5€/mois), et démocratique (1 copropriété = 1 voix).

Objectifs métier (Business Goals)
----------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 30 30 30

   * - Palier
     - Objectif métier
     - Métriques clés
     - Capacités requises
   * - **1. Validation**
     - Prouver Product-Market Fit
     - 100 copropriétés, 80k€ revenus
     - CRUD, Auth, RGPD basique
   * - **2. Viabilité**
     - Atteindre rentabilité opérationnelle
     - 500 copropriétés, 400k€
     - Workflow factures, comptabilité PCMN
   * - **3. Impact**
     - Devenir référence Belgique
     - 1.000 copropriétés, -107t CO₂/an
     - Assemblées numériques, signatures
   * - **4. Leadership**
     - Expansion Europe francophone
     - 2.000 copropriétés, 1,6M€
     - Multi-pays, multi-langues
   * - **5. Référence**
     - Standard européen de facto
     - 5.000 copropriétés, 4M€
     - Fédération, interopérabilité
   * - **6. Autonomie IA**
     - Réseau décentralisé edge computing
     - 10.000+ copropriétés, MCP grid
     - IA locale, 0 CO₂ cloud

Contraintes architecturales
----------------------------

**Techniques** :

- Stack Rust + Actix-web (backend), Astro + Svelte (frontend)
- PostgreSQL 15 (relationnel), MongoDB future (documents)
- Hébergement OVH Cloud (souveraineté EU)

**Écologiques** :

- < 0,12g CO₂/requête (objectif validé : 0,12g @ 287 req/s)
- Infrastructure minimale (VPS → K3s → K8s progressif)

**Réglementaires** :

- RGPD natif (privacy by design)
- Comptabilité PCMN belge (AR 12/07/2012)
- Archivage légal 10 ans (PV, factures)

**Financières** :

- Marge > 98% (5€/mois copropriété, coût infra 0,08€)
- Fonds solidarité 20% revenus IA
- Budget transparent publié annuellement

Phase B : Architecture Métier (Business Architecture)
======================================================

Objectif
--------

Modéliser les processus métier, rôles, et flux de valeur de la gestion de copropriété.

Processus métier principaux
----------------------------

1. **Gestion financière**

   - Saisie factures (syndic)
   - Workflow approbation (Draft → PendingApproval → Approved/Rejected)
   - Répartition charges (quote-part copropriétaires)
   - Comptabilité PCMN (Plan Comptable Minimum Normalisé)
   - Relances automatisées (4 niveaux : Gentle → Formal → FinalNotice → LegalAction)

2. **Assemblées générales**

   - Convocation (email automatisé, délai 15 jours)
   - Vote électronique (1 copropriété = 1 voix, quorum validé)
   - Génération PV (template, signatures numériques)
   - Archivage légal (10 ans, RGPD-compliant)

3. **Gestion documentaire**

   - Upload/stockage (MinIO S3, chiffrement AES-256)
   - Catégorisation (factures, PV, règlements, plans)
   - Recherche full-text (ElasticSearch future)
   - Partage sécurisé (expiration liens, audit trail)

4. **Communication**

   - Emails transactionnels (SendGrid)
   - Notifications push (PWA)
   - Chat temps réel (WebSocket future)
   - Fil d'actualité (incidents, travaux)

Rôles métier
------------

.. list-table::
   :header-rows: 1
   :widths: 25 50 25

   * - Rôle
     - Responsabilités
     - Permissions critiques
   * - **Syndic**
     - Gestion quotidienne, factures, AG
     - Create/Update factures, convocations
   * - **Copropriétaire**
     - Consultation, vote, paiement
     - Read factures, Vote AG, Update profil
   * - **Conseil Syndical**
     - Validation factures > seuil, audit
     - Approve/Reject factures, Read comptabilité
   * - **Comptable**
     - Comptabilité, rapports financiers
     - Full access comptabilité, Export bilans
   * - **Admin ASBL**
     - Config plateforme, support
     - Full access (God mode)

Flux de valeur
--------------

**Pour copropriétés** :

1. Abonnement 5€/mois → Accès plateforme
2. Gain temps 60% (vs Excel/email)
3. Conformité RGPD automatique
4. Réduction litiges 40% (transparence)

**Pour syndics** :

1. Outil professionnel gratuit (inclus dans forfait copropriété)
2. Workflow factures automatisé
3. Génération rapports 1-clic
4. Réduction charge administrative 50%

**Pour ASBL** :

1. Revenus récurrents (5€ × nb copropriétés)
2. Revenus IA (20% → fonds solidarité, 80% → développement)
3. Impact écologique mesurable (-840t CO₂/an @ palier 5)
4. Communauté contributeurs engagée

Phase C : Architecture Systèmes d'Information
==============================================

Objectif
--------

Définir l'architecture applicative, les intégrations, et le flux de données.

Architecture applicative
-------------------------

**Style** : Hexagonal (Ports & Adapters) + Domain-Driven Design (DDD)

**Couches** :

.. code-block:: text

   ┌─────────────────────────────────────────┐
   │  Infrastructure (Adapters)              │
   │  - Web (Actix-web handlers)             │
   │  - Database (PostgreSQL repositories)   │
   │  - External (SendGrid, MinIO, S3)       │
   └─────────────────┬───────────────────────┘
                     │ implémente
   ┌─────────────────▼───────────────────────┐
   │  Application (Use Cases + Ports)        │
   │  - BuildingUseCases, ExpenseUseCases    │
   │  - Ports (traits): BuildingRepository   │
   └─────────────────┬───────────────────────┘
                     │ dépend de
   ┌─────────────────▼───────────────────────┐
   │  Domain (Entities + Services)           │
   │  - Building, Expense, Owner, Meeting    │
   │  - Invariants métier (validations)      │
   └─────────────────────────────────────────┘

**Modules applicatifs** :

1. **Core** : Gestion immobilière (Buildings, Units, Owners)
2. **Finance** : Factures, comptabilité PCMN, relances
3. **Governance** : AG, votes, PV
4. **Documents** : Storage, indexation, partage
5. **IAM** : Auth, rôles, permissions
6. **Notifications** : Emails, push, webhooks

Intégrations externes
---------------------

.. list-table::
   :header-rows: 1
   :widths: 25 40 35

   * - Service
     - Usage
     - SLA cible
   * - **OVH Cloud**
     - Hébergement VPS/K8s
     - 99.9% uptime
   * - **SendGrid**
     - Emails transactionnels
     - 99.9% delivery
   * - **MinIO/S3**
     - Stockage documents
     - 99.99% durability
   * - **Stripe**
     - Paiements (futur)
     - 99.99% uptime
   * - **Isabel/Bancontact**
     - Paiements Belgique (futur)
     - 99.5% uptime

Flux de données critiques
--------------------------

**Données sensibles RGPD** :

- **Copropriétaires** : Nom, email, adresse, téléphone, IBAN
- **Factures** : Montants, dates paiement, historique
- **PV assemblées** : Votes nominatifs, positions exprimées

**Mesures protection** :

1. Chiffrement at-rest (LUKS AES-XTS-512)
2. Chiffrement in-transit (TLS 1.3)
3. Anonymisation logs (PII removed)
4. Backup chiffrés GPG (clé 4096-bit RSA)
5. Droit à l'oubli automatisé (soft delete + purge 30j)

Phase D : Architecture Technologique
=====================================

Objectif
--------

Définir l'infrastructure, les technologies, et la stack technique.

Infrastructure par palier
--------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 30 30 25

   * - Palier
     - Infrastructure
     - Capacité
     - Coût mensuel
   * - **1-2**
     - VPS OVH (2 vCPU, 4GB RAM)
     - 100-500 copropriétés
     - 8€
   * - **3-4**
     - K3s (3 nodes, 6 vCPU total)
     - 500-2.000 copropriétés
     - 24€
   * - **5**
     - K8s (5 nodes, 20 vCPU)
     - 2.000-5.000 copropriétés
     - 100€
   * - **6**
     - K8s + MCP Grid (edge)
     - 5.000-10.000+
     - 200€ + edge distribué

**Évolution progressive** : Pas de sur-engineering. Upgrade uniquement quand capacité atteinte.

Stack technique validée
-----------------------

**Backend** :

- Rust 1.83 + Actix-web 4.9 (API REST)
- SQLx 0.8 (compile-time query verification)
- PostgreSQL 15 (données relationnelles)
- Redis 7 future (cache, sessions)

**Frontend** :

- Astro 4.x (SSG, Islands Architecture)
- Svelte 4.x (composants interactifs)
- TailwindCSS 3.x (styling)
- PWA (offline-first, ServiceWorker)

**Infrastructure** :

- Docker Compose (dev)
- Traefik 3.0 (reverse proxy, TLS auto)
- Prometheus + Grafana (monitoring)
- Loki (logs centralisés)
- Suricata IDS + CrowdSec WAF (sécurité)

**CI/CD** :

- GitHub Actions (tests, build, deploy)
- GitOps (Terraform + Ansible)
- Testcontainers (tests intégration)
- Playwright (tests E2E)

Performance validée (rapport 2025-01-14)
-----------------------------------------

**Benchmarks réels** :

- **Latency P50** : 364ms (1 vCPU, charge soutenue)
- **Latency P99** : 752ms (< 1s, objectif atteint)
- **Throughput** : 287 req/s soutenu (> 100 copropriétés simultanées)
- **Memory** : 128MB utilisés / 2GB disponibles (5%)
- **CO₂** : 0,12g/requête (96% réduction vs marché)

**Optimisations futures** :

1. Connection pooling PostgreSQL (P99 → 300ms)
2. Cache Redis (P99 → 100ms)
3. CDN Cloudflare (P99 → 50ms)
4. Query optimization (indexes, EXPLAIN ANALYZE)

Phase E : Opportunités et Solutions
====================================

Objectif
--------

Identifier les opportunités d'amélioration et évaluer les alternatives.

Opportunités identifiées
-------------------------

1. **Réseau MCP décentralisé (Jalon 6)**

   **Problème** : Coût cloud IA prohibitif à grande échelle

   **Opportunité** : Participants apportent compute (Raspberry Pi, vieux laptops) → Réseau edge computing → IA locale → 0 CO₂ cloud

   **Impact** : Revenus IA partagés (80% développement, 20% fonds solidarité)

2. **Comptabilité temps réel**

   **Problème** : Comptabilité actuelle = batch mensuel

   **Opportunité** : Stream processing (Apache Kafka) → Comptabilité temps réel → Dashboards live

   **Impact** : Syndics voient solde/budget instantanément

3. **IA copilote syndic**

   **Problème** : Tâches répétitives (relances, convocations)

   **Opportunité** : LLM local (llama.cpp) → Génération automatique emails/documents

   **Impact** : Gain temps syndics 80%

Évaluation alternatives
-----------------------

**Décision 1 : Rust vs Go vs Node.js**

.. list-table::
   :header-rows: 1
   :widths: 20 30 25 25

   * - Critère
     - Rust ✅
     - Go
     - Node.js
   * - Performance
     - Excellent (0 overhead)
     - Bon (GC)
     - Moyen (V8)
   * - Mémoire
     - 128MB
     - 200MB
     - 300MB
   * - Écologie
     - 0,12g CO₂/req
     - 0,18g
     - 0,25g
   * - Courbe apprentissage
     - Raide
     - Douce
     - Très douce
   * - **Décision**
     - **CHOISI**
     - Rejeté
     - Rejeté

**Justification** : Performance et écologie non négociables. Courbe apprentissage compensée par documentation exhaustive.

**Décision 2 : PostgreSQL vs MongoDB vs ScyllaDB**

.. list-table::
   :header-rows: 1
   :widths: 20 30 25 25

   * - Critère
     - PostgreSQL ✅
     - MongoDB
     - ScyllaDB
   * - Transactions ACID
     - Natif
     - Limité
     - Limité
   * - Requêtes complexes
     - Excellent (SQL)
     - Moyen (aggregation)
     - Faible
   * - Scalabilité
     - Vertical (OK paliers 1-5)
     - Horizontal
     - Horizontal
   * - Conformité PCMN
     - Parfait (relationnel)
     - Difficile
     - Impossible
   * - **Décision**
     - **Paliers 1-5**
     - Future (documents)
     - Future (time-series)

**Justification** : PCMN comptabilité = relationnel obligatoire. Migration progressive vers polyglot persistence.

Phase F : Planification Migration
==================================

Objectif
--------

Planifier la migration progressive de l'infrastructure et des fonctionnalités.

Séquence migration infrastructure
----------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 30 30 25

   * - Phase
     - De → Vers
     - Trigger
     - Durée estimée
   * - **1 → 2**
     - Docker Compose → Docker Compose
     - 100 copropriétés atteintes
     - N/A (même stack)
   * - **2 → 3**
     - VPS → K3s (3 nodes)
     - 500 copropriétés, latence > 1s
     - 2 semaines
   * - **3 → 4**
     - K3s → K3s (scale)
     - 1.000 copropriétés
     - 1 semaine
   * - **4 → 5**
     - K3s → K8s
     - 2.000 copropriétés
     - 1 mois
   * - **5 → 6**
     - K8s → K8s + MCP Grid
     - 5.000 copropriétés
     - 3 mois

**Principe** : Migration zero-downtime. Blue/green deployment. Rollback plan obligatoire.

Séquence migration fonctionnelle
---------------------------------

**Palier 1 : MVP (Nov 2025 - Fév 2026)**

- CRUD complet (Buildings, Units, Owners)
- Auth basique (JWT, multi-rôles)
- Workflow factures (Draft → Approved)
- RGPD basique (droit accès, rectification, effacement)

**Palier 2 : Viabilité (Mar - Mai 2026)**

- Comptabilité PCMN complète (90 comptes)
- Relances automatisées (4 niveaux)
- Assemblées numériques (convocation, vote, PV)
- Backoffice contractant

**Palier 3 : Impact (Jun - Août 2026)**

- Signatures électroniques (eIDAS)
- Documents avancés (OCR, indexation)
- Dashboard temps réel (WebSocket)
- Mobile app (PWA → native)

**Palier 4-6** : Voir ROADMAP.md détaillé

Phase G : Gouvernance d'Implémentation
=======================================

Objectif
--------

Définir les processus de gouvernance, décision, et validation architecturale.

Processus de décision
----------------------

**RFC (Request for Comments)** :

- Propositions majeures (nouvelles features, changements archi)
- Template : ``docs/governance/rfc/template.rst``
- Workflow : Draft → Review (7j min) → Accepted/Rejected → Implemented
- Approbation : PO (ASBL) + 2 tech leads minimum

**ADR (Architecture Decision Records)** :

- Décisions techniques (choix stack, patterns)
- Template : ADR-0001 MCP Integration (exemple)
- Immutables (historique décisions)
- Numérotation séquentielle (0001, 0002, ...)

**Scrum + Nexus** :

- Sprints 2 semaines
- 4 équipes (Backend, Frontend, Infra, IA)
- Nexus Integration Team (NIT) : PO + SM + Tech Leads
- Backlog unifié GitHub Projects

Critères de validation architecture
------------------------------------

**Definition of Done (DoD) architecture** :

1. ✅ RFC approuvé (si changement majeur)
2. ✅ ADR rédigé (si décision technique)
3. ✅ Tests unitaires + intégration (> 90% coverage)
4. ✅ Documentation Sphinx mise à jour
5. ✅ Performance P99 < 5ms validée (benchmarks)
6. ✅ Impact CO₂ mesuré (< 0,12g/req)
7. ✅ Code review approuvé (2+ reviewers)
8. ✅ Déployé en staging (smoke tests OK)

**Gate reviews** :

- **Jalon 1 → 2** : 100 copropriétés, 80k€ revenus, RGPD audit passé
- **Jalon 2 → 3** : 500 copropriétés, comptabilité PCMN validée expert-comptable
- **Jalon 3 → 4** : 1.000 copropriétés, mobile app > 1000 downloads
- **Jalon 4 → 5** : 2.000 copropriétés, expansion 2+ pays
- **Jalon 5 → 6** : 5.000 copropriétés, MCP grid opérationnel

Itération continue (ADM Cycle)
===============================

L'ADM n'est pas linéaire mais **itératif**. Chaque jalon déclenche un nouveau cycle :

1. **Phase A** : Nouvelle vision (ex: expansion Europe)
2. **Phase B** : Nouveaux processus métier (ex: multi-pays)
3. **Phase C-D** : Adaptation archi (ex: i18n, multi-currency)
4. **Phase E** : Nouvelles opportunités (ex: fédération)
5. **Phase F** : Migration (ex: K3s → K8s)
6. **Phase G** : Gouvernance ajustée (ex: comités nationaux)

**Révision ADM** : Trimestrielle (synchronisée avec AG ASBL)

Alignement avec ROADMAP
========================

L'ADM complète le ROADMAP par capacités :

- **ROADMAP** : Quoi (capacités) + Quand (jalons)
- **TOGAF ADM** : Pourquoi (vision) + Comment (architecture)

**Liens directs** :

- Jalon 1-2 (MVP VPS) → Phases A-D (architecture initiale)
- Jalon 3-4 (K3s) → Phase F (migration K3s)
- Jalon 5 (K8s) → Phase E (opportunités scalabilité)
- Jalon 6 (MCP Grid) → Phase E (IA décentralisée) + ADR-0001

Voir aussi
==========

- :doc:`/ROADMAP_PAR_CAPACITES` : Roadmap détaillée par capacités
- :doc:`/governance/nexus/framework` : Scaling Scrum avec Nexus
- :doc:`/governance/scrum/ceremonies` : Cérémonies Scrum locales
- :doc:`/governance/rfc/template` : Template RFC
- :doc:`/governance/adr/0001-mcp-integration` : Exemple ADR

---

*Document maintenu par KoproGo ASBL - TOGAF adapté pour l'open-source et l'écologie*
