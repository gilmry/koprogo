=========================================================
KoproGo - WBS Release 0.5.0 (DDD-BDD-TDD-SOLID)
=========================================================

:Version: 1.0
:Date: 26 fevrier 2026
:Methode: Approche DDD + Pyramide des Tests (BDD/TDD/SOLID)
:Auteur: Claude Code (Audit automatise)
:Branche cible: release/v0.5.0
:Effort estime: ~134h (10-12 semaines a 10-15h/sem)

.. contents:: Table des matieres
   :depth: 4
   :local:

=========================================================
1. Philosophie : DDD - BDD - TDD - SOLID
=========================================================

1.1 Domain-Driven Design (DDD)
------------------------------

Le WBS est organise par **Bounded Contexts** (contextes bornes), chacun
representant un sous-domaine metier autonome avec ses propres entites,
services, ports et adaptateurs.

**13 Bounded Contexts identifies** :

.. list-table:: Bounded Contexts KoproGo
   :header-rows: 1
   :widths: 5 25 10 10 10

   * - #
     - Bounded Context
     - Entites
     - Endpoints
     - Tests Domain
   * - BC1
     - Core Infrastructure (Building, Unit, Owner, Org, Auth)
     - 9
     - ~50
     - 50
   * - BC2
     - Financial & Accounting (Expense, PCMN, Payment, Budget)
     - 7
     - ~90
     - 94
   * - BC3
     - Meeting & Governance (Meeting, Resolution, Poll, Board)
     - 8
     - ~60
     - 68
   * - BC4
     - Property Management (Ticket, Quote, WorkReport, EtatDate)
     - 6
     - ~70
     - 38
   * - BC5
     - Community Features (SEL, Notice, Skill, Sharing, Booking, Gamification)
     - 12
     - ~90
     - 107
   * - BC6
     - Notification & Preferences
     - 2
     - ~20
     - 11
   * - BC7
     - GDPR & Privacy (Articles 15-21)
     - 4
     - ~15
     - 23
   * - BC8
     - Energy & IoT (Campaigns, Linky, Readings)
     - 5
     - ~35
     - 46
   * - BC9
     - Security & 2FA (TOTP, Tokens)
     - 2
     - ~10
     - 17
   * - BC10
     - Reporting & Analytics (Reports, Dashboard)
     - 0 (cross-context)
     - ~10
     - 8
   * - BC11
     - SaaS Administration (Org, User mgmt)
     - 2
     - ~15
     - 8
   * - BC12
     - Revenue Management (CallForFunds, Contributions)
     - 2
     - ~12
     - 10
   * - BC13
     - Public & Open (Syndic public page)
     - 0 (cross-context)
     - ~1
     - 4
   * -
     - **TOTAL**
     - **52**
     - **~450+**
     - **501**

1.2 BDD (Behavior-Driven Development)
--------------------------------------

**Approche** : Scenarios Gherkin -> Step Definitions -> Use Cases

Chaque Bounded Context a ses propres fichiers ``.feature`` et est
implemente dans un fichier BDD dedie groupe par domaine.

**Architecture BDD choisie : 4 fichiers groupes + 1 legacy**

.. code-block:: text

   backend/tests/
   +-- bdd.rs                   # Legacy (24 features existants, 194 scenarios)
   +-- bdd_financial.rs         # BC2 + BC10 + BC12 (74 scenarios)
   +-- bdd_governance.rs        # BC3 + BC9 + BC11 + BC13 (74 scenarios)
   +-- bdd_operations.rs        # BC4 + BC6 + BC8 (83 scenarios)
   +-- bdd_community.rs         # BC5 (72 scenarios)
   +-- features/
       +-- (48 fichiers .feature, 473 scenarios total)

**Cycle BDD** pour chaque feature :

1. Ecrire le ``.feature`` (Given/When/Then) -- **FAIT pour les 24 nouveaux**
2. Ecrire le World struct + step definitions dans le fichier BDD
3. Executer ``cargo test --test bdd_<domain>``
4. Corriger jusqu'a ce que tous les scenarios passent
5. Refactorer si necessaire

1.3 TDD (Test-Driven Development)
----------------------------------

**Approche** : RED -> GREEN -> REFACTOR

Pour chaque composant nouveau ou modifie :

1. **RED** : Ecrire le test qui echoue (unit test domain, BDD scenario, E2E)
2. **GREEN** : Implementer le minimum pour faire passer le test
3. **REFACTOR** : Nettoyer sans casser les tests

**Pyramide des tests** :

.. code-block:: text

   Niveau        | Existant | Cible 0.5.0 | Delta
   --------------|----------|-------------|------
   Unit (domain) |    501   |    550+     |  +49
   BDD scenarios |    194   |    473      | +279
   E2E backend   |    ~30   |    215      | +185
   Playwright    |     27   |     64      |  +37
   Benchmarks    |      5   |      5      |   0
   --------------|----------|-------------|------
   TOTAL         |   ~757   |  ~1,307     | +550

