================================================
Migration Milestones: Phases → Jalons Capacités
================================================

:Date: 2025-11-13
:Version: 1.0
:Statut: Migration Complétée ✅

.. contents:: Table des matières
   :depth: 3
   :local:

================================================
Contexte et Objectif de la Migration
================================================

Philosophie: Pas de Dates, Des Capacités
-----------------------------------------

Le projet KoproGo adopte une approche de **progression par capacités** plutôt que par dates fixes, conformément au document :doc:`ROADMAP_PAR_CAPACITES`.

**Principe fondamental**: "KoproGo avance quand les **conditions sont remplies**, pas selon un calendrier arbitraire."

Problèmes de l'Ancienne Structure (Phases 1-4)
------------------------------------------------

❌ **Dates d'échéance fixes** (2026-03-31, 2026-06-30, etc.)
   * Créait une pression calendaire artificielle
   * Promesses potentiellement non tenues
   * Stress d'équipe et risque de burnout

❌ **Organisation par infrastructure** plutôt que par capacités
   * Phase 1: VPS MVP
   * Phase 2: K3s
   * Phase 3: K8s
   * Focus sur la technologie plutôt que sur la valeur métier

Avantages de la Nouvelle Structure (Jalons 0-7)
-------------------------------------------------

✅ **Zéro date d'échéance** (due_on: null)
   * Livraison quand c'est prêt
   * Qualité préservée
   * Équipe soutenable

✅ **Organisation par capacités mesurables**
   * Jalon 1: Sécurité & GDPR → Débloque 50-100 copros
   * Jalon 2: Conformité Légale → Débloque 200-500 copros
   * Focus sur la valeur métier et l'adoption

✅ **Conditions de déblocage claires**
   * "Jalon 1 complété quand tous les tests sécurité + GDPR passent"
   * Mesurable et vérifiable
   * Transparence totale

================================================
Mapping Détaillé: Phase → Jalon
================================================

Vue d'Ensemble
--------------

.. list-table:: Correspondance Phases → Jalons
   :header-rows: 1
   :widths: 20 20 15 45

   * - Ancien (Phase)
     - Nouveau (Jalon)
     - Issues
     - Changements Clés
   * - Phase 1 (VPS MVP)
     - Jalon 1-2
     - 23
     - Split: Sécurité (J1) + Conformité (J2)
   * - Phase 2 (K3s)
     - Jalon 3-4
     - 15
     - Split: Différenciation (J3) + Automation (J4)
   * - Phase 3 (K8s)
     - Jalon 5-6
     - 10
     - Split: Mobile/API (J5) + PropTech 2.0 (J6)
   * - Phase 4 (Ecosystem)
     - Jalon 7
     - 1
     - Platform Economy
   * - No Milestone
     - Jalon 0
     - 5
     - Fondations (100% complété)

Jalon 0: Fondations Techniques ✅ (5 issues - TOUS CLOS)
----------------------------------------------------------

**Statut**: Achevé (Automne 2025)

**Capacité débloquée**: 10-20 early adopters (beta fermée)

**Issues migrées**:

* #28 (CLOSED): Support multi-rôles utilisateurs
* #30 (CLOSED): Améliorer affichage comptes test
* #33 (CLOSED): Update docs multi-owner + Git hooks
* #57 (CLOSED): Document branch workflow
* #68 (CLOSED): Fix BDD tests super_admin

**Livrables complétés**:

* ✅ Architecture hexagonale implémentée
* ✅ 73 endpoints API REST
* ✅ Tests E2E Playwright
* ✅ Load tests validés (99.74% success, 287 req/s)
* ✅ Documentation Sphinx publiée

**Conformité légale**: 30% (features CRUD de base)

Jalon 1: Sécurité & GDPR 🔒 (11 issues - 3 clos, 8 open)
----------------------------------------------------------

**Débloque**: 50-100 copropriétés (beta publique possible)

