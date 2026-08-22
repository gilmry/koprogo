============================
Jalons Atteints
============================

:Date: 2025-01-19
:État: Ce qui est déjà fait

.. contents:: Table des matières
   :depth: 2
   :local:

Introduction
============

Cette page présente les **jalons déjà atteints** par KoproGo. Elle démontre la **maturité technique** du projet et valide la **faisabilité** de la vision 2025-2030.

.. tip::
   **Le projet est déjà opérationnel !**

   KoproGo n'est pas un concept : c'est une plateforme fonctionnelle avec 73 endpoints API, des tests automatisés, et une infrastructure en production.

Jalon 0 : Fondations Techniques ✅
===================================

**État** : **COMPLÉTÉ** (Automne 2025)

**Capacité débloquée** : 10-20 early adopters (beta fermée)

**Conformité légale** : 30% (features CRUD de base)

Architecture & Code
-------------------

✅ **Architecture Hexagonale Implémentée**

* Domain layer (entités avec invariants)
* Application layer (use cases + ports)
* Infrastructure layer (adapters web, database)
* Séparation stricte des couches
* Dependency inversion respectée

✅ **73 Endpoints REST API Opérationnels**

* Buildings : 6 endpoints (CRUD + stats)
* Units : 8 endpoints (CRUD + owners management)
* Owners : 7 endpoints (CRUD + units relationship + history)
* Expenses : 12 endpoints (CRUD + workflow approval + payment reminders)
* Meetings : 6 endpoints (CRUD + attendees)
* Documents : 6 endpoints (CRUD + upload)
* Auth : 5 endpoints (login, register, refresh, logout, switch-role)
* Accounts (PCMN) : 7 endpoints
* Payment Reminders : 8 endpoints
* Unit Owners : 8 endpoints (multi-owner support)

✅ **11 Entités du Domaine**

1. Organization
2. User (avec multi-role support)
3. Building
4. Unit
5. Owner
6. UnitOwner (relation N-N avec pourcentages)
7. Expense (avec workflow approval)
8. InvoiceLineItem (lignes de facturation + TVA)
9. Meeting
10. Document
11. Account (PCMN belge)
12. PaymentReminder (workflow recouvrement)

Tests & Qualité
---------------

✅ **Tests E2E Automatisés (Playwright)**

* 45+ scénarios E2E
* Backend + Frontend intégration
* Visual regression testing
* Tests cross-browser

✅ **Tests Unitaires & Intégration**

* Domain : 100% couverture (tests in-module)
* Application : Use cases testés
* Infrastructure : Tests intégration PostgreSQL (testcontainers)
* BDD : Scénarios Cucumber/Gherkin

✅ **Load Tests Validés**

* **287 req/s** soutenus (charge réelle)
* **752ms** latence P99 (1 vCPU)
* **99,74% success rate** (26 échecs sur 17 200 requêtes)
* **128MB RAM** utilisée (sur 2GB disponibles)

Performance & Écologie
----------------------

✅ **Performance Validée en Production**

* Throughput : 287 req/s soutenu
* Latency P99 : 752ms (1 vCPU VPS)
* Memory : 128MB utilisée
* CPU : < 20% utilisation moyenne
* Uptime : 99,74%

✅ **Empreinte Carbone Mesurée**

* **0,12g CO₂/requête** (mesuré en production)
* vs 11,5g concurrence (96% réduction)
* Datacenter OVH Gravelines (60g CO₂/kWh)
* Architecture Rust optimisée

Infrastructure & Sécurité
--------------------------