1.4 SOLID
---------

**Verification SOLID par couche** :

**S** - Single Responsibility :
  - 1 entite = 1 fichier domaine
  - 1 use case = 1 orchestration metier
  - 1 handler = 1 endpoint HTTP
  - 1 repository = 1 persistence

**O** - Open/Closed :
  - Ports (traits) ouverts a extension, fermes a modification
  - Nouveaux repositories implementent les traits existants
  - Workflows extensibles via pattern State Machine

**L** - Liskov Substitution :
  - ``PostgresBuildingRepository`` substituable partout ou ``BuildingRepository`` est attendu
  - Mock repositories en tests substituent les implementations reelles

**I** - Interface Segregation :
  - Chaque port (trait) ne definit que les methodes de son domaine
  - ``TicketRepository`` ne contient pas de methodes de ``PaymentRepository``

**D** - Dependency Inversion :
  - Domain ne depend de rien (couche la plus interne)
  - Application depend des ports (traits, pas implementations)
  - Infrastructure implemente les ports et depend d'Application

=========================================================
2. Etat Actuel (Baseline)
=========================================================

2.1 Metriques Code
------------------

.. list-table:: Metriques Code Source (2026-02-26)
   :header-rows: 1

   * - Couche
     - Fichiers
     - LOC
   * - Domain (entities + services)
     - 62
     - ~22,500
   * - Application (ports + use_cases + dto)
     - 140
     - ~25,000
   * - Infrastructure (repos + handlers)
     - 100
     - ~40,000
   * - Migrations SQL
     - 58
     - ~7,200
   * - Tests backend
     - 70+
     - ~17,000
   * - Frontend (pages + components + lib)
     - 80+
     - ~15,000
   * - **TOTAL**
     - **~510**
     - **~126,700**

2.2 Couverture Tests Actuelle
-----------------------------

.. list-table:: Couverture Tests (2026-02-26)
   :header-rows: 1
   :widths: 20 15 15 15 35

   * - Couche Test
     - Fichiers
     - Tests
     - Statut
     - Gap
   * - Unit (domain)
     - 62
     - 501
     - VERT
     - Quelques entites faibles (work_report: 3, technical_inspection: 4)
   * - BDD (features)
     - 48
     - 473 scenarios
     - 279 SKIPPED
     - 24 features sans step definitions
   * - E2E (backend)
     - 23
     - ~215
     - 19/23 CASSES
     - Seulement e2e_http + e2e.rs + bdd compilent en CI
   * - Playwright (frontend)
     - 3
     - 27
     - PARTIEL
     - <5% des pages couvertes
   * - Benchmarks
     - 1
     - 5
     - VERT
     - Adequat pour 0.5.0

2.3 Documentation Actuelle
--------------------------

.. list-table:: Documentation Feature (2026-02-26)
   :header-rows: 1
   :widths: 40 15 45

   * - Feature
     - Doc existante?
     - Fichier
   * - Belgian Accounting PCMN
     - OUI
     - ``BELGIAN_ACCOUNTING_PCMN.rst`` (441 lignes)
   * - Invoice Workflow
     - OUI
     - ``INVOICE_WORKFLOW.rst`` (460 lignes)
   * - Payment Recovery
     - OUI
     - ``PAYMENT_RECOVERY_WORKFLOW.rst`` (471 lignes)
   * - Energy Buying Groups
     - OUI
     - ``ENERGY_BUYING_GROUPS.rst`` (598 lignes)
   * - IoT Integration
     - OUI
     - ``IOT_INTEGRATION.rst`` (1,247 lignes)
   * - Board of Directors
     - OUI
     - ``BOARD_OF_DIRECTORS_GUIDE.md``
   * - Multi-Owner Support
     - OUI
     - ``MULTI_OWNER_SUPPORT.md``
   * - Multi-Role Support
     - OUI
     - ``MULTI_ROLE_SUPPORT.md``
   * - 22+ autres features
     - NON
     - Seulement dans CLAUDE.md

**Couverture documentation features : 8/30 = 27%**

=========================================================
3. WBS par Bounded Context
=========================================================

Chaque BC est decompose en Work Packages (WP) suivant le cycle :

.. code-block:: text

   WP-BDD : BDD Step Definitions (feature -> step -> use_case)
   WP-E2E : E2E Test Fix/Creation (HTTP-level tests)
   WP-TDD : Unit Test Gaps (RED-GREEN-REFACTOR)
   WP-DOC : Documentation (standalone feature doc)
   WP-PW  : Playwright Spec (frontend E2E)