**Conformité légale**: 40%

**Issues migrées**:

**Depuis Phase 1** (9 issues):

* #39 (OPEN): LUKS Encryption at rest
* #40 (OPEN): Encrypted backups (GPG + S3)
* #41 (OPEN): Monitoring stack (Prometheus/Grafana)
* #42 (OPEN): GDPR export & deletion (Art 15/17)
* #43 (OPEN): Security hardening (fail2ban/WAF/IDS)
* #44 (CLOSED): Document storage strategy
* #45 (CLOSED): File upload UI drag-and-drop
* #55 (OPEN): Automate MinIO/S3 bootstrap
* #66 (OPEN): E2E Admin login timeouts GDPR tests
* #69 (OPEN): Playwright E2E unit/document tests
* #78 (OPEN): Security hardening (rate limit/2FA)
* #32 (OPEN): Rewrite E2E tests for unit_owner endpoints

**⚠️ DÉPLACÉ de Phase 3 → Jalon 1**:

* #48 (OPEN): Strong auth (itsme®/eID)
* **Raison**: Auth forte est un prérequis GDPR et doit être fait AVANT les features avancées

**Livrables critiques**:

* 🔐 Données chiffrées au repos (LUKS)
* 💾 Backups quotidiens automatisés (GPG + S3)
* 📜 Conformité GDPR Articles 15 & 17
* 🔑 Authentification multi-facteur (itsme®)
* 🛡️ Security hardening production

**Conditions de déblocage**: Tous les tests sécurité + GDPR passent

**Effort estimé**: Solo dev (10-20h/sem) = 2-3 mois | Duo (40-60h/sem) = 6-8 semaines

Jalon 2: Conformité Légale Belge 📋 (11 issues - 5 clos, 6 open)
------------------------------------------------------------------

**Débloque**: 200-500 copropriétés (production ouverte)

**Conformité légale**: 80%

**Issues migrées**:

**Depuis Phase 1** (11 issues):

* #29 (OPEN): Validation quotes-parts (total = 100%)
* #51 (OPEN): Board tools (polls/tasks/issues)
* #73 (CLOSED): Invoice encoding with workflow ✅
* #75 (OPEN): Complete Meeting Management API
* #76 (OPEN): Document Upload & Download System
* #77 (CLOSED): Financial Reports Generation ✅
* #79 (CLOSED): Belgian Accounting Chart (PCMN) ✅
* #80 (OPEN): État Daté generation 🏛️ **CRITIQUE**
* #81 (OPEN): Annual Budget with variance 💰 **CRITIQUE**
* #82 (OPEN): Board of Directors (Conseil) >20 units 📋 **CRITIQUE**
* #83 (CLOSED): Payment Recovery Workflow ✅
* #44 (CLOSED): Document storage strategy ✅
* #45 (CLOSED): File upload UI ✅

**Livrables**:

* 📊 Plan Comptable Normalisé Belge (PCMN AR 12/07/2012) ✅
* 📄 Génération États Datés automatique (ventes immobilières)
* 💰 Budgets prévisionnels avec variance analysis
* 👥 Dashboard Conseil de Copropriété (obligatoire >20 lots)
* 💸 Workflow recouvrement impayés ✅

**Bloquants levés**:

* ✅ État daté → Permet ventes de lots (60% du marché belge)
* ✅ Conseil copropriété → Débloque copros >20 lots
* ✅ PCMN → Crédibilité auprès syndics professionnels

**Conditions de déblocage**: Validation experts-comptables + notaires (beta)

**Effort estimé**: Solo = 4-6 mois | Duo = 8-12 sem | Équipe = 4-6 sem

Jalon 3: Features Différenciantes 🎯 (7 issues - tous open)
-------------------------------------------------------------

**Débloque**: 500-1,000 copropriétés (différenciation marché)

**Conformité légale**: 90%

**Issues migrées**:

**Depuis Phase 2** (5 issues):

