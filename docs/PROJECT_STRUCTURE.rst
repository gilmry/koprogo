
KoproGo Project Structure
=========================

**Last updated**\ : 2026-02-19

This document provides an overview of the KoproGo project structure, updated
to reflect the actual codebase state.

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
   │   ├── README.md
   │   ├── application.yaml
   │   ├── argocd-helper.sh
   │   └── docker-compose.argocd.yml
   ├── backend
   │   ├── Cargo.lock
   │   ├── Cargo.toml
   │   ├── DATABASE_CONSTRAINTS.md
   │   ├── Dockerfile
   │   ├── Dockerfile.dev
   │   ├── Dockerfile.production
   │   ├── audit.toml
   │   ├── benches
   │   ├── docs
   │   ├── migrations
   │   ├── src
   │   ├── tests
   │   └── uploads
   ├── deploy
   │   ├── production
   │   └── staging
   ├── docker-compose.yml
   ├── docs
   │   ├── PROJECT_STRUCTURE.rst  (this file)
   │   ├── ROADMAP_PAR_CAPACITES.rst
   │   ├── WBS_2026_02_18.rst
   │   ├── conf.py
   │   ├── index.rst
   │   └── ...
   ├── frontend
   │   └── src
   └── infrastructure
       └── ansible

Backend Structure (Hexagonal Architecture)
------------------------------------------

Domain Layer
^^^^^^^^^^^^

The core business logic with no external dependencies.

.. code-block::

   backend/src/domain
   ├── entities
   │   ├── account.rs
   │   ├── achievement.rs
   │   ├── board_decision.rs
   │   ├── board_member.rs
   │   ├── budget.rs
   │   ├── building.rs
   │   ├── call_for_funds.rs
   │   ├── challenge.rs
   │   ├── charge_distribution.rs
   │   ├── convocation.rs
   │   ├── convocation_recipient.rs
   │   ├── document.rs
   │   ├── energy_bill_upload.rs
   │   ├── energy_campaign.rs
   │   ├── etat_date.rs
   │   ├── expense.rs
   │   ├── gdpr_export.rs
   │   ├── gdpr_objection.rs
   │   ├── gdpr_rectification.rs
   │   ├── gdpr_restriction.rs
   │   ├── invoice_line_item.rs
   │   ├── iot_reading.rs
   │   ├── journal_entry.rs
   │   ├── linky_device.rs
   │   ├── local_exchange.rs
   │   ├── meeting.rs
   │   ├── mod.rs
   │   ├── notice.rs
   │   ├── notification.rs
   │   ├── organization.rs
   │   ├── owner.rs
   │   ├── owner_contribution.rs
   │   ├── owner_credit_balance.rs
   │   ├── payment.rs
   │   ├── payment_method.rs
   │   ├── payment_reminder.rs
   │   ├── poll.rs
   │   ├── poll_vote.rs
   │   ├── quote.rs
   │   ├── refresh_token.rs
   │   ├── resolution.rs
   │   ├── resource_booking.rs
   │   ├── shared_object.rs
   │   ├── skill.rs
   │   ├── technical_inspection.rs
   │   ├── ticket.rs
   │   ├── two_factor_secret.rs
   │   ├── unit.rs
   │   ├── unit_owner.rs
   │   ├── user.rs
   │   ├── user_role_assignment.rs
   │   ├── vote.rs
   │   └── work_report.rs
   ├── i18n.rs
   ├── mod.rs
   └── services
       ├── expense_calculator.rs
       ├── mod.rs
       ├── pcn_exporter.rs
       └── pcn_mapper.rs

**Entities**\ : 52+ entities (modules listed above)
**Services**\ : 3 domain services

Domain Entity Highlights
""""""""""""""""""""""""