✅ **Infrastructure Production (Issue #39, #40, #41, #43)**

* VPS OVH s1-2 (1 vCPU, 2GB RAM) - 6,30€/mois
* Docker + GitOps (auto-deploy)
* LUKS Encryption at-rest (AES-XTS-512)
* Backups quotidiens GPG + S3 off-site
* Monitoring : Prometheus + Grafana + Loki

✅ **Sécurité Renforcée**

* TLS 1.3 (Let's Encrypt)
* JWT authentication + refresh tokens
* Hashing passwords (Argon2id)
* Firewall UFW (ports 22, 80, 443)
* Fail2ban (SSH, Traefik, API abuse)
* Suricata IDS (détection intrusions)
* CrowdSec WAF (threat intelligence)

Documentation
-------------

✅ **Documentation Sphinx Complète**

* 50+ documents RST
* Architecture détaillée (backend, frontend, infra)
* Guides utilisateurs (syndic, owner, comptable)
* ADR (Architecture Decision Records)
* RFC (Request For Comments)
* API documentation
* Tutoriels vidéo YouTube

✅ **Workflows Git Documentés**

* Pre-commit hooks (format + lint)
* Pre-push hooks (lint + test)
* Conventional commits
* DCO (Developer Certificate of Origin)
* Pull request templates

Jalon 1 : Sécurité & GDPR (Complété) ✅
========================================

**État** : **100% COMPLÉTÉ (Mai 2026)**

**Capacité débloquée** : 50-100 copropriétés (bêta privée fermée, VPS-first)

**Conformité légale** : 95%

Déjà Fait
---------

✅ **Infrastructure Sécurisée** (Issues #39, #40, #41, #43)

* LUKS Encryption at-rest ✅
* Backups GPG + S3 ✅
* Monitoring/Alerting (Prometheus, Grafana, Loki, Alertmanager) ✅
* Intrusion Detection (Suricata) ✅
* WAF (CrowdSec) ✅
* fail2ban + SSH hardening ✅

✅ **GDPR Complet** (Issues #42, #90, #64, #65, #67)

* Export données utilisateur (Article 15 GDPR) ✅
* Droit à l'oubli (Article 17 GDPR) ✅
* Articles 16, 18, 21 (rectification, restriction, opposition) ✅
* Privacy policy + documentation GDPR ✅
* Tests GDPR automatisés ✅

✅ **Sécurité Auth & Comptable** (Mai 2026 — WP-A1/A4..A7, FE1)

* #271 / #272 / #273 : Quorum AG (Art. 3.87 §5 CC), 2e convocation, vote mandataire ✅
* #525 (WP-A1 Story C1) : Governance Decimal (quotas/voting power exacts) ✅
* #433 umbrella : Decimal end-to-end (EXP-005/006/007/008) + erreurs domaine typées ✅
* #343 (WP-FE1) : JWT hors localStorage — refresh = cookie ``HttpOnly``, access en mémoire, silent-refresh single-flight (#550) ✅
* #339 (WP-A7) : Rotation clés API (gate SYNDIC/SUPERADMIN, anti-rejeu, secret une fois) ✅
* #526 : ``expenses.amount > 0`` conservé, annulations = contre-écritures journal PCMN ✅
* ADR-0008 amendement 2026-05-19 : carve-outs ``f64`` fermés (résolution %-affichage / ratio proxy vote / gamification / surfaces m² / IoT) ✅

⏳ **Différé v0.2.0** (non bloquant bêta fermée)

* #48 : itsme® / eID pour votes AG (Phase 2 — auth forte)
* Cascade port/use-case/handler ``Result<_, String>`` → ``AppError`` (pont conservé)

Modules Fonctionnels Déjà Implémentés
======================================

Multi-Owner Support ✅
----------------------

**Issue** : Implémenté (multi-tenancy refactor)

* Junction table `unit_owners` (N-N)
* Pourcentages de propriété (validation ≤ 100%)
* Historique temporel (date début/fin)
* Contact principal unique
* Transfert de propriété
* API endpoints : 8 endpoints

**Frontend** : Components Svelte (UnitOwners.svelte, OwnerList.svelte, modals)

**Documentation** : :doc:`../MULTI_OWNER_SUPPORT`

Multi-Role Support ✅
---------------------

**Issue** : Implémenté

* Table `user_roles` (user_id, organization_id, role)
* Rôles : syndic, copropriétaire, comptable, administrateur
* Rôle actif sélectionnable
* Middleware AuthenticatedUser expose `role_id`
* Endpoints : `/auth/login`, `/auth/switch-role`, `/auth/me`

**Frontend** : Sélecteur multi-rôle (Navigation.svelte)

**Tests** : E2E scénario multi-rôles + BDD

**Documentation** : :doc:`../MULTI_ROLE_SUPPORT`

Comptabilité Belge (PCMN) ✅
-----------------------------

**Issue** : #79 (COMPLÉTÉ)

* Plan Comptable Minimum Normalisé (AR 12/07/2012)
* ~90 comptes pré-seedés (8 classes)
* Hiérarchie complète (classes, sous-classes, groupes)
* Validation codes comptables
* Domain entity : `Account`
* Use cases : Create, Read, Update, Delete, Seed
* Repository PostgreSQL
* API endpoints : 7 endpoints
* Financial reports : Bilan & Compte de résultats

**Tests** : 100% couverture domain + integration PostgreSQL

**Documentation** : :doc:`../BELGIAN_ACCOUNTING_PCMN`

Invoice Workflow ✅
-------------------

**Issue** : #73 (COMPLÉTÉ)

* Workflow : Draft → PendingApproval → Approved/Rejected
* Gestion TVA belge (6%, 12%, 21%)
* Multi-lignes (InvoiceLineItem)
* Validation métier (empêche modification après approbation)
* Domain entities : `Expense`, `InvoiceLineItem`
* Endpoints : `/expenses/:id/submit-for-approval`, `/approve`, `/reject`

**Tests** : Scénarios BDD + E2E workflow

**Documentation** : :doc:`../INVOICE_WORKFLOW`

Payment Recovery Workflow ✅
----------------------------

**Issue** : #83 (COMPLÉTÉ)

* Workflow automatisé de recouvrement
* 4 niveaux : Gentle (J+15) → Formal (J+30) → FinalNotice (J+45) → LegalAction (J+60)
* Calcul pénalités retard (taux légal belge 8% annuel)
* Traçabilité (sent_date, tracking_number, notes)
* Domain entity : `PaymentReminder`
* Use cases : Create, Escalate, MarkSent, Stats
* Endpoints : 8 endpoints

**Tests** : Scénarios d'escalade + calcul pénalités

**Documentation** : :doc:`../PAYMENT_RECOVERY_WORKFLOW`

Statistiques Globales (État Actuel)
====================================

**Architecture & Code**

* **73 endpoints REST API** ✅
* **11 entités du domaine** ✅
* **Architecture hexagonale** ✅
* **100% open-source** (AGPL-3.0) ✅

**Tests & Qualité**

* **45+ scénarios E2E** (Playwright) ✅
* **100% couverture domain** ✅
* **Load tests** (287 req/s, 99,74% uptime) ✅
* **Benchmarks** (Criterion) ✅

**Performance**

* **287 req/s** soutenus ✅
* **752ms** latence P99 ✅
* **0,12g CO₂/requête** ✅
* **128MB RAM** utilisée ✅

**Infrastructure**

* **VPS production** (OVH GRA11) ✅
* **LUKS encryption** ✅
* **Backups GPG + S3** ✅
* **Monitoring/Alerting** ✅
* **IDS/WAF** (Suricata + CrowdSec) ✅

**Documentation**

* **50+ documents RST** ✅
* **Sphinx documentation** ✅
* **Tutoriels YouTube** ✅
* **RFC/ADR** ✅

Conclusion
==========

**KoproGo est déjà opérationnel et prêt pour les early adopters.**

Les fondations techniques sont **solides**, l'architecture est **mature**, et la performance est **validée en production**.

**Prochaine étape** : Compléter Jalon 1 (GDPR + Auth forte) pour débloquer la **beta publique** (50-100 copropriétés).

----

**Voir Aussi**

* :doc:`jalons-a-venir` - Ce qui vient ensuite
* :doc:`roadmap-2025-2030` - Roadmap complète 2025-2030
* :doc:`../PERFORMANCE_REPORT` - Rapport de performance détaillé

----

*Jalons Atteints - Documentation KoproGo ASBL*

*Dernière mise à jour : 2025-01-19*