3.1 BC1 - Core Infrastructure
------------------------------

**Entites** : Building, Unit, Owner, Organization, User, UserRoleAssignment,
UnitOwner, RefreshToken

**Tests existants** : 50 unit tests, BDD step defs existants (building, auth,
multitenancy), E2E ``e2e.rs`` + ``e2e_auth.rs`` + ``e2e_unit_owner.rs``

.. list-table:: BC1 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-E2E-1.1
     - Fix ``e2e_auth.rs`` compilation (imports, DTO changes)
     - 1h
     - P0
   * - WP-E2E-1.2
     - Fix ``e2e_unit_owner.rs`` compilation
     - 1h
     - P0
   * - WP-DOC-1.1
     - Creer ``backend/README.md`` (architecture, modules, how-to)
     - 2h
     - P1
   * - WP-DOC-1.2
     - Creer ``frontend/README.md`` (stack, scripts, structure)
     - 2h
     - P1
   * - WP-PW-1.1
     - ``Login.spec.ts`` : login, register, token refresh
     - 2h
     - P1
   * - WP-PW-1.2
     - ``Buildings.spec.ts`` : CRUD building, navigation
     - 2h
     - P2

**Total BC1** : 10h

3.2 BC2 - Financial & Accounting
---------------------------------

**Entites** : Expense (1108 LOC), Account (536), InvoiceLineItem (255),
Payment (522), PaymentMethod (321), PaymentReminder (480), ChargeDistribution (342)

**Tests existants** : 94 unit tests, BDD existants (invoices, payment_recovery,
budget, etat_date), E2E ``e2e_payments.rs`` + ``e2e_payment_recovery.rs`` +
``e2e_budget.rs``

.. list-table:: BC2 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-2.1
     - Step defs ``payments.feature`` (18 scenarios) dans ``bdd_financial.rs``
     - 5h
     - P0
   * - WP-BDD-2.2
     - Step defs ``payment_methods.feature`` (10 scenarios) dans ``bdd_financial.rs``
     - 3h
     - P0
   * - WP-BDD-2.3
     - Step defs ``journal_entries.feature`` (8 scenarios) dans ``bdd_financial.rs``
     - 3h
     - P2
   * - WP-BDD-2.4
     - Step defs ``call_for_funds.feature`` (10 scenarios) dans ``bdd_financial.rs``
     - 3h
     - P2
   * - WP-BDD-2.5
     - Step defs ``owner_contributions.feature`` (8 scenarios) dans ``bdd_financial.rs``
     - 3h
     - P2
   * - WP-BDD-2.6
     - Step defs ``charge_distribution.feature`` (6 scenarios) dans ``bdd_financial.rs``
     - 2h
     - P2
   * - WP-BDD-2.7
     - Step defs ``dashboard.feature`` (4 scenarios) dans ``bdd_financial.rs``
     - 1h
     - P3
   * - WP-E2E-2.1
     - Fix ``e2e_payments.rs`` compilation
     - 1h
     - P0
   * - WP-E2E-2.2
     - Fix ``e2e_payment_recovery.rs`` compilation
     - 0.5h
     - P0
   * - WP-E2E-2.3
     - Fix ``e2e_budget.rs`` compilation
     - 0.5h
     - P1
   * - WP-DOC-2.1
     - Creer ``docs/PAYMENT_INTEGRATION.rst`` (~200 lignes)
     - 3h
     - P0
   * - WP-PW-2.1
     - ``Expenses.spec.ts`` : create expense, workflow approbation
     - 2h
     - P2

**Total BC2** : 27h

3.3 BC3 - Meeting & Governance
-------------------------------

**Entites** : Meeting (265), Convocation (543), ConvocationRecipient (393),
Resolution (461), Vote (264), Poll (401), PollVote (172), BoardMember (452)

**Tests existants** : 68 unit tests, BDD existants (meetings, board, polls),
E2E ``e2e_resolutions.rs`` + ``e2e_convocations.rs``

.. list-table:: BC3 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-3.1
     - Step defs ``resolutions.feature`` (14 scenarios) dans ``bdd_governance.rs``
     - 5h
     - P0
   * - WP-BDD-3.2
     - Step defs ``convocations.feature`` (13 scenarios) dans ``bdd_governance.rs``
     - 5h
     - P0
   * - WP-BDD-3.3
     - Step defs ``public_syndic.feature`` (4 scenarios) dans ``bdd_governance.rs``
     - 1h
     - P3
   * - WP-E2E-3.1
     - Fix ``e2e_resolutions.rs`` compilation (1,161 lignes)
     - 1h
     - P0
   * - WP-E2E-3.2
     - Fix ``e2e_convocations.rs`` compilation (1,501 lignes)
     - 1h
     - P0
   * - WP-E2E-3.3
     - Fix ``e2e_meetings.rs`` compilation
     - 0.5h
     - P1
   * - WP-DOC-3.1
     - Creer ``docs/CONVOCATIONS_AG.rst`` (~200 lignes)
     - 2h
     - P0
   * - WP-DOC-3.2
     - Creer ``docs/MEETINGS_RESOLUTIONS_VOTING.rst`` (~200 lignes)
     - 2h
     - P1
   * - WP-PW-3.1
     - ``Meetings.spec.ts`` : create meeting, resolutions, vote
     - 2h
     - P2