- **account** - Belgian PCMN chart of accounts (AR 12/07/2012), ``AccountType``
- **achievement**, **user_achievement** - Gamification achievements, 8 categories, 5 tiers
- **board_decision**, **board_member** - Board governance, ``DecisionStatus``, ``BoardPosition``
- **budget** - Budget management, ``BudgetStatus``
- **building** - Core aggregate (SEO slug, syndic public info)
- **call_for_funds** - Owner contribution calls, ``CallForFundsStatus``
- **challenge**, **challenge_progress** - Time-bound gamification challenges, ``ChallengeType``
- **charge_distribution** - Expense charge distribution across units
- **convocation**, **convocation_recipient** - AG invitation system with legal deadline validation, ``ConvocationType``, ``AttendanceStatus``
- **document** - File storage, ``DocumentType``
- **energy_bill_upload**, **energy_campaign** - IoT/Linky energy management, ``EnergyType``, ``CampaignStatus``
- **etat_date** - Belgian "état des dates" compliance document, ``EtatDateStatus``
- **expense** - Charges with approval workflow (Draft → PendingApproval → Approved/Rejected)
- **gdpr_export**, **gdpr_objection**, **gdpr_rectification**, **gdpr_restriction** - GDPR Articles 15–21 compliance
- **invoice_line_item** - Multi-line invoices with Belgian VAT (6%, 12%, 21%)
- **iot_reading**, **linky_device** - IoT energy meter readings, ``MetricType``, ``DeviceType``
- **journal_entry** - Belgian double-entry bookkeeping
- **local_exchange**, **owner_credit_balance** - SEL time-based exchange system, ``ExchangeType``, ``ParticipationLevel``
- **meeting** - General assemblies, ``MeetingType``, ``MeetingStatus``
- **notice** - Community notice board, ``NoticeCategory``, ``NoticeStatus``
- **notification**, **notification_preference** - Multi-channel notifications (Email/SMS/Push/InApp), ``NotificationType``
- **organization** - Multi-tenancy root, ``SubscriptionPlan``
- **owner**, **unit_owner** - Co-owners and unit ownership relationships
- **owner_contribution** - Owner financial contributions, ``ContributionType``
- **payment**, **payment_method** - Stripe + SEPA integration, ``TransactionStatus``, ``PaymentMethodType``
- **payment_reminder** - Automated payment recovery (4 escalation levels), ``ReminderLevel``
- **poll**, **poll_vote** - Board consultation polls, ``PollType``, ``PollStatus``
- **quote** - Contractor quotes with Belgian legal compliance (3 mandatory for works >5000 EUR), ``QuoteStatus``
- **refresh_token** - JWT refresh tokens
- **resolution**, **vote** - Meeting voting system with Belgian majority types, ``MajorityType``, ``VoteChoice``
- **resource_booking** - Resource calendar booking, ``BookingStatus``, ``ResourceType``
- **shared_object** - Object sharing library, ``SharedObjectCategory``, ``ObjectCondition``
- **skill** - Skills directory, ``SkillCategory``, ``ExpertiseLevel``
- **technical_inspection** - Technical inspection records, ``InspectionType``, ``InspectionStatus``
- **ticket** - Maintenance request management, ``TicketPriority``, ``TicketCategory``, ``TicketStatus``
- **two_factor_secret** - Two-Factor Authentication (2FA) secrets
- **unit** - Lots within buildings, ``UnitType``
- **user**, **user_role_assignment** - Multi-role user system, ``UserRole``
- **work_report** - Contractor work reports, ``WorkType``, ``WarrantyType``

Application Layer
^^^^^^^^^^^^^^^^^

Use cases and port definitions (interfaces).

**Use Cases**\ : 43+ use cases
**Ports**\ : 40+ repository ports
**DTOs**\ : 40+ data transfer object modules

