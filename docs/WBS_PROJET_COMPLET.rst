=========================================================
KoproGo - WBS Projet Complet (DDD-BDD-TDD-SOLID)
=========================================================

:Version: 2.1
:Date: 25 mars 2026
:Methode: Domain-Driven Design + Behavior-Driven Development + Test-Driven Development + SOLID
:Auteur: Claude Code (Audit automatise)
:Statut: Document de reference technique
:Couverture: Jalon 0 (complete) -> Jalon 7 (vision long terme)

.. note::

   **Mise a jour 2026-03-25** : Audit croise code vs GitHub issues.
   7 issues creees (#335-#341) pour stubs non implementes, bugs silencieux et RBAC manquant.
   #331 (Playwright E2E) assignee au Jalon 1.
   4 commits thematiques : unit tests 30 modules use cases (+14,580 LOC),
   fix auth headers E2E (4 fichiers), refactor i18n centralise (closes #330),
   fix misc (dto duplicates, derive Clone, test infra dual-mode).
   Milestones GitHub resynchro : J0 3/3, J1 28/31, J2 24/24, J3 24/28,
   J4 20/22, J5 2/12, J6 1/5, J7 0/1.

.. note::

   **Mise a jour 2026-03-24** : Grand merge integration -> main (48 commits, 65 conflits resolus).
   55 issues fermees (#220-237 R&D, #252-265 MCP, #271-280 Legal+Features, #300-317 Bugs+GDPR, #326-334 retroactives).
   8 issues creees retroactivement (#326-334) pour tracabilite du travail non planifie
   (consent, security incidents, API keys, GDPR Art.30, i18n, BDD tests, OpenAPI, CI fixes).
   Toutes les features Jalon 1-3 sont maintenant implementees (backend 100%, BDD 100%, E2E 100%).
   Branches nettoyees : main = dev = integration = staging = production (identiques).
   CI corrigee : formatting (cargo fmt + prettier), RUSTSEC-2026-0066, astro check, SSG build.
   4 PRs dependabot mergees (#318-321).
   Playwright : 219/240 tests passent (91%), 21 en echec (ApiKeys + SecurityIncidents).

.. note::

   **Mise a jour 2026-03-22** : Tests E2E manuels — 17 issues creees (#301-#317).
   8 bugs (2 CRITIQUES: multi-tenant #302, tantiemes #306), 1 GAP architectural (#309),
   5 conformite legale AG (#310-#314), 3 RGPD (#315-#317).
   Matrice conformite : 67% (25/37). RGPD : 60% (6/10 articles).
   docs/github-export/ resynchronise (59 -> 139 fichiers RST).

.. note::

   **Mise a jour 2026-03-15** : Reorganisation des jalons et releases.
   Schema de releases simplifie : v0.5.0/v0.6.0/v0.7.0 remplace par **v0.1.0 / v0.2.0**.
   MCP Tools (#252-265) et itsme (#48) repousses de Jalon 3 vers Jalon 4.
   K3s infra (#266-268) deplace vers Jalon 5.
   WBS obsoletes supprimes (WBS_RELEASE_0_5_0.rst, WBS_RELEASE_0_6_0.rst, WBS_2026_02_18.rst).

.. contents:: Table des matieres
   :depth: 4
   :local:

=========================================================
PARTIE I : CADRE METHODOLOGIQUE
=========================================================

1. Approche DDD (Domain-Driven Design)
----------------------------------------

1.1 Bounded Contexts
~~~~~~~~~~~~~~~~~~~~~

Le projet est decompose en **17 Bounded Contexts** (BC), chacun autonome
avec ses propres entites, ports, use cases et adaptateurs.

.. code-block:: text

   BC1  Core Infrastructure      Building, Unit, Owner, Org, Auth, UserRole
   BC2  Financial & Accounting   Expense, PCMN, Payment, Budget, ChargeDistrib
   BC3  Meeting & Governance     Meeting, Resolution, Poll, Board, Convocation
   BC4  Property Management      Ticket, Quote, WorkReport, Inspection, EtatDate
   BC5  Community Features       SEL, Notice, Skill, SharedObject, Booking, Gamification
   BC6  Notification System      Notification, NotificationPreference
   BC7  GDPR & Privacy           GdprExport, Rectification, Restriction, Objection
   BC8  Energy & IoT             EnergyCampaign, EnergyBill, IoTReading, LinkyDevice
   BC9  Security & 2FA           TwoFactorSecret, RefreshToken
   BC10 Reporting & Analytics    Dashboard, FinancialReports (cross-context)
   BC11 SaaS Administration      Organization CRUD, User CRUD (SuperAdmin)
   BC12 Revenue Management       CallForFunds, OwnerContribution
   BC13 Public & Open            PublicSyndicPage (no auth)
   BC14 Marketplace & Evaluations  ServiceProvider, ContractEvaluation (ancre L13 Art. 3.89 §5 12°)
   BC15 AG Visioconference        AgSession, quorum combine, convocation enrichie (Art. 3.87 §1)
   BC16 Contractor Backoffice PWA ContractorReport, magic link JWT, CdC validation -> paiement
   BC17 AGE Agile & Concertation   AgeRequest, petition 1/5 quotites, auto-convocation (Art. 3.87 §2 al.2)

1.2 Context Map (relations inter-BC)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   BC1 (Core) <--- SHARED KERNEL ---> BC11 (SaaS Admin)
       |                                    |
       +--- CUSTOMER/SUPPLIER ---> BC2 (Financial)
       |                               |
       +--- CUSTOMER/SUPPLIER ---> BC3 (Governance)
       |                               |
       +--- CUSTOMER/SUPPLIER ---> BC4 (Property Mgmt)
       |                               |
       +--- CUSTOMER/SUPPLIER ---> BC5 (Community)
       |
       +--- CONFORMIST ---------> BC7 (GDPR)
       |
       +--- ANTICORRUPTION -----> BC8 (Energy/IoT)
       |
       +--- PUBLISHED LANGUAGE --> BC13 (Public)

   BC6 (Notifications) <--- OPEN HOST SERVICE (consomme events de tous les BC)
   BC9 (Security)      <--- SEPARATE WAYS (module autonome, pas de deps)
   BC10 (Reporting)    <--- PUBLISHED LANGUAGE (agregation cross-context)
   BC12 (Revenue)      <--- CONFORMIST (depend de BC1 + BC2)
   BC14 (Marketplace)  <--- CUSTOMER/SUPPLIER (depend de BC4 Ticket+Quote pour auto-trigger evaluation)
   BC15 (AG Visio)     <--- CUSTOMER/SUPPLIER (depend de BC3 Meeting+Convocation)
   BC16 (Contractor)   <--- CUSTOMER/SUPPLIER (depend de BC4 Ticket+Quote + BC2 Payment)
   BC17 (AGE Agile)    <--- CUSTOMER/SUPPLIER (depend de BC3 Convocation + BC15 AG Visio)

1.3 Ubiquitous Language
~~~~~~~~~~~~~~~~~~~~~~~~~

Termes metier imposes par le domaine belge de la copropriete :

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Terme
     - Definition
   * - Copropriete
     - Immeuble en propriete partagee (= Building)
   * - Lot
     - Unite privative au sein d'un immeuble (= Unit)
   * - Tantieme/Millieme
     - Quote-part de propriete (0-1000, voting_power)
   * - Syndic
     - Gestionnaire legal de la copropriete (role)
   * - AG (Assemblee Generale)
     - Reunion annuelle obligatoire (= Meeting)
   * - Convocation
     - Invitation legale a l'AG (delai 15j ordinaire, 8j extraordinaire)
   * - Resolution
     - Decision soumise au vote en AG
   * - Majorite Simple
     - 50%+1 des votes exprimes
   * - Majorite Absolue
     - 50%+1 de tous les votes (y compris absents)
   * - Majorite Qualifiee
     - Seuil personnalise (ex: 2/3, 3/4)
   * - PCMN
     - Plan Comptable Minimum Normalise (AR 12/07/2012)
   * - Etat Date
     - Document legal obligatoire pour vente de lot
   * - SEL
     - Systeme d'Echange Local (monnaie temps)
   * - Appel de fonds
     - Demande de paiement collective aux proprietaires

2. Approche BDD (Behavior-Driven Development)
-----------------------------------------------

2.1 Pyramide des Tests - Etat et Cibles
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   Couche          Etat 02/2026  Cible v0.5  Cible v1.0  Cible v2.0
   --------------- ------------- ----------- ----------- -----------
   Unit (domain)       501          550+        600+        800+
   BDD scenarios       194          473         500+        600+
   E2E backend          30          215         250+        300+
   Playwright           27           64         120+        200+
   Benchmarks            5            5          10+         20+
   Load tests            0            0           5+         10+
   --------------- ------------- ----------- ----------- -----------
   TOTAL              ~757       ~1,307      ~1,485+     ~1,930+

2.2 Architecture BDD
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   backend/tests/
   |-- bdd.rs                   # Legacy World (24 features, 194 scenarios)
   |-- bdd_financial.rs         # BC2+BC10+BC12 (74 scenarios)
   |-- bdd_governance.rs        # BC3+BC9+BC11+BC13 (74 scenarios)
   |-- bdd_operations.rs        # BC4+BC6+BC8 (83 scenarios)
   |-- bdd_community.rs         # BC5 (72 scenarios)
   |-- features/
   |   |-- (48 fichiers .feature, 473 scenarios)
   |-- common/
       |-- mod.rs               # Setup testcontainers partage

2.3 Cycle BDD par Feature
~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   1. [SPEC]  Ecrire .feature (Given/When/Then) avec Product Owner
   2. [RED]   Executer -> scenarios PENDING (pas de step defs)
   3. [GREEN] Ecrire step definitions -> appellent use_cases
   4. [RUN]   Executer -> scenarios PASS
   5. [DOC]   Documenter la feature (RST/MD)

3. Approche TDD (Test-Driven Development)
-------------------------------------------

3.1 Cycle TDD par Composant
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   Domain Entity :
     1. RED   : Ecrire test unitaire (#[test] fn test_new_invalid_...)
     2. GREEN : Implementer la validation dans Entity::new()
     3. REFACTOR : Extraire helpers si > 3 validations similaires

   Use Case :
     1. RED   : Ecrire scenario BDD (.feature)
     2. GREEN : Implementer UseCases::method() qui orchestre repos
     3. REFACTOR : Extraire domain service si logique dupliquee

   Handler :
     1. RED   : Ecrire test E2E (HTTP request -> assert response)
     2. GREEN : Implementer handler Actix-web
     3. REFACTOR : Extraire middleware si pattern repete

   Frontend :
     1. RED   : Ecrire spec Playwright (navigate -> assert visible)
     2. GREEN : Implementer composant Svelte
     3. REFACTOR : Extraire composant si > 200 lignes

4. Verification SOLID
----------------------

Appliquee a chaque Work Package :

.. code-block:: text

   S - Single Responsibility
     - 1 entity = 1 aggregate root avec ses invariants
     - 1 use case = 1 orchestration metier
     - 1 handler = 1 endpoint HTTP
     - ALERTE si fichier > 800 LOC -> split requis

   O - Open/Closed
     - Nouveaux BC n'impactent pas les BC existants
     - Nouveaux statuts via enum extension (pas modification)
     - Traits (ports) ajoutent methodes via trait extension

   L - Liskov Substitution
     - MockAll pour tous les traits en tests
     - PostgresXxxRepository substituable par InMemoryXxxRepository
     - AppState injectable avec n'importe quelle implementation

   I - Interface Segregation
     - 1 port (trait) = 1 responsabilite (max 20 methodes)
     - Pas de "god trait" combinant plusieurs domaines
     - Handlers ne connaissent que les use_cases qu'ils utilisent

   D - Dependency Inversion
     - Domain -> 0 deps externes (pur Rust, pas de crate I/O)
     - Application -> depend de traits (ports), pas d'implementations
     - Infrastructure -> implemente ports, injecte via AppState


=========================================================
PARTIE II : ETAT DES LIEUX (BASELINE)
=========================================================

5. Progression par Jalon
--------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 10 10 10 10 35

   * - Jalon
     - Issues
     - Fermees
     - Ouvertes
     - %
     - Statut
   * - J0 Fondations Techniques
     - 3
     - 3
     - 0
     - 100%
     - COMPLETE
   * - J1 Securite & GDPR
     - 31
     - 28
     - 3
     - 90%
     - #340 RBAC gamification, #337 consent stubs, #331 Playwright E2E
   * - J2 Conformite Legale Belge
     - 24
     - 24
     - 0
     - 100%
     - COMPLETE
   * - J3 Features Differenciantes
     - 28
     - 24
     - 4
     - 86%
     - #335 marketplace stubs, #336 individual member stubs, #338 Uuid::nil bug, #341 auto-payment
   * - J4 Automation & Integrations
     - 22
     - 20
     - 2
     - 91%
     - #339 API key rotation, #48 itsme/eID
   * - J5 Mobile & API Publique
     - 12
     - 2
     - 10
     - 17%
     - Tauri (#295-299), K3s (#266-268), Mobile (#98), BI (#97)
   * - J6 Intelligence & Expansion
     - 5
     - 1
     - 4
     - 20%
     - IoT full (#109), AI (#94), Marketplace (#95), Eco (#96)
   * - J7 Platform Economy
     - 1
     - 0
     - 1
     - 0%
     - API v2 + SDK (#111)
   * -
     - **126**
     - **102**
     - **24**
     - **81%**
     -

**Issues sans jalon** : 0 (toutes assignees)

6. Metriques Code
-------------------

.. code-block:: text

   Backend Rust   : ~125,000 LOC (domain+application+infrastructure+tests) [+14,700 tests use cases]
   Frontend Svelte: ~18,000 LOC (pages+components+lib)
   Migrations SQL : ~8,500 LOC (64 migrations)
   Documentation  : ~28,000 LOC (50+ RST/MD files)
   Infrastructure : ~5,000 LOC (Ansible, Docker, CI)
   TOTAL          : ~184,500 LOC

   Bounded Contexts   : 17 (BC1-BC17)
   Entites domaine    : 57
   Endpoints API REST : 511+
   Tests unitaires    : ~900+ (777 domain + ~120 use case mocks)
   Tests BDD          : 454+ scenarios (5 fichiers bdd_*.rs, 0 failures, 0 skips)
   GitHub Issues      : 126 total (102 fermees, 24 ouvertes)

7. Dette Technique Identifiee
-------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 15 15 20

   * - #
     - Description
     - Impact
     - Effort
     - Release cible
   * - DT-1
     - BDD step definitions : RESOLU (0 failures, 0 skips sur 454 scenarios)
     - ~~HAUT~~
     - ~~81h~~
     - ✅ v0.1.0
   * - DT-2
     - E2E backend : 27/46 use cases sans test HTTP dedie
     - HAUT
     - 40h
     - v0.1.0
   * - DT-3
     - Documentation features : partiellement couverte
     - MOYEN
     - 20h
     - v0.1.0
   * - DT-4
     - Playwright couvre 26% des pages (11/43 specs actifs)
     - HAUT
     - 32h
     - v0.1.0
   * - DT-5
     - Contract Tests DTO : 0% (aucun mecanisme backend<->frontend)
     - MOYEN
     - 8h
     - v0.1.0
   * - DT-6
     - Use case unit tests : RESOLU (30/30 modules ont des tests avec mocks)
     - ~~FAIBLE~~
     - ~~20h~~
     - ✅ v0.1.0
   * - DT-7
     - Hardcoded total_eligible_voters=10 dans poll_use_cases
     - FAIBLE
     - 1h
     - v0.1.0
   * - DT-8
     - Marketplace handlers 100% stub (#335) — #276 fermee sans impl DB
     - HAUT
     - 20h
     - v0.1.0
   * - DT-9
     - Individual member handlers 100% stub (#336) — #280 fermee sans impl DB
     - HAUT
     - 16h
     - v0.1.0
   * - DT-10
     - Consent handlers 100% stub (#337) — #326 fermee sans persistance
     - HAUT
     - 12h
     - v0.1.0
   * - DT-11
     - RBAC manquant sur 9 endpoints gamification (#340)
     - HAUT
     - 4h
     - v0.1.0
   * - DT-12
     - Uuid::nil() hardcode pour unit_id dans energy_bill_upload (#338)
     - MOYEN
     - 2h
     - v0.1.0
   * - DT-13
     - API key rotation retourne 501 Not Implemented (#339)
     - MOYEN
     - 4h
     - v0.2.0
   * - DT-14
     - Paiement auto contractor post-validation non implemente (#341)
     - MOYEN
     - 8h
     - v0.1.0


=========================================================
PARTIE III : WBS PAR JALON
=========================================================

Chaque Jalon est decompose en :

- **WP-FEAT** : Work Packages fonctionnels (nouvelles features)
- **WP-BDD** : Step definitions BDD
- **WP-TDD** : Tests unitaires manquants
- **WP-E2E** : Tests E2E backend
- **WP-PW** : Tests Playwright frontend
- **WP-DOC** : Documentation
- **WP-SOLID** : Verification/refactoring SOLID

=========================================================
8. Jalon 0 : Fondations Techniques [COMPLETE]
=========================================================

Aucun travail restant. Toutes les issues fermees.

**Livrables** : Architecture hexagonale, 73 endpoints API, tests E2E,
load tests (287 req/s), documentation Sphinx.

=========================================================
9. Jalon 1 : Securite & GDPR [90%]
=========================================================

**Debloque** : 50-100 coproprietes (beta publique)

**Release cible** : v0.1.0 (bugs legal + RBAC + Playwright) / v0.2.0 (#48 itsme)

**Issues ouvertes** : #331, #337, #340

.. note::

   **Mise a jour 2026-03-25** : #48 (itsme/eID) en Jalon 4. Issues #271, #272, #273
   fermees. 3 nouvelles issues detectees par audit code : #331 (Playwright E2E 48 fichiers),
   #337 (consent handlers stubs), #340 (RBAC gamification 9 endpoints).

9.1 WP-FEAT-J1 : Issues restantes
--------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Cycle TDD
   * - WP-FEAT-J1.1
     - ~~Quorum 50%+ validation AG~~ FERME
     - ~~#271~~
     - ~~2h~~
     - ~~BC3~~
     - ✅
   * - WP-FEAT-J1.2
     - ~~2e convocation si quorum non atteint~~ FERME
     - ~~#272~~
     - ~~2h~~
     - ~~BC3~~
     - ✅
   * - WP-FEAT-J1.3
     - ~~Reduction vote mandataire~~ FERME
     - ~~#273~~
     - ~~2h~~
     - ~~BC3~~
     - ✅
   * - WP-FEAT-J1.4
     - **RBAC gamification** : 9 endpoints acceptent n'importe quel user authentifie.
       Ajouter require_role admin/superadmin sur create/update/delete handlers.
     - #340
     - 4h
     - BC5
     - TDD: gamification_handlers unit tests
   * - WP-FEAT-J1.5
     - **Consent handlers stubs** : #326 fermee mais 0 persistance DB.
       Creer ConsentRepository + migration + wiring.
     - #337
     - 12h
     - BC7
     - BDD: consent.feature, E2E: e2e_consent
   * - WP-PW-J1.1
     - **Playwright E2E** : 48 fichiers E2E couvrant tous les modules.
       Ecrire specs + stabiliser test infra.
     - #331
     - 24h
     - All FE
     - Playwright test suite

**Total Jalon 1 restant** : ~40h

=========================================================
10. Jalon 2 : Conformite Legale Belge [COMPLETE]
=========================================================

Aucun travail fonctionnel restant. Toutes les 17 issues fermees.

**Dette test/doc a combler (incluse dans v0.5.0)** :

.. list-table::
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache (dette rattrapee)
     - Heures
     - BC
   * - WP-BDD-J2.1
     - ``payments.feature`` (18 sc.) + ``payment_methods.feature`` (10 sc.)
     - 8h
     - BC2
   * - WP-BDD-J2.2
     - ``journal_entries.feature`` (8 sc.)
     - 3h
     - BC2
   * - WP-BDD-J2.3
     - ``call_for_funds.feature`` (10 sc.) + ``owner_contributions.feature`` (8 sc.)
     - 6h
     - BC12
   * - WP-BDD-J2.4
     - ``charge_distribution.feature`` (6 sc.)
     - 2h
     - BC2
   * - WP-E2E-J2.1
     - Fix ``e2e_payments.rs`` + ``e2e_payment_recovery.rs`` + ``e2e_budget.rs``
     - 2h
     - BC2
   * - WP-E2E-J2.2
     - Fix ``e2e_etat_date.rs`` + ``e2e_documents.rs``
     - 1h
     - BC4
   * - WP-DOC-J2.1
     - Creer ``docs/PAYMENT_INTEGRATION.rst`` (~200 lignes)
     - 3h
     - BC2

**Total Jalon 2 dette** : 25h (inclus dans budget v0.5.0)

=========================================================
11. Jalon 3 : Features Differenciantes [86%]
=========================================================

**Debloque** : 500-1,000 coproprietes (differenciation marche)

**Release cible** : v0.1.0

**Issues ouvertes** : #335, #336, #338, #341

.. note::

   **Mise a jour 2026-03-25** : 24/28 issues fermees. #274-280 fermees.
   4 nouvelles issues detectees par audit code : #335 (marketplace stubs),
   #336 (individual member stubs), #338 (Uuid::nil energy bill),
   #341 (auto-payment contractor). Ce sont des implementations manquantes
   derriere des issues precedemment fermees.

11.1 WP-FEAT-J3 : Issues restantes (4 ouvertes, 7 fermees)
------------------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Etat actuel
   * - WP-FEAT-J3.1
     - ~~BC15 AG Visioconference~~ FERME
     - ~~#274~~
     - ~~8h~~
     - ~~BC15~~
     - ✅
   * - WP-FEAT-J3.2
     - ~~BC17 AGE Agile~~ FERME
     - ~~#279~~
     - ~~8h~~
     - ~~BC17~~
     - ✅
   * - WP-FEAT-J3.3
     - ~~BC16 Backoffice prestataires~~ FERME
     - ~~#275~~
     - ~~6h~~
     - ~~BC16~~
     - ✅
   * - WP-FEAT-J3.4
     - **Marketplace handlers stubs** : #276 fermee mais handlers
       retournent JSON hardcode. Creer domain+repo+migration.
     - #335
     - 20h
     - BC14
     - Handlers stubs only, 0 persistance
   * - WP-FEAT-J3.5
     - **Individual member handlers stubs** : #280 fermee mais handlers
       retournent JSON hardcode. Creer repo+migration+GDPR consent.
     - #336
     - 16h
     - BC8
     - Handlers stubs only, 0 persistance
   * - WP-FEAT-J3.6
     - **Uuid::nil() energy bill** : unit_id hardcode a Uuid::nil()
       dans energy_bill_upload_handlers.rs. Lookup unit_owners requis.
     - #338
     - 2h
     - BC8
     - Bug silencieux, GET retourne vide
   * - WP-FEAT-J3.7
     - **Paiement auto contractor** : Apres validation ContractorReport,
       le paiement automatique lie au devis n'est pas declenche.
     - #341
     - 8h
     - BC16+BC2
     - TODO B16-6 dans use_cases
   * - WP-FEAT-J3.8
     - ~~Guide legal contextuel UI~~ FERME
     - ~~#277~~
     - ~~10h~~
     - ~~BC3~~
     - ✅
   * - WP-FEAT-J3.9
     - ~~Orchestrateur energie neutre~~ FERME
     - ~~#280~~
     - ~~16h~~
     - ~~BC8~~
     - ✅
   * - WP-FEAT-J3.10
     - ~~Blog 18 articles RST~~ FERME
     - ~~#278~~
     - ~~22h~~
     - ~~Docs~~
     - ✅

**Total Jalon 3 restant** : ~46h (stubs + bug + auto-payment)

=========================================================
12. Jalon 4 : Automation & Integrations [91%]
=========================================================

**Debloque** : 1,000-2,000 coproprietes (scalabilite)

**Release cible** : v0.2.0

**Issues ouvertes** : 2 (#339 API key rotation, #48 itsme)

.. note::

   **Mise a jour 2026-03-25** : 20/22 issues fermees. MCP Tools (#252-265) fermes.
   Reste #339 (API key rotation 501 Not Implemented) et #48 (itsme/eID).

12.1 WP-FEAT-J4 : Features restantes
--------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Approche DDD/SOLID
   * - WP-FEAT-J4.1
     - **WCAG 2.1 AA Accessibility** : Audit Lighthouse/axe,
       fix contrastes, aria-labels, navigation clavier,
       focus management, skip-to-content
     - #93
     - 40h
     - All FE
     - Component-level TDD avec axe-playwright.
       **S**: 1 composant = 1 scope accessible.
       **O**: Mixins CSS accessibility extensibles.
   * - WP-FEAT-J4.2
     - **RBAC granulaire** : Matrice droits dynamique.
       Nouvelles entites ``Permission``, ``RolePermission``.
       Middleware ``RequirePermission("building:edit")``.
     - #72
     - 30h
     - BC1
     - **DDD** : Nouveau BC "Authorization" ou extension BC1.
       **TDD** : RED=test middleware deny, GREEN=implemente.
       **S**: Middleware ne gere que l'autorisation.
       **I**: Trait ``PermissionChecker`` separe de ``AuthRepository``.
   * - WP-FEAT-J4.3
     - **Roles Organization Admin / Building Manager** :
       Etendre enum UserRole. Migration DB. UI role selector.
     - #71
     - 15h
     - BC1
     - **DDD** : Extends existing UserRoleAssignment aggregate.
       **BDD** : ``roles_extended.feature`` (8 scenarios).
       **L**: Nouveaux roles substituables dans AuthenticatedUser.
   * - WP-FEAT-J4.4
     - **Documentation GDPR & QA** : Document de conformite formel,
       registre des traitements (Article 30), DPO contact,
       procedure de breach notification.
     - #67
     - 12h
     - BC7
     - Principalement documentation, pas de code.

12.2 WP-BDD-J4 : Step Definitions (dette)
--------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature -> Step Definitions
     - Scenarios
     - Heures
   * - WP-BDD-J4.1
     - ``tickets.feature`` -> ``bdd_operations.rs``
     - 17
     - 6h
   * - WP-BDD-J4.2
     - ``notifications.feature`` -> ``bdd_operations.rs``
     - 14
     - 5h
   * - WP-BDD-J4.3
     - ``energy_campaigns.feature`` -> ``bdd_operations.rs``
     - 14
     - 5h

12.3 WP-BDD-J4 NEW : BDD nouvelles features
----------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature (a ecrire + implementer)
     - Scenarios est.
     - Heures
   * - WP-BDD-J4.4
     - ``rbac.feature`` : permission check, deny, role hierarchy
     - ~12
     - 8h
   * - WP-BDD-J4.5
     - ``wcag.feature`` : focus trap, aria-labels, contrast (BDD atypique)
     - ~6
     - 3h
   * - WP-BDD-J4.6
     - ``roles_extended.feature`` : OrgAdmin, BuildingManager
     - ~8
     - 5h

12.4 WP-E2E-J4 : Fix E2E (dette)
----------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-E2E-J4.1
     - Fix ``e2e_tickets.rs`` (938 LOC)
     - 1h
   * - WP-E2E-J4.2
     - Fix ``e2e_notifications.rs`` (784 LOC)
     - 1h
   * - WP-E2E-J4.3
     - Fix ``e2e_board.rs`` + ``e2e_board_dashboard.rs``
     - 1h
   * - WP-E2E-J4.4
     - Fix ``e2e_meetings.rs`` + ``e2e_unit_owner.rs``
     - 1h

12.5 WP-DOC-J4
----------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-DOC-J4.1
     - Creer ``docs/TICKETS_MAINTENANCE.rst``
     - 2h
   * - WP-DOC-J4.2
     - Creer ``docs/NOTIFICATIONS_SYSTEM.rst``
     - 2h
   * - WP-DOC-J4.3
     - Creer ``docs/RBAC_PERMISSIONS.rst`` (nouveau systeme)
     - 3h
   * - WP-DOC-J4.4
     - Creer ``docs/WCAG_ACCESSIBILITY.rst``
     - 2h
   * - WP-DOC-J4.5
     - Completer ``docs/GDPR_COMPLIANCE.rst`` (registre Art. 30)
     - 3h

12.6 WP-PW-J4 : Playwright
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Spec
     - Heures
   * - WP-PW-J4.1
     - ``Tickets.spec.ts`` : create, assign, resolve, close
     - 2h
   * - WP-PW-J4.2
     - ``Notifications.spec.ts`` : list, mark read, preferences
     - 2h
   * - WP-PW-J4.3
     - ``Accessibility.spec.ts`` : axe-playwright audit pages critiques
     - 3h

**Total Jalon 4** : ~170h (97h features + 44h dette test + 22h docs + 7h PW)

=========================================================
13. Jalon 5 : Mobile & API Publique [17%]
=========================================================

**Debloque** : 2,000-5,000 coproprietes (expansion)

**Release cible** : v1.0.0

**Issues ouvertes** : #97, #98, #266, #267, #268, #295, #296, #297, #298, #299

13.1 WP-FEAT-J5 : Features
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Approche DDD/SOLID
   * - WP-FEAT-J5.1
     - **PWA Offline Mode** : Service Worker avec Workbox,
       cache API responses, offline-first pour lecture,
       sync queue pour ecriture, manifest.json,
       install prompt, push notifications
     - #87
     - 60h
     - All FE
     - **TDD** : Playwright offline mode tests.
       **S** : ServiceWorker isole du code app.
       **O** : Cache strategies extensibles (CacheFirst, NetworkFirst).
   * - WP-FEAT-J5.2
     - **BI & Analytics Dashboard** : Charts (Chart.js/D3),
       KPIs syndic temps reel, export PDF rapports,
       comparaisons inter-buildings, alertes seuils
     - #97
     - 50h
     - BC10
     - **DDD** : Nouveau aggregate ``AnalyticsQuery``.
       **TDD** : Unit tests aggregations SQL.
       **S** : Analytics isole du CRUD.
       **I** : Trait ``AnalyticsProvider`` specialise.
   * - WP-FEAT-J5.3
     - **Mobile App natif** (etude) : React Native ou
       Capacitor wrapper de la PWA.
       Biometric auth (FaceID/TouchID).
       Push notifications natives.
     - #98
     - 80h
     - All
     - **SOLID** : Couche API identique (REST).
       Nouveau repo frontend mobile.
       **BDD** : ``mobile.feature`` scenarios cross-platform.

13.2 WP-BDD-J5
----------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature (a ecrire + implementer)
     - Scenarios est.
     - Heures
   * - WP-BDD-J5.1
     - ``pwa_offline.feature`` : cache, sync, conflict resolution
     - ~10
     - 6h
   * - WP-BDD-J5.2
     - ``analytics.feature`` : KPIs, exports, comparisons
     - ~12
     - 6h

13.3 WP-DOC-J5
----------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-DOC-J5.1
     - ``docs/PWA_OFFLINE.rst``
     - 3h
   * - WP-DOC-J5.2
     - ``docs/ANALYTICS_DASHBOARD.rst``
     - 3h
   * - WP-DOC-J5.3
     - ``docs/API_PUBLIC_v1.rst`` (documentation API publique)
     - 5h

13.4 WP-PW-J5
---------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Spec
     - Heures
   * - WP-PW-J5.1
     - ``PWA.spec.ts`` : install, offline browse, sync
     - 4h
   * - WP-PW-J5.2
     - ``Analytics.spec.ts`` : dashboard, charts, export
     - 3h

**Total Jalon 5** : ~230h (190h features + 12h BDD + 11h docs + 7h PW + 10h tests)

=========================================================
14. Jalon 6 : Intelligence & Expansion [20% -> 100%]
=========================================================

**Debloque** : 5,000-10,000 coproprietes (leadership PropTech)

**Release cible** : v2.0.0

**Prerequis** : Equipe 3-4 ETP, revenus >10k EUR/mois

**Issues ouvertes** : #94, #95, #96, #109

.. warning::

   Ce jalon necessite des competences specialisees (Data Science,
   IoT, FinTech). Le WBS ci-dessous est indicatif et sera affine
   quand les prerequis seront remplis.

14.1 WP-FEAT-J6 : Features
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Approche DDD
   * - WP-FEAT-J6.1
     - **IoT Platform complet** : MQTT Broker (Mosquitto/EMQX),
       TimescaleDB pour time-series, edge gateway,
       dashboards temps reel, alertes seuils,
       integration Enedis API (au-dela du prototype Linky actuel)
     - #109
     - 200h
     - BC8
     - **DDD** : Nouveau BC ``IoTPlatform`` avec aggregates
       ``Sensor``, ``Alert``, ``DataStream``.
       **SOLID/I** : Trait ``MeterProvider`` (Linky, Ores, Generic).
       **BDD** : 20+ scenarios (readings, alerts, aggregations).
   * - WP-FEAT-J6.2
     - **AI Features** : OCR factures (Tesseract/Google Vision),
       predictions budgetaires (ARIMA), detection anomalies
       consommation, chatbot reglementaire syndic
     - #94
     - 300h
     - BC10+NEW
     - **DDD** : Nouveau BC ``AIAssistant``.
       **S** : Chaque capacite IA = 1 service domain isole.
       **D** : Trait ``OCRProvider``, ``PredictionModel`` abstraits.
       **TDD** : Golden file tests pour OCR, regression tests pour ML.
   * - WP-FEAT-J6.3
     - **Service Provider Marketplace** : Annuaire prestataires
       verifies, avis et notations, demande de devis
       automatisee, matching prestataire/besoin
     - #95
     - 120h
     - BC4+NEW
     - **DDD** : Nouveau aggregate ``ServiceProvider`` dans BC4.
       **BDD** : ``marketplace.feature`` (15 scenarios).
       **O** : Scoring algorithm extensible.
   * - WP-FEAT-J6.4
     - **Sustainability Tracking** : Empreinte carbone batiment,
       DPE (Diagnostic Performance Energetique),
       recommandations eco, certification energetique
     - #96
     - 80h
     - BC8+NEW
     - **DDD** : Extend BC8 avec ``CarbonFootprint`` entity.
       **BDD** : ``sustainability.feature`` (10 scenarios).

14.2 WP-BDD-J6
----------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature (a ecrire)
     - Scenarios est.
     - Heures
   * - WP-BDD-J6.1
     - ``iot_platform.feature`` (full platform, not just readings)
     - ~20
     - 12h
   * - WP-BDD-J6.2
     - ``ai_ocr.feature`` + ``ai_predictions.feature``
     - ~15
     - 10h
   * - WP-BDD-J6.3
     - ``marketplace.feature``
     - ~15
     - 8h
   * - WP-BDD-J6.4
     - ``sustainability.feature``
     - ~10
     - 5h

14.3 WP-DOC-J6
----------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-DOC-J6.1
     - ``docs/IOT_PLATFORM_FULL.rst`` (extension du IoT existant)
     - 5h
   * - WP-DOC-J6.2
     - ``docs/AI_FEATURES.rst``
     - 5h
   * - WP-DOC-J6.3
     - ``docs/SERVICE_MARKETPLACE.rst``
     - 3h
   * - WP-DOC-J6.4
     - ``docs/SUSTAINABILITY_TRACKING.rst``
     - 3h

**Total Jalon 6** : ~766h (700h features + 35h BDD + 16h docs + 15h PW)

=========================================================
15. Jalon 7 : Platform Economy [0% -> 100%]
=========================================================

**Debloque** : 10,000+ coproprietes (echelle europeenne)

**Release cible** : v3.0.0

**Prerequis** : Equipe 10-15 ETP, audits securite externes

**Issue ouverte** : #111

15.1 WP-FEAT-J7 : Features
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Approche DDD
   * - WP-FEAT-J7.1
     - **API Publique v2** : Versioned API (v1 legacy, v2 new),
       rate limiting par tenant, API keys management,
       webhook subscriptions, pagination cursor-based
     - #111
     - 120h
     - BC13
     - **DDD** : Extend BC13 ``PublicAPI`` avec aggregates
       ``ApiKey``, ``WebhookSubscription``, ``RateLimit``.
       **SOLID/O** : Versionning par namespace (pas modification).
       **I** : ``ApiKeyRepository`` separe de ``WebhookRepository``.
   * - WP-FEAT-J7.2
     - **SDK Multi-langages** : Python, JavaScript, PHP, Ruby.
       Auto-generation depuis OpenAPI spec.
       Tests integration par langage.
     - #111
     - 80h
     - BC13
     - **TDD** : Test suite par SDK (pytest, jest, phpunit).
       **D** : SDKs dependent uniquement de l'API HTTP publique.
   * - WP-FEAT-J7.3
     - **Marketplace Plugins** : Store modules tiers,
       plugin API, sandboxing, review process,
       revenue sharing model
     - #111
     - 150h
     - NEW
     - **DDD** : Nouveau BC ``PluginEcosystem``.
       **S** : Plugin runner isole (WASM sandbox).
       **O** : Hook system extensible.
   * - WP-FEAT-J7.4
     - **Blockchain Voting** (optionnel) : Votes AG immutables
       sur Polygon, smart contracts Solidity,
       verification on-chain, audit trail blockchain
     - #111
     - 200h
     - BC3
     - **DDD** : ``BlockchainVoteAdapter`` implemente ``VoteRepository``.
       **D** : Blockchain = infrastructure adapter, pas domain concern.
       **L** : Substituable avec PostgreSQL adapter existant.
   * - WP-FEAT-J7.5
     - **Carbon Credits Trading** (optionnel) : Tokenisation
       economies CO2, trading entre coproprietes
     - #111
     - 120h
     - BC8
     - **DDD** : ``CarbonCreditToken`` aggregate.
       Anticorruption layer vers blockchain.

**Total Jalon 7** : ~750h+ (670h features + 40h BDD + 20h docs + 20h PW)

=========================================================
PARTIE IV : RELEASES & PLANNING
=========================================================

16. Matrice Releases (mise a jour mars 2026)
----------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 15 30 15 15 15

   * - Release
     - Jalons
     - Contenu principal
     - Tests cible
     - Effort restant
     - Prerequis equipe
   * - **v0.1.0**
     - J0-J3
     - Premiere release. Bugs legal J1, features J3 (BC14-17),
       E2E backend (27), Playwright (32), docs, blog
     - 1,400 tests
     - ~96h
     - Solo
   * - **v0.2.0**
     - J4 (partiel)
     - MCP Tools AI Syndic (14 tools), auth itsme/eID
     - 1,600 tests
     - ~120h
     - Solo
   * - **v1.0.0**
     - J4 complet + J5
     - WCAG AA, RBAC, PWA offline, analytics, K3s infra
     - 1,800 tests
     - ~300h
     - Solo + 1 contributeur
   * - **v2.0.0**
     - J6 complet
     - IoT platform, AI, marketplace, sustainability
     - 2,000 tests
     - ~766h
     - 3-4 ETP
   * - **v3.0.0**
     - J7 complet
     - API v2, SDK, plugins, blockchain (opt.)
     - 2,200 tests
     - ~750h
     - 10-15 ETP

17. Timeline Realiste (Solo Dev 10-15h/sem)
---------------------------------------------

.. code-block:: text

   2026
   |-- Mars-Juin     : v0.1.0 (J0-J3 complet)           ~96h   8-10 sem
   |-- Juillet-Oct   : v0.2.0 (MCP + itsme)            ~120h   8-12 sem
   |-- Nov-2027 Q1   : v1.0.0 (J4+J5, WCAG, PWA)      ~300h   (solo + contributeur)
   |
   2027
   |-- Q2-Q4         : v2.0.0 (IoT + AI)               ~766h   (besoin 3-4 ETP)
   |
   2028
   |-- Q1-Q4         : v3.0.0 (API v2 + ecosystem)     ~750h   (besoin 10-15 ETP)

.. note::

   Cette timeline est un **scenario conservateur solo**.
   La v0.1.0 est la priorite absolue (branche ``release/0.1.0`` active).

18. Budget Test par Release
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 15 15 15 15 15 10

   * - Release
     - Unit
     - BDD
     - E2E
     - PW
     - Bench
     - Total
   * - v0.1.0
     - 777
     - 454
     - 200
     - 45
     - 5
     - 1,481
   * - v0.2.0
     - 800
     - 500
     - 250
     - 70
     - 5
     - 1,625
   * - v1.0.0
     - 850
     - 560
     - 300
     - 120
     - 15
     - 1,845
   * - v2.0.0
     - 900
     - 620
     - 350
     - 160
     - 20
     - 2,050
   * - v3.0.0
     - 1000
     - 680
     - 400
     - 200
     - 25
     - 2,305


=========================================================
PARTIE V : ANNEXES
=========================================================

A. Inventaire Complet des Work Packages
-----------------------------------------

.. list-table:: Tous les Work Packages
   :header-rows: 1
   :widths: 10 10 40 10 10 10

   * - Jalon
     - WP
     - Description
     - Heures
     - Type
     - Release
   * - J1
     - WP-FEAT-J1.1
     - Finaliser 2FA TOTP end-to-end
     - 4
     - FEAT
     - v0.1.0
   * - J1
     - WP-FEAT-J1.2
     - Fix E2E admin login timeout (#66)
     - 2
     - FEAT
     - v0.1.0
   * - J1
     - WP-FEAT-J1.3
     - Playwright E2E units + documents (#69)
     - 4
     - PW
     - v0.1.0
   * - J1
     - WP-FEAT-J1.4
     - Authentification itsme/eID (#48)
     - 8
     - FEAT
     - v0.2.0
   * - J1
     - WP-BDD-J1.1
     - two_factor.feature step defs
     - 4
     - BDD
     - v0.1.0
   * - J1
     - WP-BDD-J1.2
     - organizations.feature step defs
     - 3
     - BDD
     - v0.1.0
   * - J3
     - WP-FEAT-J3.1-4
     - PDF Generation (4 types documents)
     - 21
     - FEAT
     - v0.1.0
   * - J3
     - WP-FEAT-J3.5
     - Contractor Backoffice frontend
     - 12
     - FEAT
     - v0.1.0
   * - J4
     - WP-FEAT-J4.1
     - WCAG 2.1 AA Accessibility (#93)
     - 40
     - FEAT
     - v0.2.0
   * - J4
     - WP-FEAT-J4.2
     - RBAC granulaire (#72)
     - 30
     - FEAT
     - v0.2.0
   * - J4
     - WP-FEAT-J4.3
     - Roles OrgAdmin/BuildingManager (#71)
     - 15
     - FEAT
     - v0.2.0
   * - J4
     - WP-FEAT-J4.4
     - Documentation GDPR formelle (#67)
     - 12
     - DOC
     - v0.2.0
   * - J5
     - WP-FEAT-J5.1
     - PWA Offline Mode (#87)
     - 60
     - FEAT
     - v1.0.0
   * - J5
     - WP-FEAT-J5.2
     - BI Analytics Dashboard (#97)
     - 50
     - FEAT
     - v1.0.0
   * - J5
     - WP-FEAT-J5.3
     - Mobile App natif (#98)
     - 80
     - FEAT
     - v1.0.0
   * - J6
     - WP-FEAT-J6.1
     - IoT Platform complet (#109)
     - 200
     - FEAT
     - v2.0.0
   * - J6
     - WP-FEAT-J6.2
     - AI Features (#94)
     - 300
     - FEAT
     - v2.0.0
   * - J6
     - WP-FEAT-J6.3
     - Service Provider Marketplace (#95)
     - 120
     - FEAT
     - v2.0.0
   * - J6
     - WP-FEAT-J6.4
     - Sustainability Tracking (#96)
     - 80
     - FEAT
     - v2.0.0
   * - J7
     - WP-FEAT-J7.1-5
     - API v2, SDK, Plugins, Blockchain, Carbon
     - 670
     - FEAT
     - v3.0.0
   * - TRANS
     - WP-INFRA-*
     - Infrastructure BDD (4 fichiers + CI + Makefile)
     - 6.25
     - INFRA
     - v0.1.0
   * - TRANS
     - WP-E2E-C*
     - Fix E2E commun (common/mod.rs + board)
     - 3
     - E2E
     - v0.1.0
   * - TRANS
     - WP-CI-*
     - Pipeline CI/CD updates
     - 2.75
     - CI
     - v0.1.0
   * - TRANS
     - WP-REL-*
     - Release mechanics (tags, notes, version bump)
     - 4.25
     - REL
     - v0.1.0

B. Effort Total par Type (mise a jour mars 2026)
---------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 15 15 15

   * - Type
     - v0.1.0
     - v0.2.0
     - v1.0.0
     - v2.0.0+
     - **Total**
   * - FEAT (features)
     - 90h
     - 120h
     - 200h
     - 700h
     - **1,110h**
   * - BDD/E2E/PW (tests)
     - 40h
     - 30h
     - 50h
     - 60h
     - **180h**
   * - DOC (docs)
     - 20h
     - 10h
     - 15h
     - 16h
     - **61h**
   * - INFRA/CI/REL
     - 6h
     - 5h
     - 35h
     - 10h
     - **56h**
   * - **Total**
     - **156h**
     - **165h**
     - **300h**
     - **786h**
     - **1,407h**

.. note::

   v3.0.0 (Jalon 7) ajoute ~750h supplementaires pour un total
   projet de ~2,127h. Ce jalon est conditionnel a l'equipe de 10-15 ETP.

C. Definition of Done (Globale)
---------------------------------

**Par Work Package Feature** :

.. code-block:: text

   [x] DDD : Entite domain avec invariants valides dans constructeur
   [x] DDD : Port (trait) defini dans application/ports/
   [x] DDD : Use case orchestre les ports (pas de SQL direct)
   [x] DDD : Repository PostgreSQL implemente le port
   [x] DDD : Handler Actix-web utilise AppState injection

   [x] BDD : Feature .feature ecrit en Gherkin
   [x] BDD : Step definitions implementees et passent
   [x] BDD : Scenarios couvrent happy path + error paths

   [x] TDD : Tests unitaires domain (>= 5 par entite)
   [x] TDD : E2E test pour endpoint principal
   [x] TDD : Aucune regression (cargo test passe)

   [x] SOLID/S : Fichier < 800 LOC (sinon split)
   [x] SOLID/O : Pas de modification des traits existants
   [x] SOLID/L : Implementations substituables
   [x] SOLID/I : Trait < 20 methodes
   [x] SOLID/D : Domain ne depend pas d'infrastructure

   [x] DOC : Documentation feature creee dans docs/
   [x] DOC : CHANGELOG mis a jour
   [x] DOC : OpenAPI spec mise a jour

   [x] CODE : cargo fmt + clippy clean
   [x] CODE : Pas de TODO/FIXME non-trackes

D. Arbre de Dependances Features
-----------------------------------

.. code-block:: text

   v0.5.0 (dette)
   |
   +-- v0.6.0 (J3: PDF + Contractor)
   |   |
   |   +-- v0.7.0 (J4: WCAG + RBAC + Roles)
   |       |
   |       +-- v1.0.0 (J5: PWA + Analytics + Mobile)
   |           |
   |           +-- v2.0.0 (J6: IoT + AI + Marketplace + Eco)
   |               |
   |               +-- v3.0.0 (J7: API v2 + SDK + Blockchain)

   Dependances inter-features :
   - RBAC (#72) bloque roles etendus (#71)
   - PWA (#87) prerequis pour Mobile (#98)
   - IoT readings (existant) prerequis pour IoT Platform (#109)
   - Analytics (#97) prerequis pour AI predictions (#94)
   - Marketplace prestataires (#95) etend Quotes (#91, existant)
   - API v2 (#111) prerequis pour SDK + Plugins

E. Glossaire DDD-BDD-TDD-SOLID
---------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Terme
     - Definition
   * - Aggregate Root
     - Entite racine qui garantit la coherence d'un groupe d'objets (ex: Building)
   * - Bounded Context
     - Perimetre linguistique et technique ou un modele est valide
   * - Port
     - Trait Rust definissant une interface (ex: ``BuildingRepository``)
   * - Adapter
     - Implementation d'un port (ex: ``PostgresBuildingRepository``)
   * - Use Case
     - Orchestration d'un ou plusieurs ports pour un cas d'usage metier
   * - Feature File
     - Fichier Gherkin (.feature) decrivant le comportement en langage naturel
   * - Step Definition
     - Fonction Rust liant un step Gherkin a du code executable
   * - World Struct
     - Structure portant l'etat entre les steps d'un scenario BDD
   * - RED-GREEN-REFACTOR
     - Cycle TDD : test qui echoue -> code minimal -> nettoyage
   * - Golden File Test
     - Test comparant la sortie a un fichier de reference (utile pour PDF/OCR)
   * - Anticorruption Layer
     - Couche d'isolation entre un BC et un systeme externe (ex: Stripe, Enedis)