**Total BC3** : 19.5h

3.4 BC4 - Property Management & Maintenance
--------------------------------------------

**Entites** : Ticket (444), Quote (619), WorkReport (203),
TechnicalInspection (269), EtatDate (618), Document (166)

**Tests existants** : 38 unit tests, E2E ``e2e_tickets.rs`` + ``e2e_quotes.rs``
+ ``e2e_etat_date.rs`` + ``e2e_documents.rs``

.. list-table:: BC4 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-4.1
     - Step defs ``tickets.feature`` (17 scenarios) dans ``bdd_operations.rs``
     - 6h
     - P0
   * - WP-BDD-4.2
     - Step defs ``quotes.feature`` (13 scenarios) dans ``bdd_governance.rs``
     - 5h
     - P0
   * - WP-BDD-4.3
     - Step defs ``work_reports.feature`` (10 scenarios) dans ``bdd_operations.rs``
     - 3h
     - P2
   * - WP-BDD-4.4
     - Step defs ``technical_inspections.feature`` (12 scenarios) dans ``bdd_operations.rs``
     - 4h
     - P2
   * - WP-E2E-4.1
     - Fix ``e2e_tickets.rs`` compilation (938 lignes)
     - 1h
     - P0
   * - WP-E2E-4.2
     - Fix ``e2e_quotes.rs`` compilation (1,381 lignes)
     - 1h
     - P0
   * - WP-E2E-4.3
     - Fix ``e2e_etat_date.rs`` + ``e2e_documents.rs`` compilation
     - 1h
     - P1
   * - WP-DOC-4.1
     - Creer ``docs/TICKETS_MAINTENANCE.rst`` (~150 lignes)
     - 2h
     - P0
   * - WP-DOC-4.2
     - Creer ``docs/CONTRACTOR_QUOTES.rst`` (~150 lignes)
     - 2h
     - P0
   * - WP-PW-4.1
     - ``Tickets.spec.ts`` : create ticket, workflow, assign
     - 2h
     - P1

**Total BC4** : 27h

3.5 BC5 - Community Features
------------------------------

**Entites** : LocalExchange (577), OwnerCreditBalance (328), Notice (908),
Skill (625), SharedObject (801), ResourceBooking (839), Achievement (542),
Challenge (616)

**Tests existants** : 107 unit tests, BDD existants (local_exchange, polls),
E2E ``e2e_local_exchange.rs``

.. list-table:: BC5 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-5.1
     - Step defs ``notices.feature`` (15 scenarios) dans ``bdd_community.rs``
     - 5h
     - P1
   * - WP-BDD-5.2
     - Step defs ``skills.feature`` (13 scenarios) dans ``bdd_community.rs``
     - 4h
     - P1
   * - WP-BDD-5.3
     - Step defs ``shared_objects.feature`` (15 scenarios) dans ``bdd_community.rs``
     - 5h
     - P1
   * - WP-BDD-5.4
     - Step defs ``resource_bookings.feature`` (16 scenarios) dans ``bdd_community.rs``
     - 5h
     - P1
   * - WP-BDD-5.5
     - Step defs ``gamification.feature`` (13 scenarios) dans ``bdd_community.rs``
     - 4h
     - P2
   * - WP-E2E-5.1
     - Fix ``e2e_local_exchange.rs`` compilation (580 lignes)
     - 0.5h
     - P1
   * - WP-DOC-5.1
     - Creer ``docs/COMMUNITY_FEATURES.rst`` (~400 lignes, couvre 6 phases)
     - 3h
     - P1

**Total BC5** : 26.5h

3.6 BC6 - Notifications
-------------------------

**Entites** : Notification (425), NotificationPreference

**Tests existants** : 11 unit tests, E2E ``e2e_notifications.rs``

.. list-table:: BC6 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-6.1
     - Step defs ``notifications.feature`` (14 scenarios) dans ``bdd_operations.rs``
     - 5h
     - P0
   * - WP-E2E-6.1
     - Fix ``e2e_notifications.rs`` compilation (784 lignes)
     - 1h
     - P0
   * - WP-DOC-6.1
     - Creer ``docs/NOTIFICATIONS_SYSTEM.rst`` (~150 lignes)
     - 2h
     - P0
   * - WP-PW-6.1
     - ``Notifications.spec.ts`` : list, mark read, preferences
     - 2h
     - P2