Use Case Modules
""""""""""""""""

- ``account_use_cases`` - Belgian PCMN accounting operations
- ``auth_use_cases`` - Authentication, JWT, multi-role login
- ``board_dashboard_use_cases`` - Board governance dashboard aggregation (``BoardDashboardUseCases``)
- ``board_decision_use_cases`` - Board decision lifecycle
- ``board_member_use_cases`` - Board member management
- ``budget_use_cases`` - Budget creation and tracking
- ``building_use_cases`` - Building CRUD, slug generation, syndic info
- ``call_for_funds_use_cases`` - Call for funds management
- ``charge_distribution_use_cases`` - Expense distribution across units
- ``convocation_use_cases`` - AG convocation workflow with legal validation
- ``dashboard_use_cases`` - Multi-role dashboard aggregation
- ``document_use_cases`` - Document upload and management
- ``energy_bill_upload_use_cases`` - Energy bill upload and parsing
- ``energy_campaign_use_cases`` - IoT/Linky energy campaign management (``CampaignStats``)
- ``etat_date_use_cases`` - Belgian état des dates generation
- ``expense_use_cases`` - Expense approval workflow
- ``financial_report_use_cases`` - Belgian balance sheet and income statement
- ``gamification_use_cases`` - Achievements, challenges, leaderboard (``AchievementUseCases``, ``ChallengeUseCases``, ``GamificationStatsUseCases``)
- ``gdpr_use_cases`` - GDPR Articles 15–21 (export, erasure, rectification, restriction, objection)
- ``iot_use_cases`` - IoT reading ingestion, Linky device management (``IoTUseCases``, ``LinkyUseCases``)
- ``journal_entry_use_cases`` - Double-entry bookkeeping journal entries
- ``local_exchange_use_cases`` - SEL exchange workflow with credit balance management
- ``meeting_use_cases`` - General assembly management
- ``notice_use_cases`` - Community notice board (``NoticeStatistics``)
- ``notification_use_cases`` - Multi-channel notification dispatch and preferences
- ``owner_contribution_use_cases`` - Owner financial contribution tracking
- ``owner_use_cases`` - Owner CRUD and unit relationships
- ``payment_method_use_cases`` - Payment method management (Stripe/SEPA)
- ``payment_reminder_use_cases`` - Automated payment recovery escalation
- ``payment_use_cases`` - Payment transaction lifecycle
- ``pcn_use_cases`` - Belgian PCN (Plan Comptable Normalisé) export
- ``poll_use_cases`` - Board poll creation, voting, and results
- ``quote_use_cases`` - Contractor quote comparison with Belgian legal scoring
- ``resolution_use_cases`` - Meeting resolutions with majority calculation (``VoteStatistics``)
- ``resource_booking_use_cases`` - Resource calendar booking
- ``shared_object_use_cases`` - Object sharing library management
- ``skill_use_cases`` - Community skills directory
- ``technical_inspection_use_cases`` - Technical inspection records
- ``ticket_use_cases`` - Maintenance ticket workflow (``TicketStatistics``)
- ``two_factor_use_cases`` - 2FA setup, verification, and disable
- ``unit_owner_use_cases`` - Unit ownership with Belgian percentage validation
- ``unit_use_cases`` - Unit (lot) management
- ``work_report_use_cases`` - Contractor work report management

Infrastructure Layer
^^^^^^^^^^^^^^^^^^^^

Adapters implementing the ports.

**Repositories**\ : 40+ PostgreSQL repository implementations
**Handlers**\ : 51+ HTTP handler modules

HTTP Handler Modules
""""""""""""""""""""

- ``account_handlers`` - Belgian PCMN accounts + seed endpoint
- ``admin_gdpr_handlers`` - Admin GDPR audit log management
- ``auth_handlers`` - Login, logout, token refresh, role switch, profile
- ``board_decision_handlers`` - Board decision CRUD and lifecycle
- ``board_member_handlers`` - Board member management
- ``budget_handlers`` - Budget CRUD and tracking
- ``building_handlers`` - Building CRUD with syndic public info
- ``call_for_funds_handlers`` - Call for funds management
- ``charge_distribution_handlers`` - Charge distribution per unit
- ``convocation_handlers`` - AG convocation, scheduling, sending, tracking
- ``dashboard_handlers`` - Multi-role dashboard data endpoints
- ``document_handlers`` - Document upload, download, listing
- ``energy_bill_upload_handlers`` - Energy bill upload (multipart)
- ``energy_campaign_handlers`` - IoT/Linky energy campaign management
- ``etat_date_handlers`` - Belgian état des dates generation
- ``expense_handlers`` - Expense CRUD with approval workflow
- ``financial_report_handlers`` - Belgian balance sheet and income statement
- ``financial_report_handlers_building`` - Per-building financial reports
- ``gamification_handlers`` - Achievements, challenges, leaderboard (22 endpoints)
- ``gdpr_handlers`` - GDPR self-service (export, erasure, rectification, restriction, marketing opt-out)
- ``health`` - Health check endpoint
- ``iot_handlers`` - IoT readings and Linky device management
- ``journal_entry_handlers`` - Double-entry bookkeeping journal entries
- ``local_exchange_handlers`` - SEL exchange marketplace (17 endpoints)
- ``meeting_handlers`` - General assembly management
- ``metrics`` - Prometheus metrics endpoint
- ``notice_handlers`` - Community notice board
- ``notification_handlers`` - Notifications (my, unread, read, preferences)
- ``organization_handlers`` - Organization management
- ``owner_contribution_handlers`` - Owner contribution management
- ``owner_handlers`` - Owner CRUD and unit relationships
- ``payment_handlers`` - Payment transactions (Stripe + SEPA)
- ``payment_method_handlers`` - Payment method management
- ``payment_reminder_handlers`` - Automated payment recovery escalation
- ``pcn_handlers`` - Belgian PCN export
- ``poll_handlers`` - Board polls (voting, results, lifecycle)
- ``public_handlers`` - Public syndic info (no authentication required)
- ``quote_handlers`` - Contractor quotes with comparison scoring
- ``resolution_handlers`` - Meeting resolutions and voting
- ``resource_booking_handlers`` - Resource calendar booking
- ``seed_handlers`` - Development data seeding
- ``shared_object_handlers`` - Object sharing library
- ``skill_handlers`` - Community skills directory
- ``stats_handlers`` - System-wide statistics
- ``technical_inspection_handlers`` - Technical inspection records
- ``ticket_handlers`` - Maintenance ticket workflow (17 endpoints)
- ``two_factor_handlers`` - 2FA setup and verification
- ``unit_handlers`` - Unit (lot) CRUD
- ``unit_owner_handlers`` - Unit ownership relationships and transfer
- ``user_handlers`` - User management (admin)
- ``work_report_handlers`` - Contractor work reports

New Capabilities Added Since Initial Release
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The following major capabilities have been added beyond the original MVP:

- **IoT / Linky Integration** - Energy meter readings, Linky device management, energy campaign tracking (``iot_reading``, ``linky_device``, ``energy_campaign``, ``energy_bill_upload``)
- **Two-Factor Authentication (2FA)** - TOTP-based 2FA setup and verification (``two_factor_secret``, ``two_factor_use_cases``)
- **Gamification & Achievements** - 8 categories, 5 tiers, challenges, leaderboard (Issue #49 Phase 6)
- **SEL - Systeme d'Echange Local** - Time-based local exchange trading system with credit balances, ratings, and leaderboard (Issue #49 Phase 1)
- **Community Notice Board** - Building notice board with categories and lifecycle (Issue #49 Phase 2)
- **Skills Directory** - Community skills exchange directory (Issue #49 Phase 3)
- **Object Sharing Library** - Shared object management with loan workflow (Issue #49 Phase 4)
- **Resource Booking Calendar** - Shared resource reservation system (Issue #49 Phase 5)
- **GDPR Compliance (Articles 15-21)** - Full GDPR: right to access, erasure, rectification, restriction, and objection to marketing (Issue #90)
- **Belgian Accounting (PCMN)** - Full Plan Comptable Minimum Normalise belge with ~90 pre-seeded accounts and financial reports (Issue #79)
- **Meeting Voting System** - Resolutions with Simple/Absolute/Qualified majority types, tantiemes/milliemes, proxy votes (Issue #46)
- **Maintenance Ticket System** - Maintenance request workflow with priorities, contractor assignment, and statistics (Issue #85)
- **Multi-Channel Notifications** - Email, SMS, Push, and In-App notifications with user preferences (Issue #86)
- **Payment Integration** - Stripe Payment Intents and SEPA Direct Debit with idempotency and refund support (Issue #84)
- **Convocation System** - Automatic AG invitation system with Belgian legal deadline validation and email tracking (Issue #88)
- **Contractor Quotes Module** - Belgian 3-quote mandatory requirement with automatic scoring algorithm (Issue #91)
- **Board Decision Polls** - Quick owner consultations between general assemblies, 4 poll types (Issue #51)
- **Public Syndic Information** - SEO-friendly public syndic contact page (no authentication), Belgian legal compliance (Issue #92)
- **Board Dashboard** - Board member governance dashboard with decision and meeting aggregation
- **Belgian Budgets & Etat des Dates** - Budget tracking and Belgian legal compliance documents
- **Payment Recovery Workflow** - Automated 4-level escalation (Gentle to Formal to FinalNotice to LegalAction)
- **Journal Entries** - Double-entry bookkeeping journal entries
- **Technical Inspections** - Technical inspection record management
- **Work Reports** - Contractor work report management
- **Call for Funds** - Owner contribution call management
- **Charge Distribution** - Expense charge distribution per unit

Frontend Structure
------------------

Stack: Astro (SSG) + Svelte (interactive islands) + Tailwind CSS

.. code-block::

   frontend/src
   ├── components
   │   ├── BoardDashboard.svelte
   │   ├── BoardMemberList.svelte
   │   ├── BuildingDetail.svelte
   │   ├── BuildingFinancialReports.svelte
   │   ├── BuildingList.svelte
   │   ├── BuildingSelector.svelte
   │   ├── CallForFundsForm.svelte
   │   ├── CallForFundsList.svelte
   │   ├── DecisionTracker.svelte
   │   ├── DocumentList.svelte
   │   ├── DocumentUploadModal.svelte
   │   ├── ExpenseDetail.svelte
   │   ├── ExpenseList.svelte
   │   ├── FinancialReports.svelte
   │   ├── GdprDataPanel.svelte
   │   ├── InvoiceForm.svelte
   │   ├── InvoiceList.svelte
   │   ├── InvoiceWorkflow.svelte
   │   ├── JournalEntryForm.svelte
   │   ├── LoginForm.svelte
   │   ├── McpChatbot.svelte
   │   ├── MeetingDetail.svelte
   │   ├── MeetingList.svelte
   │   ├── Navigation.svelte
   │   ├── OrganizationList.svelte
   │   ├── OwnerContributionForm.svelte
   │   ├── OwnerContributionList.svelte
   │   ├── OwnerList.svelte
   │   ├── OwnerUnits.svelte
   │   ├── PWAInstallPrompt.svelte
   │   ├── Pagination.svelte
   │   ├── PaymentReminderDetail.svelte
   │   ├── PaymentReminderList.svelte
   │   ├── ProfilePanel.svelte
   │   ├── RegisterForm.svelte
   │   ├── RouteGuard.svelte
   │   ├── SessionManager.svelte
   │   ├── SyncStatus.svelte
   │   ├── UnitList.svelte
   │   ├── UnitOwners.svelte
   │   ├── UserListAdmin.svelte
   │   ├── UserOwnerLinkManager.svelte
   │   ├── admin/
   │   ├── bookings/
   │   ├── budgets/
   │   ├── convocations/
   │   ├── dashboards/
   │   │   ├── AccountantDashboard.svelte
   │   │   ├── AdminDashboard.svelte
   │   │   ├── OwnerDashboard.svelte
   │   │   └── SyndicDashboard.svelte
   │   ├── energy-campaigns/
   │   └── etats-dates/
   ├── layouts
   │   └── Layout.astro
   ├── lib
   │   ├── api.ts
   │   ├── config.ts
   │   ├── i18n.ts
   │   ├── sync.ts
   │   └── types.ts
   ├── locales
   │   ├── de.json
   │   ├── en.json
   │   ├── fr.json
   │   └── nl.json
   ├── pages
   │   ├── accountant/
   │   ├── admin/
   │   ├── board-dashboard.astro
   │   ├── bookings.astro
   │   ├── budget-detail.astro
   │   ├── budgets.astro
   │   ├── building-detail.astro
   │   ├── buildings/
   │   ├── call-for-funds.astro
   │   ├── convocation-detail.astro
   │   ├── convocations.astro
   │   ├── documents.astro
   │   ├── energy-campaigns.astro
   │   ├── etat-date-detail.astro
   │   ├── etats-dates.astro
   │   ├── exchange-detail.astro
   │   ├── exchanges.astro
   │   ├── expense-detail.astro
   │   ├── expenses.astro
   │   ├── gamification.astro
   │   ├── index.astro
   │   ├── inspections.astro
   │   ├── invoice-workflow.astro
   │   ├── journal-entries.astro
   │   ├── login.astro
   │   ├── mcp-chat.astro
   │   ├── meeting-detail.astro
   │   ├── meetings.astro
   │   ├── mentions-legales.astro
   │   ├── notice-detail.astro
   │   ├── notices.astro
   │   ├── notifications.astro
   │   ├── owner/
   │   ├── owner-contributions.astro
   │   ├── owners.astro
   │   ├── payment-reminder-detail.astro
   │   ├── payment-reminders.astro
   │   ├── polls/
   │   ├── polls.astro
   │   ├── profile.astro
   │   ├── quotes/
   │   ├── register.astro
   │   ├── reports.astro
   │   ├── settings.astro
   │   ├── sharing.astro
   │   ├── skills.astro
   │   ├── syndic/
   │   ├── ticket-detail.astro
   │   ├── tickets.astro
   │   ├── units.astro
   │   └── work-reports.astro
   ├── stores
   │   └── auth.ts
   └── styles
       └── global.css

Tests Structure
---------------

.. code-block::

   backend/tests
   ├── bdd.rs
   ├── common/
   ├── e2e.rs
   ├── e2e_auth.rs
   ├── e2e_board.rs
   ├── e2e_board_dashboard.rs
   ├── e2e_budget.rs
   ├── e2e_convocations.rs
   ├── e2e_documents.rs
   ├── e2e_etat_date.rs
   ├── e2e_gdpr.rs
   ├── e2e_gdpr_audit.rs
   ├── e2e_http.rs
   ├── e2e_local_exchange.rs
   ├── e2e_meetings.rs
   ├── e2e_notifications.rs
   ├── e2e_payment_recovery.rs
   ├── e2e_payments.rs
   ├── e2e_quotes.rs
   ├── e2e_resolutions.rs
   ├── e2e_tickets.rs
   ├── e2e_unit_owner.rs
   ├── integration_unit_owner.rs
   ├── storage_s3.rs
   └── features
       ├── auth.feature
       ├── building.feature
       ├── documents.feature
       ├── expenses_pagination.feature
       ├── expenses_pcn.feature
       ├── i18n.feature
       ├── meetings.feature
       ├── multitenancy.feature
       ├── pagination_filtering.feature
       ├── polls.feature
       └── ...

Documentation Structure
-----------------------

.. code-block::

   docs
   ├── PROJECT_STRUCTURE.rst  (this file)
   ├── ROADMAP_PAR_CAPACITES.rst
   ├── WBS_2026_02_18.rst
   ├── BELGIAN_ACCOUNTING_PCMN.rst
   ├── INVOICE_WORKFLOW.rst
   ├── PAYMENT_RECOVERY_WORKFLOW.rst
   ├── MULTI_OWNER_SUPPORT.md
   ├── MULTI_ROLE_SUPPORT.md
   ├── GIT_HOOKS.rst
   ├── conf.py
   ├── index.rst
   ├── requirements.txt
   ├── _static/
   ├── archive/
   └── github-export/

----

*This document was last updated manually on 2026-02-19.*
*Source of truth: ``backend/src/domain/entities/mod.rs``, ``backend/src/infrastructure/web/handlers/mod.rs``, ``backend/src/application/use_cases/mod.rs``*