* #46 (OPEN): Meeting voting system
* #47 (OPEN): PDF generation (minutes/contracts)
* #49 (OPEN): Community features (SEL/exchange)
* #52 (OPEN): Contractor backoffice
* #84 (OPEN): Online Payment (Stripe/SEPA)

**⚠️ DÉPLACÉ de Phase 3 → Jalon 3**:

* #99 (OPEN): Community Modules (SEL, Swap Shop, Skills)
* **Raison**: SEL est explicitement mentionné dans Jalon 3 comme feature différenciante core (duplicate apparent de #49)

**Livrables**:

* 🗳️ Votes AG avec signature itsme® (PostgreSQL, non-blockchain)
* 📄 Templates PDF tous documents légaux
* 💚 Module SEL - Système Échange Local
* 📦 Bibliothèque objets partagés
* 🔧 Espace prestataires
* 💳 Paiement en ligne

**Avantage compétitif**: Features communautaires uniques (mission ASBL)

**Impact**:

* ✨ Différenciation: SEL + Partage = unique sur le marché
* 🤝 Impact social: Modules communautaires créent lien social
* 🌱 Impact écologique: 790 tonnes CO₂/an évitées
* 💰 Économie circulaire: 750k€/an échanges SEL

**Conditions de déblocage**: Jalon 2 complet + Beta utilisateurs validée

**Effort estimé**: Solo = 5-8 mois | Duo = 10-14 sem | Équipe = 5-7 sem

Jalon 4: Automation & Intégrations 📅 (11 issues - tous open)
---------------------------------------------------------------

**Débloque**: 1,000-2,000 copropriétés (scalabilité)

**Conformité légale**: 95%

**Issues migrées**:

**Depuis Phase 2** (10 issues):

* #64 (OPEN): GDPR Article 21 (direct marketing)
* #65 (OPEN): GDPR Articles 16 & 18 (rectif/restrict)
* #67 (OPEN): Final GDPR docs & QA review
* #85 (OPEN): Ticketing System (maintenance)
* #86 (OPEN): Multi-Channel Notifications 📧
* #88 (OPEN): Automatic AG Convocations 📅 **CRITIQUE**
* #89 (OPEN): Digital Maintenance Logbook
* #90 (OPEN): GDPR Complementary Articles **CRITIQUE**
* #91 (OPEN): Contractor Quotes Multi-Comparison
* #92 (OPEN): Public Syndic Information Page
* #93 (OPEN): WCAG 2.1 AA Accessibility ♿

**Depuis Phase 3** (2 issues - études):

* #71 (OPEN): Étudier Org Admin & Building Manager roles
* #72 (OPEN): Étudier RBAC granulaire dynamique

**Livrables**:

* 📧 Workflow AG 100% automatisé
* 📖 Carnet d'entretien digital
* 📜 GDPR compliance totale (Articles 16, 18, 21)
* 💼 Comparaison devis multi-entrepreneurs
* 🌐 Page publique syndic (SEO)
* ♿ Accessibilité WCAG 2.1 AA

**Impact**:

* ⚡ Automation: Temps syndic réduit de 50%
* ♿ Accessibilité: Conformité EU Accessibility Act 2025
* 🔍 SEO: Discovery organique

**Conditions de déblocage**: Base utilisateurs stable (>500 copros) pour feedback

**Effort estimé**: Solo = 6-10 mois | Duo = 12-16 sem | Équipe = 6-8 sem

Jalon 5: Mobile & API Publique 📱 (3 issues - tous open)
----------------------------------------------------------

**Débloque**: 2,000-5,000 copropriétés (expansion)

**Conformité légale**: 100%

**Issues migrées**:

**Depuis Phase 2-3**:

* #87 (OPEN): Progressive Web App (PWA)
* #98 (OPEN): Native Mobile App (iOS/Android)
* #97 (OPEN): Business Intelligence Dashboard 📊

**Livrables**:

* 📱 Progressive Web App installable
* 📲 Native Mobile App iOS/Android
* 🔌 API publique v1 documentée
* 🌍 Multi-langue NL/FR/DE/EN
* 🧾 Intégrations comptables (Winbooks, Exact)
* 📊 Analytics & Dashboards KPIs

**Impact**:

* 🌐 Écosystème: API publique → développeurs tiers
* 💼 Intégrations: Winbooks/Exact → syndics professionnels
* 📱 Mobile: PWA → adoption copropriétaires
* 🇪🇺 International: Multi-langue → expansion EU

**Prérequis**: Équipe structurée (+Mobile dev +API architect)

**Effort estimé**: Équipe = 14-18 sem | Communauté active = 6-8 sem

Jalon 6: Intelligence & Expansion (PropTech 2.0) 🤖 (5 issues - tous open)
----------------------------------------------------------------------------

**Débloque**: 5,000-10,000 copropriétés (leadership)

⚠️ **PropTech 2.0 Zone**: Modules avancés nécessitant **maturité technique + équipe 3-4 ETP minimum**

**Issues migrées**:

**Depuis Phase 3** (5 issues):

* #94 (OPEN): AI Features (OCR/Predictions/Chatbot) 🤖
* #95 (OPEN): Service Provider Marketplace
* #96 (OPEN): Sustainability & Ecology Tracking 🌱
* #109 (OPEN): IoT Integration Platform 🌐
* #110 (OPEN): Energy Buying Groups Platform ⚡

**Livrables**:

* 🤖 IA Assistant Syndic (GPT-4/Claude) [+2€/mois]
* 🏦 API Bancaire PSD2 (réconciliation auto)
* 📡 IoT Sensors (MQTT + TimescaleDB) [+1€/mois]
* 🏪 Marketplace Services Locaux
* 📈 Prédictions budgétaires (ML/ARIMA)
* ⚡ Groupements d'achat énergie

**Prérequis CRITIQUES**:

* ✅ Base utilisateurs stable (>2,000 copros)
* ✅ Revenus >10,000€/mois pour R&D
* ✅ Équipe: +Data scientist, +IoT engineer, +FinTech expert
* ✅ Budget infrastructure IoT

⚠️ **NE PAS démarrer avant Jalon 5 complet + revenus >10k€/mois**

**Effort estimé**: Équipe élargie (3-4 ETP) = 18-24 sem

Jalon 7: Platform Economy (PropTech 2.0) 🚀 (1 issue - open)
--------------------------------------------------------------

**Débloque**: 10,000+ copropriétés (scale planétaire)

⚠️ **PropTech 2.0 Expérimental**: Features blockchain nécessitant **équipe 10-15 ETP + audits externes**

**Issues migrées**:

**Depuis Phase 4**:

* #111 (OPEN): Public API v2 + SDK Multi-langages + Marketplace

**Livrables**:

* 📚 SDK multi-langages (Python, JS, PHP, Ruby)
* 🏪 Store modules tiers (marketplace plugins)
* ⛓️ Blockchain Voting (Polygon, audit Trail of Bits)
* 🌱 Carbon Credits Trading (ERC-20)
* 🏢 White-label pour fédérations
* 🇪🇺 Interopérabilité EU (standards CEN)

**Impact**:

* 🌐 Écosystème complet: +20-50 modules/an par devs tiers
* 🔐 Blockchain Immutabilité: Votes AG auditables
* 💚 Carbon Economy: Trading 840 tonnes CO₂/an
* 🇪🇺 Expansion EU: France, Espagne, Italie

**Prérequis CRITIQUES**:

* ✅ Organisation mature (10-15 ETP)
* ✅ Revenus >50,000€/mois
* ✅ Équipe blockchain: +Blockchain dev, +Auditor, +Legal
* ✅ Budget audits externes (50-100k€/audit)
* ✅ Agrément trading carbone (FSMA, AMF)

⚠️ **NE démarrer que si surplus ASBL > 100k€/an**

**Effort estimé**: Organisation mature (10-15 ETP) = 24-36 sem

================================================
Réassignations Critiques
================================================

Deux issues ont été déplacés de Phase 3 vers des jalons plus précoces:

Issue #48: itsme® Auth (Phase 3 → Jalon 1) ⚠️
-----------------------------------------------

**Ancien**: Phase 3: K8s Production
**Nouveau**: Jalon 1: Sécurité & GDPR

**Raison**:

L'authentification forte via itsme® est un **prérequis fondamental** pour:

1. **Conformité GDPR**: Identification certaine des utilisateurs
2. **Votes AG sécurisés**: Signature électronique légale
3. **Sécurité générale**: Base de toute l'application

**Conséquence**: Ne peut pas être repoussé en Phase 3 (K8s). Doit être fait dès Jalon 1 pour débloquer la beta publique.

**Priorité**: CRITIQUE - Bloque adoption 50-100 copros

Issue #99: Community Modules (Phase 3 → Jalon 3) ⚠️
-----------------------------------------------------

**Ancien**: Phase 3: K8s Production
**Nouveau**: Jalon 3: Features Différenciantes

**Raison**:

Le module SEL (Système Échange Local) est explicitement mentionné dans ROADMAP_PAR_CAPACITES.rst comme une **feature différenciante core** de Jalon 3.

**Conséquence**:

* Fait partie de l'avantage compétitif "SEL + Partage = unique sur le marché"
* Contribue à l'impact social et écologique (790 tonnes CO₂/an évitées)
* Économie circulaire: 750k€/an échanges SEL

**Note**: Semble être un duplicate de #49 (Community features). À vérifier et potentiellement merger.

**Priorité**: HAUTE - Différenciation marché

================================================
Statistiques de Migration
================================================

État Avant Migration
--------------------

.. list-table:: Milestones Phase (Ancienne Structure)
   :header-rows: 1
   :widths: 40 15 15 30

   * - Milestone
     - Issues
     - Due Date
     - % Total
   * - Phase 1: VPS MVP + Legal Compliance
     - 23
     - 2026-03-31
     - 43%
   * - Phase 2: K3s + Automation
     - 15
     - 2026-06-30
     - 28%
   * - Phase 3: K8s Production
     - 10
     - 2026-09-30
     - 19%
   * - Phase 4: Ecosystem & Scale
     - 1
     - 2027-03-31
     - 2%
   * - No Milestone
     - 5
     - N/A
     - 9%
   * - **TOTAL**
     - **54**
     -
     - **100%**

État Après Migration
---------------------

.. list-table:: Milestones Jalon (Nouvelle Structure)
   :header-rows: 1
   :widths: 40 15 15 30

   * - Milestone
     - Issues
     - Due Date
     - % Total
   * - Jalon 0: Fondations Techniques ✅
     - 5
     - **null** ✅
     - 9% (100% complété)
   * - Jalon 1: Sécurité & GDPR 🔒
     - 12
     - **null** ✅
     - 22% (8 open)
   * - Jalon 2: Conformité Légale Belge 📋
     - 13
     - **null** ✅
     - 24% (6 open)
   * - Jalon 3: Features Différenciantes 🎯
     - 7
     - **null** ✅
     - 13% (7 open)
   * - Jalon 4: Automation & Intégrations 📅
     - 13
     - **null** ✅
     - 24% (13 open)
   * - Jalon 5: Mobile & API Publique 📱
     - 3
     - **null** ✅
     - 6% (3 open)
   * - Jalon 6: Intelligence & Expansion 🤖
     - 5
     - **null** ✅
     - 9% (5 open)
   * - Jalon 7: Platform Economy 🚀
     - 1
     - **null** ✅
     - 2% (1 open)
   * - **TOTAL**
     - **59**
     -
     - **109%** (5 issues assignés à 2 jalons)

**Note**: Le total est > 54 car certains issues complétés (Jalon 0-2) étaient également référencés dans les anciennes Phases.

Distribution Open/Closed
-------------------------

.. list-table:: Statut Issues par Jalon
   :header-rows: 1
   :widths: 40 15 15 30

   * - Jalon
     - Open
     - Closed
     - % Complété
   * - Jalon 0: Fondations ✅
     - 0
     - 5
     - **100%**
   * - Jalon 1: Sécurité 🔒
     - 9
     - 3
     - 25%
   * - Jalon 2: Conformité 📋
     - 7
     - 6
     - 46%
   * - Jalon 3: Différenciation 🎯
     - 7
     - 0
     - 0%
   * - Jalon 4: Automation 📅
     - 13
     - 0
     - 0%
   * - Jalon 5: Mobile 📱
     - 3
     - 0
     - 0%
   * - Jalon 6: Intelligence 🤖
     - 5
     - 0
     - 0%
   * - Jalon 7: Platform 🚀
     - 1
     - 0
     - 0%

**Prochain Objectif**: Compléter Jalon 1 (9 issues open) pour débloquer beta publique 50-100 copros

================================================
Changements Organisationnels
================================================

GitHub Milestones
-----------------

**Supprimés** (2025-11-13):

* Phase 1: VPS MVP + Legal Compliance (milestone #1)
* Phase 2: K3s + Automation (milestone #2)
* Phase 3: K8s Production (milestone #3)
* Phase 4: Ecosystem & Scale (milestone #4)

**Créés** (2025-11-13):

* Jalon 0: Fondations Techniques ✅ (milestone #5)
* Jalon 1: Sécurité & GDPR 🔒 (milestone #6)
* Jalon 2: Conformité Légale Belge 📋 (milestone #7)
* Jalon 3: Features Différenciantes 🎯 (milestone #8)
* Jalon 4: Automation & Intégrations 📅 (milestone #9)
* Jalon 5: Mobile & API Publique 📱 (milestone #10)
* Jalon 6: Intelligence & Expansion (PropTech 2.0) 🤖 (milestone #11)
* Jalon 7: Platform Economy (PropTech 2.0) 🚀 (milestone #12)

**Caractéristique clé**: Tous les nouveaux milestones ont ``due_on: null`` ✅

GitHub Projects
---------------

Les 2 GitHub Projects existants reflètent automatiquement les nouveaux milestones:

* **Project #2**: KoproGo - Software Roadmap (56 items)
* **Project #3**: KoproGo - Infrastructure Roadmap (38 items)

**Note**: Les issues restent dans leurs projets respectifs, seuls les milestones ont changé.

================================================
Impact et Bénéfices
================================================

Bénéfices Immédiats
-------------------

✅ **Conformité philosophique**

* Alignement total avec ROADMAP_PAR_CAPACITES.rst
* "Pas de Dates, Des Capacités" respecté
* Conditions de déblocage mesurables

✅ **Meilleure organisation technique**

* Séparation claire: Sécurité (J1) → Légal (J2) → Features (J3)
* Dépendances explicites (J2 nécessite J1 complet)
* Priorisation basée sur la valeur métier

✅ **Transparence accrue**

* Conditions de déblocage claires pour chaque jalon
* Prérequis explicites (équipe, budget, infrastructure)
* Jalons 6-7: Avertissements PropTech 2.0 (ne pas démarrer trop tôt)

Métriques Avant/Après
----------------------

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Métrique
     - Avant (Phases)
     - Après (Jalons)
   * - Milestones avec dates fixes
     - 4 (100%)
     - 0 (0%) ✅
   * - Milestones sans dates
     - 0 (0%)
     - 8 (100%) ✅
   * - Pression calendaire
     - Élevée 📅
     - Nulle ✅
   * - Focus organisation
     - Infrastructure (VPS/K3s/K8s)
     - Capacités métier ✅
   * - Conditions déblocage
     - Dates arbitraires
     - Conditions mesurables ✅
   * - Jalons complétés
     - 0 (0%)
     - 1 (Jalon 0 - 100%) ✅

Risques Atténués
-----------------

❌ → ✅ **Stress d'équipe**

* Avant: Pression dates fixes (Mars 2026, Juin 2026, etc.)
* Après: Livraison quand c'est prêt

❌ → ✅ **Qualité compromise**

* Avant: Risque de rush pour respecter deadlines
* Après: Tests exhaustifs avant déblocage

❌ → ✅ **Promesses non tenues**

* Avant: "Livré en Mars 2026" = promesse risquée
* Après: "Livré quand sécurité validée" = engagement réaliste

❌ → ✅ **Mauvaise priorisation**

* Avant: #48 (itsme® auth) en Phase 3 = trop tard
* Après: #48 en Jalon 1 = au bon moment

================================================
Prochaines Étapes
================================================

Priorité Immédiate: Compléter Jalon 1
---------------------------------------

**Objectif**: Débloquer 50-100 copropriétés (beta publique)

**9 issues open critiques**:

1. #39: LUKS Encryption at rest
2. #40: Encrypted backups (GPG + S3)
3. #41: Monitoring stack (Prometheus/Grafana)
4. #42: GDPR export & deletion (Art 15/17)
5. #43: Security hardening (fail2ban/WAF/IDS)
6. #48: itsme® strong authentication
7. #55: Automate MinIO/S3 bootstrap
8. #66: E2E Admin login timeouts GDPR tests
9. #69: Playwright E2E unit/document tests

**Conditions de déblocage**: Tous les tests sécurité + GDPR passent

**Effort estimé**: Solo (20h/sem) = 4-6 semaines restantes

Ensuite: Attaquer Jalon 2
--------------------------

**⚠️ NE commence QUE quand Jalon 1 est complet** (données sécurisées)

**7 issues open critiques**:

1. #80: État Daté generation 🏛️ **BLOQUE ventes immobilières**
2. #82: Board of Directors >20 units 📋 **BLOQUE 60% du marché**
3. #81: Annual Budget **OBLIGATION LÉGALE**
4. #75: Complete Meeting Management API
5. #76: Document Upload & Download System
6. #29: Validation quotes-parts (total = 100%)
7. #51: Board tools (polls/tasks/issues)

**Conditions de déblocage**: Validation experts-comptables + notaires (beta)

**Effort estimé**: Solo = 4-6 mois | Duo = 8-12 sem | Équipe = 4-6 sem

Mise à Jour Documentation
--------------------------

Documents à mettre à jour pour référencer "Jalons" au lieu de "Phases":

* ✅ :doc:`JALONS_MIGRATION` (ce document)
* ⏳ README.md
* ⏳ CLAUDE.md
* ⏳ docs/ROADMAP.md (si existe)
* ⏳ Autres références dans docs/*.rst

================================================
Références
================================================

* :doc:`ROADMAP_PAR_CAPACITES` - Philosophie "Pas de Dates, Des Capacités"
* :doc:`VISION` - Vision macro et problème sociétal
* :doc:`MISSION` - Mission holistique et valeurs
* :doc:`ECONOMIC_MODEL` - Viabilité économique
* `GitHub Milestones <https://github.com/gilmry/koprogo/milestones>`_ - Nouveaux Jalons 0-7
* `GitHub Project #2 <https://github.com/users/gilmry/projects/2>`_ - Software Roadmap
* `GitHub Project #3 <https://github.com/users/gilmry/projects/3>`_ - Infrastructure Roadmap

---

*Document de Migration Milestones KoproGo v1.0*

*"Nous livrons quand c'est prêt, pas quand le calendrier le dit."*

*Contact: contact@koprogo.com - GitHub: github.com/gilmry/koprogo*
