
KoproGo Project Structure
=========================

**Last updated**\ : 2025-10-25 20:33:29

This document provides an overview of the KoproGo project structure, automatically generated
from the actual codebase.

Root Structure
--------------

.. code-block::

   .
   ├── CHANGELOG.md
   ├── CLAUDE.md
   ├── LICENSE
   ├── Makefile
   ├── README.md
   ├── argocd
   │   ├── README.md
   │   ├── application.yaml
   │   ├── argocd-helper.sh
   │   └── docker-compose.argocd.yml
   ├── backend
   │   ├── Cargo.lock
   │   ├── Cargo.toml
   │   ├── DATABASE_CONSTRAINTS.md
   │   ├── Dockerfile
   │   ├── Dockerfile.dev
   │   ├── Dockerfile.production
   │   ├── audit.toml
   │   ├── benches
   │   ├── docs
   │   ├── migrations
   │   ├── run-realistic-seed.sh
   │   ├── src
   │   ├── tests
   │   ├── uploads
   │   └── validate_seed.sh
   ├── deploy
   │   ├── production
   │   └── staging
   ├── docker-compose.yml
   ├── docs
   │   ├── ECONOMIC_MODEL.md
   │   ├── DEPLOY_GITOPS.md
   │   ├── E2E_TESTING_GUIDE.md
   │   ├── INFRASTRUCTURE_ROADMAP.md
   │   ├── MAKEFILE_GUIDE.md
   │   ├── Makefile
   │   ├── PERFORMANCE_REPORT.md
   │   ├── PERFORMANCE_TESTING.md
   │   ├── PROJECT_STRUCTURE.md
   │   ├── README.md
   │   ├── VPS_DEPLOYMENT.md
   │   ├── _static
   │   ├── archive
   │   ├── backend
   │   ├── changelog.md
   │   ├── conf.py
   │   ├── config
   │   ├── frontend
   ...

Backend Structure (Hexagonal Architecture)
------------------------------------------

Domain Layer
^^^^^^^^^^^^

The core business logic with no external dependencies.

.. code-block::

   backend/src/domain
   ├── entities
   │   ├── building.rs
   │   ├── document.rs
   │   ├── expense.rs
   │   ├── meeting.rs
   │   ├── mod.rs
   │   ├── organization.rs
   │   ├── owner.rs
   │   ├── refresh_token.rs
   │   ├── unit.rs
   │   └── user.rs
   ├── i18n.rs
   ├── mod.rs
   └── services
       ├── expense_calculator.rs
       ├── mod.rs
       ├── pcn_exporter.rs
       └── pcn_mapper.rs

   3 directories, 16 files

**Entities**\ : 9 entities
**Services**\ : 3 domain services

Application Layer
^^^^^^^^^^^^^^^^^

Use cases and port definitions (interfaces).

.. code-block::

   backend/src/application
   ├── dto
   │   ├── auth_dto.rs
   │   ├── building_dto.rs
   │   ├── document_dto.rs
   │   ├── expense_dto.rs
   │   ├── filters.rs
   │   ├── meeting_dto.rs
   │   ├── mod.rs
   │   ├── owner_dto.rs
   │   ├── pagination.rs
   │   ├── pcn_dto.rs
   │   └── unit_dto.rs
   ├── mod.rs
   ├── ports
   │   ├── audit_log_repository.rs
   │   ├── building_repository.rs
   │   ├── document_repository.rs
   │   ├── expense_repository.rs
   │   ├── meeting_repository.rs
   │   ├── mod.rs
   │   ├── organization_repository.rs
   │   ├── owner_repository.rs
   │   ├── refresh_token_repository.rs
   │   ├── unit_repository.rs
   │   └── user_repository.rs
   └── use_cases
       ├── auth_use_cases.rs
       ├── building_use_cases.rs
       ├── document_use_cases.rs
       ├── expense_use_cases.rs
       ├── meeting_use_cases.rs
       ├── mod.rs
       ├── owner_use_cases.rs
       ├── pcn_use_cases.rs
       └── unit_use_cases.rs

   4 directories, 32 files

**Use Cases**\ : 8 use cases
**Ports**\ : 10 ports
**DTOs**\ : 10 DTOs

Infrastructure Layer
^^^^^^^^^^^^^^^^^^^^

Adapters implementing the ports.

.. code-block::

   backend/src/infrastructure
   ├── audit.rs
   ├── database
   │   ├── mod.rs
   │   ├── pool.rs
   │   ├── repositories
   │   │   ├── audit_log_repository_impl.rs
   │   │   ├── building_repository_impl.rs
   │   │   ├── document_repository_impl.rs
   │   │   ├── expense_repository_impl.rs
   │   │   ├── meeting_repository_impl.rs
   │   │   ├── mod.rs
   │   │   ├── organization_repository_impl.rs
   │   │   ├── owner_repository_impl.rs
   │   │   ├── refresh_token_repository_impl.rs
   │   │   ├── unit_repository_impl.rs
   │   │   └── user_repository_impl.rs
   │   └── seed.rs
   ├── mod.rs
   ├── storage
   │   ├── file_storage.rs
   │   └── mod.rs
   └── web
       ├── app_state.rs
       ├── handlers
       │   ├── auth_handlers.rs
       │   ├── building_handlers.rs
       │   ├── document_handlers.rs
       │   ├── expense_handlers.rs
       │   ├── health.rs
       │   ├── meeting_handlers.rs
       │   ├── mod.rs
       │   ├── organization_handlers.rs
       │   ├── owner_handlers.rs
       │   ├── pcn_handlers.rs
       │   ├── seed_handlers.rs
       │   ├── stats_handlers.rs
       │   ├── unit_handlers.rs
       │   └── user_handlers.rs
       ├── middleware.rs
       ├── mod.rs
       └── routes.rs

   6 directories, 36 files