**Total BC6** : 10h

3.7 BC7 - GDPR & Privacy
--------------------------

**Entites** : GdprExport (326), GdprRectification (174), GdprRestriction (215),
GdprObjection (251)

**Tests existants** : 23 unit tests, BDD existants (gdpr), E2E ``e2e_gdpr.rs``
+ ``e2e_gdpr_audit.rs``, Playwright ``Gdpr.spec.ts``

.. list-table:: BC7 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-E2E-7.1
     - Fix ``e2e_gdpr.rs`` + ``e2e_gdpr_audit.rs`` compilation
     - 1h
     - P1

**Total BC7** : 1h (deja bien couvert)

3.8 BC8 - Energy & IoT
------------------------

**Entites** : EnergyCampaign (624), EnergyBillUpload (587), IoTReading (496),
LinkyDevice (457)

**Tests existants** : 46 unit tests

.. list-table:: BC8 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-8.1
     - Step defs ``energy_campaigns.feature`` (14 scenarios) dans ``bdd_operations.rs``
     - 5h
     - P2
   * - WP-BDD-8.2
     - Step defs ``iot.feature`` (12 scenarios) dans ``bdd_operations.rs``
     - 4h
     - P2

**Total BC8** : 9h

3.9 BC9 - Security & 2FA
--------------------------

**Entites** : TwoFactorSecret (327), RefreshToken (92)

**Tests existants** : 17 unit tests

.. list-table:: BC9 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-9.1
     - Step defs ``two_factor.feature`` (12 scenarios) dans ``bdd_governance.rs``
     - 4h
     - P0

**Total BC9** : 4h

3.10 BC10 - Reporting & Analytics
----------------------------------

**Cross-context** : Utilise Expense, Account, JournalEntry

**Tests existants** : 8 unit tests (dashboard + financial_report use cases)

Couvert par WP-BDD-2.7 (dashboard.feature).

**Total BC10** : 0h (integre dans BC2)

3.11 BC11 - SaaS Administration
---------------------------------

**Entites** : Organization (259), User via admin handlers

**Tests existants** : 8 unit tests

.. list-table:: BC11 Work Packages
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-BDD-11.1
     - Step defs ``organizations.feature`` (8 scenarios) dans ``bdd_governance.rs``
     - 3h
     - P0

**Total BC11** : 3h

3.12 BC12 - Revenue Management
--------------------------------

**Entites** : CallForFunds (263), OwnerContribution (276)

Couvert par WP-BDD-2.4, WP-BDD-2.5 dans BC2 (bdd_financial.rs).

**Total BC12** : 0h (integre dans BC2)

3.13 BC13 - Public & Open
---------------------------

**Endpoint** : GET /public/buildings/:slug/syndic (sans auth)

Couvert par WP-BDD-3.3 (public_syndic.feature dans bdd_governance.rs).

**Total BC13** : 0h (integre dans BC3)

=========================================================
4. Work Packages Transverses
=========================================================

4.1 WP-INFRA : Infrastructure BDD
-----------------------------------

.. list-table:: WP Infrastructure
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-INFRA-1
     - Creer scaffolding ``bdd_financial.rs`` (World struct, imports, main, setup_database)
     - 1h
     - P0
   * - WP-INFRA-2
     - Creer scaffolding ``bdd_governance.rs``
     - 1h
     - P0
   * - WP-INFRA-3
     - Creer scaffolding ``bdd_operations.rs``
     - 1h
     - P0
   * - WP-INFRA-4
     - Creer scaffolding ``bdd_community.rs``
     - 1h
     - P0
   * - WP-INFRA-5
     - Ajouter 4 ``[[test]]`` entries dans ``Cargo.toml``
     - 0.25h
     - P0
   * - WP-INFRA-6
     - Mettre a jour ``common/mod.rs`` pour tous les repos/use_cases manquants
     - 2h
     - P0

**Total WP-INFRA** : 6.25h

4.2 WP-E2E-COMMON : Fix E2E commun
------------------------------------

.. list-table:: WP E2E Common
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-E2E-C1
     - Mettre a jour ``common/mod.rs`` setup_test_db avec tous les use_cases
     - 2h
     - P0
   * - WP-E2E-C2
     - Fix ``e2e_board.rs`` + ``e2e_board_dashboard.rs`` compilation
     - 1h
     - P1

**Total WP-E2E-COMMON** : 3h

4.3 WP-CI : Pipeline CI/CD
----------------------------

