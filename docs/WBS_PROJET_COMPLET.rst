=========================================================
KoproGo - WBS Projet Complet (DDD-BDD-TDD-SOLID)
=========================================================

:Version: 1.0
:Date: 26 fevrier 2026
:Methode: Domain-Driven Design + Behavior-Driven Development + Test-Driven Development + SOLID
:Auteur: Claude Code (Audit automatise)
:Statut: Document de reference technique
:Couverture: Jalon 0 (complete) -> Jalon 7 (vision long terme)

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

Le projet est decompose en **13 Bounded Contexts** (BC), chacun autonome
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
     - 5
     - 5
     - 0
     - 100%
     - COMPLETE
   * - J1 Securite & GDPR
     - 11
     - 8
     - 3
     - 73%
     - Tests E2E (#66,#69) + itsme (#48)
   * - J2 Conformite Legale Belge
     - 17
     - 17
     - 0
     - 100%
     - COMPLETE
   * - J3 Features Differenciantes
     - 8
     - 7
     - 1
     - 88%
     - PDF generation (#47)
   * - J4 Automation & Integrations
     - 14
     - 10
     - 4
     - 71%
     - WCAG (#93), RBAC (#71,#72), GDPR docs (#67)
   * - J5 Mobile & API Publique
     - 4
     - 1
     - 3
     - 25%
     - PWA (#87), BI (#97), Mobile (#98)
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
     - **65**
     - **49**
     - **16**
     - **75%**
     -

**+ 2 issues sans jalon** : #158 (E2E compilation), #206 (Frontend UI wiring)

6. Metriques Code
-------------------

.. code-block:: text

   Backend Rust   : ~87,000 LOC (domain+application+infrastructure+tests)
   Frontend Svelte: ~15,000 LOC (pages+components+lib)
   Migrations SQL : ~7,200 LOC (58 migrations)
   Documentation  : ~24,600 LOC (45 RST/MD files)
   Infrastructure : ~5,000 LOC (Ansible, Docker, CI)
   TOTAL          : ~138,800 LOC

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
     - 24 features BDD sans step definitions (279 scenarios skipped)
     - HAUT
     - 81h
     - v0.5.0
   * - DT-2
     - 19/23 E2E backend ne compilent pas (#158)
     - HAUT
     - 8h
     - v0.5.0
   * - DT-3
     - Documentation features : 8/30 couvertes (27%)
     - MOYEN
     - 20h
     - v0.5.0
   * - DT-4
     - CHANGELOG non structure (blob [Unreleased])
     - MOYEN
     - 3h
     - v0.5.0
   * - DT-5
     - Playwright couvre <5% des pages (3/80 specs)
     - MOYEN
     - 15h
     - v0.5.0
   * - DT-6
     - frontend/README.md et backend/README.md manquants
     - FAIBLE
     - 4h
     - v0.5.0
   * - DT-7
     - Use case unit tests faibles (17/45 fichiers ont des tests)
     - FAIBLE
     - 20h
     - v1.0.0
   * - DT-8
     - OpenAPI spec potentiellement desynchronisee
     - FAIBLE
     - 2h
     - v0.5.0
   * - DT-9
     - Frontend UI wiring incomplet (#206)
     - HAUT
     - 15h
     - v0.5.0
   * - DT-10
     - Hardcoded total_eligible_voters=10 dans poll_use_cases
     - FAIBLE
     - 1h
     - v0.5.0


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
9. Jalon 1 : Securite & GDPR [73% -> 100%]
=========================================================

**Debloque** : 50-100 coproprietes (beta publique)

**Release cible** : v0.5.0

**Issues ouvertes** : #69, #66, #48

9.1 WP-FEAT-J1 : Features restantes
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
     - Finaliser 2FA TOTP : verifier que setup/enable/verify/disable
       fonctionnent de bout en bout avec des vrais TOTP codes
     - #78
     - 4h
     - BC9
     - BDD: two_factor.feature deja ecrit
   * - WP-FEAT-J1.2
     - Fix E2E admin login timeout apres GDPR user logout (#66)
     - #66
     - 2h
     - BC7
     - TDD: fix e2e_gdpr.rs test sequence
   * - WP-FEAT-J1.3
     - Playwright E2E pour units + documents (#69)
     - #69
     - 4h
     - BC1
     - BDD: features existants; PW: 2 nouveaux specs
   * - WP-FEAT-J1.4
     - Authentification forte itsme/eID (etude + prototype)
     - #48
     - 8h
     - BC9
     - TDD: integration test avec mock itsme

9.2 WP-BDD-J1 : Step Definitions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature -> Step Definitions
     - Scenarios
     - Heures
   * - WP-BDD-J1.1
     - ``two_factor.feature`` -> ``bdd_governance.rs``
     - 12
     - 4h
   * - WP-BDD-J1.2
     - ``organizations.feature`` -> ``bdd_governance.rs``
     - 8
     - 3h

9.3 WP-E2E-J1 : Fix E2E
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-E2E-J1.1
     - Fix ``e2e_auth.rs`` compilation
     - 1h
   * - WP-E2E-J1.2
     - Fix ``e2e_gdpr.rs`` + ``e2e_gdpr_audit.rs`` compilation
     - 1h

9.4 WP-DOC-J1 : Documentation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-DOC-J1.1
     - Mettre a jour ``docs/SECURITY.md`` avec 2FA section
     - 1h

**Total Jalon 1 restant** : ~28h

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
11. Jalon 3 : Features Differenciantes [88% -> 100%]
=========================================================

**Debloque** : 500-1,000 coproprietes (differenciation marche)

**Release cible** : v0.5.0 (dette) + v0.6.0 (PDF)

**Issue ouverte** : #47

11.1 WP-FEAT-J3 : Features restantes
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10 10 20

   * - WP
     - Description
     - Issue
     - Heures
     - BC
     - Approche DDD
   * - WP-FEAT-J3.1
     - **PDF Generation etendue** : PV assemblee generale
       (domain service ``MeetingMinutesExporter`` existe deja,
       connecter a endpoint + template LaTeX/Typst)
     - #47
     - 8h
     - BC3
     - Domain service -> Use case -> Handler avec file download
   * - WP-FEAT-J3.2
     - **PDF** : Releve de charges annuel
       (domain service ``AnnualReportExporter`` existe deja)
     - #47
     - 5h
     - BC10
     - Idem + aggregation cross-BC2
   * - WP-FEAT-J3.3
     - **PDF** : Contrat proprietaire (attestation)
       (domain service ``OwnershipContractExporter`` existe deja)
     - #47
     - 4h
     - BC1
     - Idem
   * - WP-FEAT-J3.4
     - **PDF** : Devis travaux formatted
       (domain service ``WorkQuoteExporter`` existe deja)
     - #47
     - 4h
     - BC4
     - Idem
   * - WP-FEAT-J3.5
     - **Contractor Backoffice frontend** : Dashboard contractor
       avec work reports, photos, payment status
     - #52
     - 12h
     - BC4
     - Pages Astro + composants Svelte + API calls

11.2 WP-BDD-J3 : Step Definitions (dette)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature -> Step Definitions
     - Scenarios
     - Heures
   * - WP-BDD-J3.1
     - ``resolutions.feature`` -> ``bdd_governance.rs``
     - 14
     - 5h
   * - WP-BDD-J3.2
     - ``convocations.feature`` -> ``bdd_governance.rs``
     - 13
     - 5h
   * - WP-BDD-J3.3
     - ``quotes.feature`` -> ``bdd_governance.rs``
     - 13
     - 5h
   * - WP-BDD-J3.4
     - ``gamification.feature`` -> ``bdd_community.rs``
     - 13
     - 4h

11.3 WP-E2E-J3 : Fix E2E (dette)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-E2E-J3.1
     - Fix ``e2e_resolutions.rs`` (1,161 LOC)
     - 1h
   * - WP-E2E-J3.2
     - Fix ``e2e_convocations.rs`` (1,501 LOC)
     - 1h
   * - WP-E2E-J3.3
     - Fix ``e2e_quotes.rs`` (1,381 LOC)
     - 1h
   * - WP-E2E-J3.4
     - Fix ``e2e_local_exchange.rs`` (580 LOC)
     - 0.5h

11.4 WP-BDD-J3 NEW : BDD pour nouvelles features
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10 10

   * - WP
     - Feature
     - Scenarios
     - Heures
   * - WP-BDD-J3.5
     - ``pdf_generation.feature`` (NOUVEAU a ecrire)
     - ~8
     - 4h
   * - WP-BDD-J3.6
     - ``contractor_backoffice.feature`` (NOUVEAU a ecrire)
     - ~6
     - 3h

11.5 WP-DOC-J3
~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 10 40 10

   * - WP
     - Tache
     - Heures
   * - WP-DOC-J3.1
     - Creer ``docs/CONTRACTOR_QUOTES.rst``
     - 2h
   * - WP-DOC-J3.2
     - Creer ``docs/CONVOCATIONS_AG.rst``
     - 2h
   * - WP-DOC-J3.3
     - Creer ``docs/MEETINGS_RESOLUTIONS_VOTING.rst``
     - 2h
   * - WP-DOC-J3.4
     - Creer ``docs/COMMUNITY_FEATURES.rst`` (6 phases)
     - 3h

**Total Jalon 3** : ~90h (33h features + 42h dette test + 15h docs)

=========================================================
12. Jalon 4 : Automation & Integrations [71% -> 100%]
=========================================================

**Debloque** : 1,000-2,000 coproprietes (scalabilite)

**Release cible** : v0.7.0

**Issues ouvertes** : #67, #71, #72, #93

12.1 WP-FEAT-J4 : Features restantes
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
13. Jalon 5 : Mobile & API Publique [25% -> 100%]
=========================================================

**Debloque** : 2,000-5,000 coproprietes (expansion)

**Release cible** : v1.0.0

**Issues ouvertes** : #87, #97, #98

13.1 WP-FEAT-J5 : Features
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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

16. Matrice Releases
---------------------

.. list-table::
   :header-rows: 1
   :widths: 10 15 30 15 15 15

   * - Release
     - Jalons
     - Contenu principal
     - Tests cible
     - Effort
     - Prerequis equipe
   * - **v0.5.0**
     - J1+J2 dette
     - Fix tests, docs, changelog, 279 BDD step defs
     - 1,307 tests
     - 134h
     - Solo
   * - **v0.6.0**
     - J3 complet
     - PDF generation, contractor backoffice
     - 1,400 tests
     - 90h
     - Solo
   * - **v0.7.0**
     - J4 complet
     - WCAG AA, RBAC granulaire, nouveaux roles
     - 1,500 tests
     - 170h
     - Solo + 1 contributeur
   * - **v1.0.0**
     - J5 complet
     - PWA offline, analytics, mobile (etude)
     - 1,700 tests
     - 230h
     - 2 ETP
   * - **v2.0.0**
     - J6 complet
     - IoT platform, AI, marketplace, sustainability
     - 2,000 tests
     - 766h
     - 3-4 ETP
   * - **v3.0.0**
     - J7 complet
     - API v2, SDK, plugins, blockchain (opt.)
     - 2,200 tests
     - 750h
     - 10-15 ETP

17. Timeline Realiste (Solo Dev 10-15h/sem)
---------------------------------------------

.. code-block:: text

   2026
   |-- Mars-Mai      : v0.5.0 (dette test + docs)      ~134h  10-12 sem
   |-- Juin-Juillet  : v0.6.0 (PDF + contractor)        ~90h   6-8 sem
   |-- Aout-Novembre : v0.7.0 (WCAG + RBAC)            ~170h  12-15 sem
   |
   2027
   |-- Q1-Q2         : v1.0.0 (PWA + analytics)         ~230h  (besoin 2 ETP)
   |-- Q3-Q4         : v2.0.0 start (IoT + AI)          ~766h  (besoin 3-4 ETP)
   |
   2028
   |-- Q1-Q4         : v3.0.0 (API v2 + ecosystem)      ~750h  (besoin 10-15 ETP)

.. note::

   Cette timeline est un **scenario conservateur solo**.
   Avec communaute active (5+ contributeurs), les jalons 1-4
   peuvent etre livres en 6-9 mois au lieu de 12-15 mois.

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
   * - v0.5.0
     - 550
     - 473
     - 215
     - 64
     - 5
     - 1,307
   * - v0.6.0
     - 570
     - 495
     - 230
     - 70
     - 5
     - 1,370
   * - v0.7.0
     - 600
     - 525
     - 260
     - 85
     - 10
     - 1,480
   * - v1.0.0
     - 650
     - 560
     - 290
     - 120
     - 15
     - 1,635
   * - v2.0.0
     - 800
     - 620
     - 350
     - 160
     - 20
     - 1,950
   * - v3.0.0
     - 900
     - 680
     - 400
     - 200
     - 25
     - 2,205


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
     - v0.5.0
   * - J1
     - WP-FEAT-J1.2
     - Fix E2E admin login timeout (#66)
     - 2
     - FEAT
     - v0.5.0
   * - J1
     - WP-FEAT-J1.3
     - Playwright E2E units + documents (#69)
     - 4
     - PW
     - v0.5.0
   * - J1
     - WP-FEAT-J1.4
     - Authentification itsme/eID (#48)
     - 8
     - FEAT
     - v0.7.0
   * - J1
     - WP-BDD-J1.1
     - two_factor.feature step defs
     - 4
     - BDD
     - v0.5.0
   * - J1
     - WP-BDD-J1.2
     - organizations.feature step defs
     - 3
     - BDD
     - v0.5.0
   * - J3
     - WP-FEAT-J3.1-4
     - PDF Generation (4 types documents)
     - 21
     - FEAT
     - v0.6.0
   * - J3
     - WP-FEAT-J3.5
     - Contractor Backoffice frontend
     - 12
     - FEAT
     - v0.6.0
   * - J4
     - WP-FEAT-J4.1
     - WCAG 2.1 AA Accessibility (#93)
     - 40
     - FEAT
     - v0.7.0
   * - J4
     - WP-FEAT-J4.2
     - RBAC granulaire (#72)
     - 30
     - FEAT
     - v0.7.0
   * - J4
     - WP-FEAT-J4.3
     - Roles OrgAdmin/BuildingManager (#71)
     - 15
     - FEAT
     - v0.7.0
   * - J4
     - WP-FEAT-J4.4
     - Documentation GDPR formelle (#67)
     - 12
     - DOC
     - v0.7.0
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
     - v0.5.0
   * - TRANS
     - WP-E2E-C*
     - Fix E2E commun (common/mod.rs + board)
     - 3
     - E2E
     - v0.5.0
   * - TRANS
     - WP-CI-*
     - Pipeline CI/CD updates
     - 2.75
     - CI
     - v0.5.0
   * - TRANS
     - WP-REL-*
     - Release mechanics (tags, notes, version bump)
     - 4.25
     - REL
     - v0.5.0

B. Effort Total par Type
--------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 15 15 15 15

   * - Type
     - v0.5.0
     - v0.6.0
     - v0.7.0
     - v1.0.0
     - v2.0.0
     - **Total**
   * - FEAT (features)
     - 10h
     - 33h
     - 97h
     - 190h
     - 700h
     - **1,030h**
   * - BDD (step defs)
     - 81h
     - 7h
     - 32h
     - 12h
     - 35h
     - **167h**
   * - E2E (backend)
     - 11h
     - 2h
     - 4h
     - 5h
     - 10h
     - **32h**
   * - PW (Playwright)
     - 15h
     - 2h
     - 7h
     - 7h
     - 15h
     - **46h**
   * - DOC (docs)
     - 20h
     - 9h
     - 12h
     - 11h
     - 16h
     - **68h**
   * - INFRA/CI/REL
     - 16h
     - 3h
     - 5h
     - 5h
     - 5h
     - **34h**
   * - **Total**
     - **153h**
     - **56h**
     - **157h**
     - **230h**
     - **781h**
     - **1,377h**

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