**Repositories**\ : 10 repository implementations
**Handlers**\ : 13 HTTP handlers

Frontend Structure
------------------

.. code-block::

   frontend/src
   ├── components
   │   ├── BuildingList.svelte
   │   ├── BuildingListExample.svelte
   │   ├── DocumentList.svelte
   │   ├── ExpenseList.svelte
   │   ├── LanguageSelector.svelte
   │   ├── LoginForm.svelte
   │   ├── MeetingList.svelte
   │   ├── Navigation.svelte
   │   ├── OrganizationList.svelte
   │   ├── OwnerList.svelte
   │   ├── Pagination.svelte
   │   ├── SyncStatus.svelte
   │   ├── UnitList.svelte
   │   ├── UserListAdmin.svelte
   │   ├── admin
   │   │   └── SeedManager.svelte
   │   └── dashboards
   │       ├── AccountantDashboard.svelte
   │       ├── AdminDashboard.svelte
   │       ├── OwnerDashboard.svelte
   │       └── SyndicDashboard.svelte
   ├── layouts
   │   └── Layout.astro
   ├── lib
   │   ├── I18N_USAGE.md
   │   ├── api.ts
   │   ├── config.ts
   │   ├── db.ts
   │   ├── i18n.ts
   │   ├── sync.ts
   │   └── types.ts
   ├── locales
   │   ├── de.json
   │   ├── en.json
   │   ├── fr.json
   │   └── nl.json
   ├── pages
   │   ├── accountant
   │   │   └── index.astro
   │   ├── admin
   │   │   ├── index.astro
   │   │   ├── organizations.astro
   │   │   ├── seed.astro
   │   │   ├── subscriptions.astro
   │   │   └── users.astro
   │   ├── buildings
   │   │   └── index.astro
   │   ├── documents.astro
   │   ├── expenses.astro
   │   ├── index.astro
   │   ├── login.astro
   │   ├── meetings.astro
   │   ├── owner
   │   │   ├── contact.astro
   │   │   ├── documents.astro
   │   │   ├── expenses.astro
   │   │   ├── index.astro
   │   │   ├── profile.astro
   │   │   └── units.astro
   │   ├── owners.astro
   │   ├── profile.astro
   │   ├── reports.astro
   │   ├── settings.astro
   │   ├── syndic
   │   │   └── index.astro
   │   └── units.astro
   ├── stores
   │   └── auth.ts
   └── styles
       └── global.css

   15 directories, 57 files

Tests Structure
---------------

.. code-block::

   backend/tests
   ├── bdd.rs
   ├── e2e.rs
   ├── e2e_auth.rs
   ├── e2e_http.rs
   └── features
       ├── auth.feature
       ├── building.feature
       ├── documents.feature
       ├── documents_delete.feature
       ├── documents_expenses.feature
       ├── documents_linking.feature
       ├── expenses_pagination.feature
       ├── expenses_pcn.feature
       ├── i18n.feature
       ├── meetings.feature
       ├── meetings_manage.feature
       ├── multitenancy.feature
       └── pagination_filtering.feature

   2 directories, 17 files

Documentation Structure
-----------------------

.. code-block::

   docs
   ├── ECONOMIC_MODEL.md
   ├── DEPLOY_GITOPS.md
   ├── E2E_TESTING_GUIDE.md
   ├── INFRASTRUCTURE_ROADMAP.md
   ├── MAKEFILE_GUIDE.md
   ├── Makefile
   ├── PERFORMANCE_REPORT.md
   ├── PERFORMANCE_TESTING.md
   ├── PROJECT_STRUCTURE.md
   ├── README.md
   ├── VPS_DEPLOYMENT.md
   ├── _static
   ├── archive
   │   ├── ANALYSIS.md
   │   ├── BUSINESS_PLAN.md
   │   ├── ISSUE_004_COMPLETION_GUIDE.md
   │   ├── MARKET_ANALYSIS.md
   │   ├── NEW_ISSUES.md
   │   ├── PRIORITIES_TABLE.md
   │   ├── ROADMAP.md
   │   ├── SESSION_SUMMARY.md
   │   ├── load-tests-troubleshooting
   │   └── root-md
   ├── backend
   │   ├── benches
   │   ├── src
   │   └── tests
   ├── changelog.md
   ├── conf.py
   ├── config
   ├── frontend
   │   └── src
   ├── index.rst
   └── requirements.txt

   12 directories, 23 files

----

*This file is automatically generated by ``.claude/scripts/sync-docs-structure.sh``\ *