.. list-table:: WP CI
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-CI-1
     - ``ci.yml`` : ``cargo test --test 'bdd*'`` (tous les BDD)
     - 0.5h
     - P0
   * - WP-CI-2
     - ``ci.yml`` : ``cargo test --test 'e2e*'`` (tous les E2E)
     - 0.5h
     - P0
   * - WP-CI-3
     - ``ci.yml`` : Ajouter ``release/**`` aux branches trigger
     - 0.25h
     - P0
   * - WP-CI-4
     - ``ci.yml`` : Ajouter job ``playwright`` (optionnel)
     - 1h
     - P2
   * - WP-CI-5
     - ``Makefile`` : Mettre a jour test-bdd et test-e2e-backend
     - 0.5h
     - P0

**Total WP-CI** : 2.75h

4.4 WP-DOC : Documentation Transverse
---------------------------------------

.. list-table:: WP Documentation Transverse
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-DOC-T1
     - Restructurer ``CHANGELOG.md`` : [Unreleased] -> [0.5.0]
     - 3h
     - P0
   * - WP-DOC-T2
     - Valider + mettre a jour ``openapi.yaml`` vs endpoints reels
     - 2h
     - P1
   * - WP-DOC-T3
     - Mettre a jour ``docs/index.rst`` toctree avec nouvelles docs
     - 0.5h
     - P1
   * - WP-DOC-T4
     - Mettre a jour ce WBS avec progression reelle
     - 1h
     - P3

**Total WP-DOC** : 6.5h

4.5 WP-RELEASE : Mecanique Release
------------------------------------

.. list-table:: WP Release
   :header-rows: 1
   :widths: 10 50 10 10

   * - WP
     - Tache
     - Heures
     - Priorite
   * - WP-REL-1
     - Merger WIP-BACKUP -> main
     - 0.25h
     - P0
   * - WP-REL-2
     - Creer branche release/v0.5.0
     - 0.25h
     - P0
   * - WP-REL-3
     - Version bump : ``Cargo.toml`` + ``package.json`` -> 0.5.0
     - 0.25h
     - P0
   * - WP-REL-4
     - Executer pre-release checklist (fmt, clippy, audit, tests)
     - 2h
     - P0
   * - WP-REL-5
     - Git tag v0.5.0 + push + ``gh release create``
     - 0.5h
     - P0
   * - WP-REL-6
     - Ecrire RELEASE_NOTES.md pour GitHub Release
     - 1h
     - P0

**Total WP-RELEASE** : 4.25h

=========================================================
5. Matrice de Priorisation (MoSCoW)
=========================================================

.. list-table:: Priorisation MoSCoW
   :header-rows: 1
   :widths: 10 60 10 20

   * - Priorite
     - Work Packages
     - Heures
     - Critere
   * - **P0 MUST**
     - WP-INFRA (6.25h) + WP-E2E-C1 (2h) + WP-CI (2.75h) + WP-REL (4.25h) + WP-DOC-T1 (3h) + BDD Tier 1: payments (8h), tickets (6h), notifications (5h), resolutions (5h), convocations (5h), quotes (5h), 2FA (4h), orgs (3h) + E2E fix P0 (5.5h) + DOC: payments (3h), tickets (2h), notifications (2h), convocations (2h), quotes (2h)
     - **75.75h**
     - Bloquant pour release
   * - **P1 SHOULD**
     - BDD community: notices (5h), skills (4h), shared_objects (5h), bookings (5h) + E2E fix P1 (3.5h) + DOC: READMEs (4h), community (3h), resolutions (2h), openapi (2h) + PW: tickets (2h) + E2E-5.1 (0.5h)
     - **36h**
     - Forte valeur ajoutee
   * - **P2 COULD**
     - BDD: gamification (4h), energy (5h), work_reports (3h), tech_insp (4h), journal (3h), call_for_funds (3h), contributions (3h), charge_dist (2h), iot (4h) + PW: expenses (2h), meetings (2h), notifications (2h), buildings (2h), owner_dashboard (3h) + WP-CI-4 (1h)
     - **43h**
     - Ameliore couverture
   * - **P3 WONT**
     - BDD: dashboard (1h), public_syndic (1h) + WP-DOC-T4 (1h)
     - **3h**
     - Defere si temps manque
   * -
     - **TOTAL**
     - **~157.75h**
     -

=========================================================
6. Calendrier d'Execution (Sprints)
=========================================================

.. code-block:: text

   Sprint   | Semaines | Focus                              | WPs
   ---------|----------|------------------------------------|---------
   Sprint 0 |  S1      | Branches + Infrastructure          | WP-REL-1/2, WP-INFRA-*,
            |          | BDD scaffolding + E2E common fix   | WP-E2E-C1, WP-CI-*
   ---------|----------|------------------------------------|---------
   Sprint 1 |  S2-S3   | BDD Tier 1a (Governance)           | WP-BDD-3.1 (resolutions),
            |          |                                    | WP-BDD-3.2 (convocations),
            |          |                                    | WP-BDD-9.1 (2FA),
            |          |                                    | WP-BDD-11.1 (orgs)
   ---------|----------|------------------------------------|---------
   Sprint 2 |  S3-S4   | BDD Tier 1b (Operations+Finance)   | WP-BDD-4.1 (tickets),
            |          |                                    | WP-BDD-6.1 (notifications),
            |          |                                    | WP-BDD-2.1/2.2 (payments)
   ---------|----------|------------------------------------|---------
   Sprint 3 |  S4-S5   | BDD Tier 1c (Quotes) + E2E fix     | WP-BDD-4.2 (quotes),
            |          | + Documentation P0                 | WP-E2E-* P0 (all),
            |          |                                    | WP-DOC-2.1, 4.1, 4.2,
            |          |                                    | WP-DOC-6.1, 3.1
   ---------|----------|------------------------------------|---------
   Sprint 4 |  S6-S7   | BDD Tier 2 (Community)             | WP-BDD-5.1-5.4 (notices,
            |          |                                    | skills, sharing, bookings)
            |          |                                    | WP-DOC-5.1, WP-DOC-1.*
   ---------|----------|------------------------------------|---------
   Sprint 5 |  S8-S9   | BDD Tier 3 (Remaining)             | WP-BDD-5.5, 8.1, 8.2,
            |          | + Playwright                       | 4.3, 4.4, 2.3-2.7
            |          |                                    | WP-PW-* (all 7 specs)
   ---------|----------|------------------------------------|---------
   Sprint 6 |  S10     | Documentation + CHANGELOG          | WP-DOC-T1 (changelog),
            |          | + Pre-release checklist             | WP-DOC-T2 (openapi),
            |          | + Release v0.5.0                   | WP-REL-3/4/5/6

=========================================================
7. Criteres d'Acceptation (Definition of Done)
=========================================================

7.1 Par Work Package BDD
--------------------------

.. code-block:: text

   [x] Feature file .feature existe et est syntaxiquement correct
   [x] Step definitions implementees dans le fichier bdd_<domain>.rs
   [x] World struct contient les use_cases necessaires
   [x] setup_database() initialise testcontainers PostgreSQL
   [x] Tous les scenarios passent : cargo test --test bdd_<domain>
   [x] Aucun scenario en SKIPPED ou FAILED
   [x] Step defs suivent le pattern DDD (appellent use_cases, pas SQL direct)

7.2 Par Work Package E2E
--------------------------

.. code-block:: text

   [x] Fichier e2e_*.rs compile sans erreur
   [x] Tous les tests async passent avec testcontainers
   [x] Tests couvrent les happy paths + error paths
   [x] Utilise common/mod.rs pour le setup
   [x] cargo test --test e2e_<feature> passe

7.3 Par Work Package Documentation
------------------------------------

.. code-block:: text

   [x] Document RST/MD cree dans docs/
   [x] Suit le pattern existant (overview, legal context, domain model, API, tests)
   [x] Reference dans docs/index.rst toctree
   [x] Aucun lien casse
   [x] Pas de duplication avec CLAUDE.md

7.4 Par Work Package Playwright
---------------------------------

.. code-block:: text

   [x] Spec file .spec.ts cree dans frontend/tests/e2e/
   [x] Utilise data-testid selectors (pas de CSS fragile)
   [x] Setup via API (pas de navigation UI pour le setup)
   [x] npx playwright test <file> passe
   [x] Screenshots/video on failure configures

7.5 Release 0.5.0 (globale)
-----------------------------

.. code-block:: text

   [x] cargo fmt --check passe
   [x] cargo clippy -- -D warnings passe
   [x] cargo audit : 0 vulnerabilite critique
   [x] cargo test --lib : 550+ unit tests passent
   [x] cargo test --test bdd : 194 scenarios passent (legacy)
   [x] cargo test --test bdd_financial : 74 scenarios passent
   [x] cargo test --test bdd_governance : 74 scenarios passent
   [x] cargo test --test bdd_operations : 83 scenarios passent
   [x] cargo test --test bdd_community : 72 scenarios passent
   [x] cargo test --test 'e2e*' : tous compilent et passent
   [x] cd frontend && npm run build : succes
   [x] CHANGELOG.md contient section [0.5.0]
   [x] Docs features P0 creees (5 fichiers minimum)
   [x] backend/README.md et frontend/README.md existent
   [x] Version 0.5.0 dans Cargo.toml et package.json
   [x] Tag git v0.5.0 cree
   [x] GitHub Release creee avec notes

=========================================================
8. Verification SOLID par Couche
=========================================================

8.1 Checklist SOLID - Domain Layer
-----------------------------------

.. code-block:: text

   S - Chaque entite a un seul domaine de responsabilite
       [x] Building = gestion immobiliere
       [x] Payment = transactions financieres
       [x] Resolution = votes AG
       [ ] Expense (1108 LOC) -> A surveiller, potentiel split

   O - Entites extensibles sans modification
       [x] Workflows via enums (ExpenseStatus, TicketStatus, etc.)
       [x] State machines avec transitions validees
       [x] Nouveaux etats ajoutables sans casser l'existant

   L - Substitution Liskov
       [x] Toute implementation Repository substituable
       [x] MockAll pour tests unitaires

   I - Segregation interfaces
       [x] 50 ports distincts (1 trait = 1 responsabilite)
       [x] Pas de "fat interface" (max ~20 methodes/trait)

   D - Inversion dependances
       [x] Domain Layer : 0 dependances externes
       [x] Use Cases dependent de traits (pas implementations)
       [x] Handlers dependent de AppState (injection via Actix)

8.2 Checklist SOLID - Tests
-----------------------------

.. code-block:: text

   S - Chaque test verifie UNE chose
       [x] Unit tests domain : 1 assertion par test
       [x] BDD : 1 scenario = 1 comportement metier
       [x] E2E : 1 test = 1 workflow complet

   O - Tests extensibles
       [x] Nouveaux .feature sans modifier bdd.rs existant
       [x] Nouveaux e2e_*.rs sans modifier common/mod.rs

   L - Substitution
       [x] MockAll traits pour unit tests
       [x] Testcontainers PostgreSQL pour integration

   I - Segregation
       [x] BDD split en 5 fichiers par domaine (pas monolithique)
       [x] E2E 1 fichier par feature

   D - Inversion
       [x] Tests appellent use_cases (pas SQL direct)
       [x] BDD step defs utilisent le layer Application

=========================================================
9. Risques et Mitigations
=========================================================

.. list-table:: Risques
   :header-rows: 1
   :widths: 30 10 30 30

   * - Risque
     - Impact
     - Probabilite
     - Mitigation
   * - E2E compilation cascade (DTO changes)
     - HAUT
     - MOYENNE
     - Commencer par les plus recents (e2e_tickets), travailler en arriere
   * - BDD step defs plus lents que prevu
     - MOYEN
     - HAUTE
     - Use case-level BDD (pas HTTP). Commencer par le BC le plus petit.
   * - CHANGELOG restructuration perd info
     - MOYEN
     - FAIBLE
     - Garder [0.1.0-internal] avec note. Le [0.5.0] est additif.
   * - CI trop lent avec tous les tests
     - FAIBLE
     - MOYENNE
     - Paralleliser jobs BDD (4 jobs au lieu de 1)
   * - Playwright flaky (database state)
     - MOYEN
     - HAUTE
     - Setup API-based, database cleanup entre tests
   * - Scope creep (perfectionnisme tests)
     - HAUT
     - HAUTE
     - Respecter MoSCoW. P2/P3 deferes si budget depasse.

=========================================================
10. Diagramme de Dependances
=========================================================

.. code-block:: text

   Phase 0 (branches)
     |
     v
   Phase 1 (infra BDD + E2E fix common)
     |
     +------+------+------+------+
     |      |      |      |      |
     v      v      v      v      v
   Sprint1 Sprint2 Sprint3      DOC (parallele)
   (gov)  (ops)   (quotes+E2E)  |
     |      |      |             v
     +------+------+        CHANGELOG
     |                         |
     v                         v
   Sprint4 (community) ---> Sprint5 (remaining+PW)
     |                         |
     +-------------------------+
     |
     v
   Sprint6 (pre-release checklist + tag + release)

=========================================================
11. Metriques de Succes
=========================================================

.. list-table:: KPIs Release 0.5.0
   :header-rows: 1
   :widths: 40 20 20 20

   * - Metrique
     - Baseline
     - Cible 0.5.0
     - Stretch
   * - Unit tests (domain)
     - 501
     - 530+
     - 550+
   * - BDD scenarios passants
     - 194
     - 390+ (P0+P1)
     - 473 (tout)
   * - BDD scenarios skipped
     - 279
     - <83 (P2+P3)
     - 0
   * - E2E fichiers compilant
     - 4/23
     - 23/23
     - 23/23
   * - Playwright specs
     - 3
     - 5+
     - 10
   * - Docs features couvertes
     - 8/30
     - 16/30
     - 22/30
   * - CHANGELOG structure
     - [Unreleased] blob
     - [0.5.0] propre
     - + notes migration
   * - CI pipeline
     - 4 tests actifs
     - 7+ tests actifs
     - + Playwright
   * - ``cargo clippy`` warnings
     - 0
     - 0
     - 0
   * - ``cargo audit`` critiques
     - 0
     - 0
     - 0
