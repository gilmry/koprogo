# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🗺️ Roadmap

**📅 For the complete development roadmap (Nov 2025 - Aug 2026), see [ROADMAP.md](docs/ROADMAP.md)**

The roadmap covers:
- **Phase 1 (VPS MVP)**: Security, GDPR, Backups, Board Tools (Nov 2025 - Feb 2026)
- **Phase 2 (K3s)**: Voting, Community Features, Contractor Backoffice (Mar - May 2026)
- **Phase 3 (K8s)**: Performance, Real-time, Mobile App (Jun - Aug 2026)
- Infrastructure progression: VPS (Docker Compose) → K3s → K8s
- All issues tracked in [GitHub Projects](https://github.com/users/gilmry/projects)

## Project Overview

KoproGo is a SaaS property management platform built with **Hexagonal Architecture** (Ports & Adapters) and **Domain-Driven Design (DDD)**. The system emphasizes performance (P99 < 5ms latency), testability, security (GDPR compliant), and ecological sustainability (< 0.5g CO2/request target).

**Stack**: Rust + Actix-web (backend), Astro + Svelte (frontend), PostgreSQL 15

## Security & Monitoring

KoproGo includes production-grade security and observability:

**Implemented (Issues #39, #40, #41, #43, #78):**
- ✅ **LUKS Encryption at Rest**: Full-disk encryption for PostgreSQL data and uploads (AES-XTS-512)
- ✅ **Encrypted Backups**: Daily GPG-encrypted backups with S3 off-site storage (7d local, configurable S3 lifecycle)
- ✅ **Monitoring Stack**: Prometheus + Grafana + Loki + Alertmanager (30d metrics, 7d logs)
- ✅ **Intrusion Detection**: Suricata IDS with custom rules (SQL injection, XSS, path traversal, etc.)
- ✅ **WAF Protection**: CrowdSec community threat intelligence
- ✅ **fail2ban**: Custom jails for SSH, Traefik, API abuse, PostgreSQL brute-force
- ✅ **SSH Hardening**: Key-only authentication, modern ciphers, reduced attack surface
- ✅ **Kernel Hardening**: sysctl security configuration (SYN cookies, IP spoofing protection, ASLR)
- ✅ **Security Auditing**: Automated Lynis audits (weekly), rkhunter scans (daily), AIDE file integrity monitoring
- ✅ **Application Security Headers**: HSTS (1 year), CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
- ✅ **Login Rate Limiting**: 5 attempts per 15 minutes per IP (anti-brute-force protection)
- ✅ **Environment Validation**: JWT secret strength enforcement (min 32 chars), CORS origin validation (no wildcards)

**Monitoring Endpoints:**
- Prometheus: http://vps-ip:9090
- Grafana: http://vps-ip:3001
- Alertmanager: http://vps-ip:9093
- Backend metrics: http://vps-ip:8080/metrics

**Documentation:** See [`infrastructure/SECURITY.md`](infrastructure/SECURITY.md) for complete setup and configuration.

**Quick deploy:**
```bash
cd infrastructure/ansible
ansible-playbook -i inventory.ini security-monitoring.yml
```

## Architecture: Hexagonal (Ports & Adapters)

The backend follows strict layering with dependency inversion:

```
Domain (Core)
  ↑ defines interfaces
Application (Use Cases + Ports)
  ↑ implements ports
Infrastructure (Adapters: Web, Database)
```

### Layer Rules (CRITICAL)

1. **Domain Layer** (`backend/src/domain/`):
   - Pure business logic, NO external dependencies
   - Contains `entities/` (aggregates with invariant validation) and `services/` (domain services)
   - Entities enforce business rules in constructors (e.g., `Building::new()` validates name is non-empty)
   - All domain tests are in-module `#[cfg(test)]` blocks

2. **Application Layer** (`backend/src/application/`):
   - `ports/`: Trait definitions (interfaces) like `BuildingRepository`
   - `use_cases/`: Orchestration logic (e.g., `BuildingUseCases`)
   - `dto/`: Data Transfer Objects for API contracts
   - Depends ONLY on Domain layer

3. **Infrastructure Layer** (`backend/src/infrastructure/`):
   - `database/repositories/`: PostgreSQL implementations of ports (e.g., `PostgresBuildingRepository`)
   - `web/handlers/`: Actix-web HTTP handlers
   - `web/routes.rs`: API route configuration
   - Depends on Application layer (implements ports)

### Key Pattern Example

```rust
// Domain: backend/src/domain/entities/building.rs
impl Building {
    pub fn new(...) -> Result<Self, String> {
        // Business invariants enforced here
    }
}

// Application: backend/src/application/ports/building_repository.rs
#[async_trait]
pub trait BuildingRepository: Send + Sync {
    async fn create(&self, building: &Building) -> Result<Building, String>;
}

// Infrastructure: backend/src/infrastructure/database/repositories/building_repository_impl.rs
impl BuildingRepository for PostgresBuildingRepository {
    async fn create(&self, building: &Building) -> Result<Building, String> {
        // PostgreSQL implementation
    }
}
```

When adding new features:
- Start with Domain entity/service
- Define Application port (trait)
- Create Use Case
- Implement Infrastructure adapter
- Add Web handler

## Commands

### Development Setup

```bash
# Start PostgreSQL only
make docker-up

# Copy environment files
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env

# Run migrations
make migrate

# Start backend (localhost:8080)
cd backend && cargo run

# Start frontend (localhost:3000)
cd frontend && npm install && npm run dev
```

**Important**: Use `make dev` for backend with auto-reload (requires `cargo-watch`), or `make dev-all` to start all services via Docker Compose.

### Testing (Pyramid Strategy)

```bash
# Unit tests (domain logic, 100% coverage target)
cargo test --lib

# Single test
cargo test --lib test_create_building_success

# Integration tests (with testcontainers)
cargo test --test integration
# OR: make test-integration

# BDD tests (Cucumber/Gherkin)
cargo test --test bdd
# Features in: backend/tests/features/*.feature

# E2E tests (full API)
cargo test --test e2e

# Benchmarks (Criterion)
cargo bench

# All tests
make test

# Coverage report (uses tarpaulin)
make coverage
# Output: coverage/index.html
```

**Test Structure**:
- Unit tests: In-module `#[cfg(test)]` blocks
- Integration: `backend/tests/integration/`
- BDD: `backend/tests/bdd.rs` + `backend/tests/features/`
- E2E: `backend/tests/e2e/`
- Benchmarks: `backend/benches/`

### Code Quality

```bash
# Format
cargo fmt                    # Backend
npm run format              # Frontend (in frontend/)
make format                 # Both

# Lint
cargo clippy -- -D warnings  # Backend
make lint                    # Backend + frontend build check

# Security audit
make audit
```

### Database

```bash
# Run migrations
cd backend && sqlx migrate run
# OR: make migrate

# Create new migration
cd backend && sqlx migrate add <name>

# Seed test data (if bin exists)
cargo run --bin seed
```

### GitHub Project Management Export

```bash
# Export GitHub data (issues, milestones, projects) to RST
make docs-export-github
# Output: docs/github-export/

# Exported data includes:
# - All issues (organized by phase, priority, label)
# - All milestones with their issues
# - GitHub Projects overview
# - Labels catalog
```

**Usage for Claude Code Web**:
Claude Code Web doesn't have direct access to GitHub API. The RST export allows it to access all project management data by simply cloning the repository:

```bash
git clone https://github.com/gilmry/koprogo.git
# Browse docs/github-export/ for complete project info
```

The export is integrated into Sphinx documentation under "GitHub Project Management" section.

### Build & Deploy

```bash
# Release build (with LTO optimization)
cargo build --release

# Docker
docker-compose up           # All services
docker-compose up postgres  # PostgreSQL only
docker-compose logs -f      # Follow logs

# Build images
make docker-build
```

## API Endpoints

Base URL: `http://localhost:8080/api/v1`

**Buildings**: `/buildings` (GET, POST), `/buildings/:id` (GET, PUT, DELETE)
**Units**: `/units` (GET, POST), `/buildings/:id/units` (GET)
**Unit owners**:
   - `GET /units/:id/owners` (actifs), `GET /units/:id/owners/history`
   - `GET /units/:id/owners/total-percentage`
   - `POST /units/:id/owners`, `DELETE /units/:unit_id/owners/:owner_id`
   - `PUT /unit-owners/:relationship_id` (quote-part ou contact principal)
   - `POST /units/:unit_id/owners/transfer`
**User roles**:
   - `POST /auth/login` (retourne `roles[]`, `active_role`)
   - `POST /auth/switch-role` (sélectionne le rôle actif)
   - `GET /auth/me` (profil enrichi)
**Owners**: `/owners` (GET, POST), `/owners/:id` (GET), `/owners/:id/units`, `/owners/:id/units/history`
**Expenses**: `/expenses` (GET, POST), `/buildings/:id/expenses` (GET), `/expenses/:id/mark-paid` (PUT)
   - **✅ NOUVEAU**: `/expenses/:id/submit-for-approval`, `/expenses/:id/approve`, `/expenses/:id/reject` (workflow)
**✅ NOUVEAU: Accounts (PCMN)**: `/accounts` (GET, POST), `/accounts/:id` (GET, PUT, DELETE), `/accounts/code/:code`, `/accounts/seed/belgian-pcmn`
**✅ NOUVEAU: Financial Reports**: `/reports/balance-sheet`, `/reports/income-statement`
**✅ NOUVEAU: Payment Reminders**: `/payment-reminders` (GET, POST), `/payment-reminders/:id` (GET, PUT, DELETE)
   - `/payment-reminders/:id/mark-sent`, `/payment-reminders/:id/escalate`, `/payment-reminders/stats`
   - `/expenses/:id/payment-reminders`, `/owners/:id/payment-reminders`
**✅ NOUVEAU: Resolutions & Voting** (Issue #46 - Phase 2 - Belgian Copropriété Law):
   - `POST /meetings/:id/resolutions` - Create resolution
   - `GET /resolutions/:id` - Get resolution details
   - `GET /meetings/:id/resolutions` - List meeting resolutions
   - `DELETE /resolutions/:id` - Delete resolution
   - `POST /resolutions/:id/vote` - Cast vote (Pour/Contre/Abstention)
   - `GET /resolutions/:id/votes` - List resolution votes
   - `PUT /votes/:id` - Change vote
   - `PUT /resolutions/:id/close` - Close voting & calculate result (Simple/Absolute/Qualified majority)
   - `GET /meetings/:id/vote-summary` - Get vote summary for meeting
**✅ NOUVEAU: Tickets (Maintenance Requests)** (Issue #85 - Phase 2):
   - `POST /tickets` - Create maintenance ticket
   - `GET /tickets/:id` - Get ticket details
   - `GET /buildings/:id/tickets` - List building tickets
   - `GET /organizations/:id/tickets` - List organization tickets
   - `GET /tickets/my` - List my tickets (requester)
   - `GET /tickets/assigned` - List assigned tickets (contractor)
   - `GET /tickets/status/:status` - List by status (Open/Assigned/InProgress/Resolved/Closed/Cancelled)
   - `DELETE /tickets/:id` - Delete ticket
   - `PUT /tickets/:id/assign` - Assign to contractor
   - `PUT /tickets/:id/start` - Start work
   - `PUT /tickets/:id/resolve` - Mark resolved
   - `PUT /tickets/:id/close` - Close ticket
   - `PUT /tickets/:id/cancel` - Cancel ticket
   - `PUT /tickets/:id/reopen` - Reopen ticket
   - `GET /tickets/statistics` - Get statistics
   - `GET /tickets/overdue` - List overdue tickets
**✅ NOUVEAU: Notifications (Multi-Channel System)** (Issue #86 - Phase 2):
   - `POST /notifications` - Create notification
   - `GET /notifications/:id` - Get notification
   - `GET /notifications/my` - List my notifications
   - `GET /notifications/unread` - List unread notifications
   - `PUT /notifications/:id/read` - Mark as read
   - `PUT /notifications/read-all` - Mark all as read
   - `DELETE /notifications/:id` - Delete notification
   - `GET /notifications/stats` - Get notification statistics
   - `GET /notification-preferences/:user_id` - Get all user preferences
   - `GET /notification-preferences/:user_id/:notification_type` - Get specific preference
   - `PUT /notification-preferences/:user_id/:notification_type` - Update preference
**✅ NOUVEAU: Payments (Stripe + SEPA Integration)** (Issue #84 - Phase 2):
   - `POST /payments` - Create payment
   - `GET /payments/:id` - Get payment
   - `GET /payments/stripe/:stripe_payment_intent_id` - Get by Stripe intent ID
   - `GET /owners/:id/payments` - List owner payments
   - `GET /buildings/:id/payments` - List building payments
   - `GET /expenses/:id/payments` - List expense payments
   - `GET /organizations/:id/payments` - List organization payments
   - `GET /payments/status/:status` - List by status (Pending/Processing/RequiresAction/Succeeded/Failed/Cancelled/Refunded)
   - `GET /payments/pending` - List pending payments
   - `GET /payments/failed` - List failed payments
   - `PUT /payments/:id/processing` - Mark as processing
   - `PUT /payments/:id/requires-action` - Mark as requires action
   - `PUT /payments/:id/succeeded` - Mark as succeeded
   - `PUT /payments/:id/failed` - Mark as failed
   - `PUT /payments/:id/cancelled` - Mark as cancelled
   - `POST /payments/:id/refund` - Process refund
   - `DELETE /payments/:id` - Delete payment
   - `GET /owners/:id/payments/stats` - Owner payment statistics
   - `GET /buildings/:id/payments/stats` - Building payment statistics
   - `GET /expenses/:id/payments/total` - Expense total paid
   - `GET /owners/:id/payments/total` - Owner total paid
   - `GET /buildings/:id/payments/total` - Building total paid
**✅ NOUVEAU: Payment Methods** (Issue #84 - Phase 2):
   - `POST /payment-methods` - Create payment method
   - `GET /payment-methods/:id` - Get payment method
   - `GET /payment-methods/stripe/:stripe_payment_method_id` - Get by Stripe ID
   - `GET /owners/:id/payment-methods` - List owner payment methods
   - `GET /owners/:id/payment-methods/active` - List active payment methods
   - `GET /owners/:id/payment-methods/default` - Get default payment method
   - `GET /organizations/:id/payment-methods` - List organization payment methods
   - `GET /owners/:id/payment-methods/type/:method_type` - List by type (Card/SepaDebit/BankTransfer/Cash)
   - `PUT /payment-methods/:id` - Update payment method
   - `PUT /payment-methods/:id/set-default` - Set as default
   - `PUT /payment-methods/:id/deactivate` - Deactivate payment method
   - `PUT /payment-methods/:id/reactivate` - Reactivate payment method
   - `DELETE /payment-methods/:id` - Delete payment method
   - `GET /owners/:id/payment-methods/count` - Count active payment methods
   - `GET /owners/:id/payment-methods/has-active` - Check if has active payment methods
**✅ NOUVEAU: Quotes (Contractor Quotes Module)** (Issue #91 - Phase 2 - Belgian Legal Compliance):
   - `POST /quotes` - Create quote request (Syndic action)
   - `GET /quotes/:id` - Get quote details
   - `GET /buildings/:building_id/quotes` - List building quotes
   - `GET /contractors/:contractor_id/quotes` - List contractor quotes
   - `GET /buildings/:building_id/quotes/status/:status` - List by status (Requested/Received/UnderReview/Accepted/Rejected/Expired/Withdrawn)
   - `POST /quotes/:id/submit` - Contractor submits quote with pricing
   - `POST /quotes/:id/review` - Syndic starts review (Received → UnderReview)
   - `POST /quotes/:id/accept` - Accept quote (decision audit trail)
   - `POST /quotes/:id/reject` - Reject quote (decision audit trail)
   - `POST /quotes/:id/withdraw` - Contractor withdraws quote
   - `POST /quotes/compare` - Compare multiple quotes (Belgian law: 3 quotes minimum for works >5000€, automatic scoring: price 40%, delay 30%, warranty 20%, reputation 10%)
   - `PUT /quotes/:id/contractor-rating` - Update contractor rating (0-100)
   - `DELETE /quotes/:id` - Delete quote
   - `GET /buildings/:building_id/quotes/count` - Count building quotes
   - `GET /buildings/:building_id/quotes/status/:status/count` - Count quotes by status
**✅ NOUVEAU: Convocations (Automatic AG Invitations)** (Issue #88 - Phase 2):
   - `POST /convocations` - Create convocation with legal deadline validation (15d ordinary, 8d extraordinary)
   - `GET /convocations/:id` - Get convocation details
   - `GET /convocations/meeting/:meeting_id` - Get convocation by meeting
   - `GET /buildings/:id/convocations` - List building convocations
   - `GET /organizations/:id/convocations` - List organization convocations
   - `DELETE /convocations/:id` - Delete convocation
   - `PUT /convocations/:id/schedule` - Schedule send date (validates before legal deadline)
   - `POST /convocations/:id/send` - Send to owners (generates PDF, creates recipients, triggers emails)
   - `PUT /convocations/:id/cancel` - Cancel convocation
   - `GET /convocations/:id/recipients` - List all recipients with tracking data
   - `GET /convocations/:id/tracking-summary` - Get aggregate statistics (opening rate, attendance rate)
   - `PUT /convocation-recipients/:id/email-opened` - Mark email opened (tracking pixel endpoint)
   - `PUT /convocation-recipients/:id/attendance` - Update attendance (Pending → WillAttend/WillNotAttend → Attended/DidNotAttend)
   - `PUT /convocation-recipients/:id/proxy` - Set proxy delegation (Belgian "procuration")
   - `POST /convocations/:id/reminders` - Send J-3 reminders to unopened emails
**✅ NOUVEAU: GDPR Complementary Articles** (Issue #90 - Phase 2):
   - `GET /gdpr/export` - Export user data (Article 15: Right to Access)
   - `DELETE /gdpr/erase` - Anonymize user data (Article 17: Right to Erasure)
   - `GET /gdpr/can-erase` - Check erasure eligibility (legal holds)
   - `PUT /gdpr/rectify` - Correct personal data (Article 16: Right to Rectification)
   - `PUT /gdpr/restrict-processing` - Limit data processing (Article 18: Right to Restriction)
   - `PUT /gdpr/marketing-preference` - Opt-out marketing (Article 21: Right to Object)
**✅ NOUVEAU: Public Syndic Information** (Issue #92 - Phase 2 - Belgian Legal Requirement):
   - `GET /public/buildings/:slug/syndic` - Get public syndic contact info (no authentication required)
**Health**: `/health` (GET)

## Domain Entities

The system models property management with these aggregates:

- **Building**: Main aggregate (name, address, total_units, construction_year)
- **Unit**: Lots within buildings (unit_number, floor, area, liens `unit_owners`)
- **Owner**: Co-owners (name, email, phone, GDPR-sensitive data)
- **UnitOwner**: Relation d'appartenance (pourcentage, temporalité, contact principal)
- **UserRoleAssignment**: Rôle utilisateur/Organisation (multi-rôles, rôle actif)
- **Expense**: Charges avec workflow d'approbation (Draft → PendingApproval → Approved/Rejected)
- **✅ NOUVEAU: Account**: Plan Comptable Normalisé Belge (PCMN AR 12/07/2012) - Issue #79
- **✅ NOUVEAU: InvoiceLineItem**: Lignes de facturation avec TVA (6%, 12%, 21%) - Issue #73
- **✅ NOUVEAU: PaymentReminder**: Relances automatisées (4 niveaux: Gentle → Formal → FinalNotice → LegalAction) - Issue #83
- **Meeting**: General assemblies (date, agenda, minutes)
- **✅ NOUVEAU: Convocation**: Automatic AG invitations with legal compliance (meeting_type, meeting_date, minimum_send_date, status, pdf_file_path, language, total_recipients, opened_count, will_attend_count, respects_legal_deadline) - Issue #88
- **✅ NOUVEAU: ConvocationRecipient**: Email tracking per owner (email_sent_at, email_opened_at, email_failed, reminder_sent_at, attendance_status, proxy_owner_id, needs_reminder) - Issue #88
- **✅ NOUVEAU: Resolution**: Meeting resolutions with voting (title, description, type, majority_required, vote_counts, status) - Issue #46
- **✅ NOUVEAU: Vote**: Individual votes on resolutions (choice: Pour/Contre/Abstention, voting_power, proxy_owner_id) - Issue #46
- **✅ NOUVEAU: Ticket**: Maintenance requests (title, description, priority, status, category, due_date, assigned_contractor_id) - Issue #85
- **✅ NOUVEAU: Notification**: Multi-channel notifications (title, message, notification_type, channel, is_read, sent_at) - Issue #86
- **✅ NOUVEAU: NotificationPreference**: User notification settings per type (enabled, email_enabled, sms_enabled, push_enabled) - Issue #86
- **✅ NOUVEAU: Payment**: Payment transactions (amount_cents, currency, status, payment_method_type, stripe_payment_intent_id, idempotency_key, refunded_amount_cents) - Issue #84
- **✅ NOUVEAU: PaymentMethod**: Stored payment methods (method_type, stripe_payment_method_id, display_label, is_default, is_active, expires_at) - Issue #84
- **✅ NOUVEAU: Quote**: Contractor quotes with Belgian legal compliance (building_id, contractor_id, project_title, amount_excl_vat, vat_rate, amount_incl_vat, validity_date, estimated_duration_days, warranty_years, contractor_rating, status) - Issue #91
- **Document**: File storage (title, file_path, document_type)

All entities use UUID for IDs and include `created_at`/`updated_at` timestamps.

### ✅ NOUVEAU: Belgian Accounting (PCMN) - Issue #79

- Implémentation complète du Plan Comptable Minimum Normalisé belge (AR 12/07/2012)
- ~90 comptes pré-seed és (8 classes: Actif, Passif, Charges, Produits, Hors-bilan)
- Hiérarchie complète (classes, sous-classes, groupes, comptes)
- Validation codes comptables et types de comptes
- Domain entity: `backend/src/domain/entities/account.rs`
- Use cases: `backend/src/application/use_cases/account_use_cases.rs`
- Repository: `backend/src/infrastructure/database/repositories/account_repository_impl.rs`
- API handlers: `backend/src/infrastructure/web/handlers/account_handlers.rs`
- Financial reports: `backend/src/application/use_cases/financial_report_use_cases.rs` (Bilan & Compte de résultats)
- Tests: 100% couverture domain + integration PostgreSQL
- Documentation: `docs/BELGIAN_ACCOUNTING_PCMN.rst`

### ✅ NOUVEAU: Invoice Workflow - Issue #73

- Workflow complet d'approbation des factures
- États: Draft → PendingApproval → Approved/Rejected
- Gestion TVA belge (6%, 12%, 21%) avec calculs automatiques
- Multi-lignes (InvoiceLineItem) avec quantités et totaux
- Validation métier (empêche modification après approbation)
- Domain entities: `backend/src/domain/entities/expense.rs`, `invoice_line_item.rs`
- Tests: Scénarios BDD + E2E workflow complet
- Documentation: `docs/INVOICE_WORKFLOW.rst`

### ✅ NOUVEAU: Payment Recovery Workflow - Issue #83

- Workflow automatisé de recouvrement des impayés
- 4 niveaux d'escalade: Gentle (J+15) → Formal (J+30) → FinalNotice (J+45) → LegalAction (J+60)
- Calcul automatique pénalités de retard (taux légal belge 8% annuel)
- Traçabilité complète (sent_date, tracking_number, notes)
- Domain entity: `backend/src/domain/entities/payment_reminder.rs`
- Use cases: `backend/src/application/use_cases/payment_reminder_use_cases.rs`
- Tests: Scénarios d'escalade + calcul pénalités
- Documentation: `docs/PAYMENT_RECOVERY_WORKFLOW.rst`

### ✅ NOUVEAU: Meeting Voting System - Issue #46 (Phase 2)

- Système de vote pour assemblées générales (conformité loi belge copropriété)
- **3 types de majorité**: Simple (50%+1 des votes exprimés), Absolute (50%+1 de tous les votes), Qualified (seuil personnalisé, ex: 2/3)
- **Système de tantièmes/millièmes**: Voting power de 0 à 1000 millièmes par lot
- **Vote par procuration**: Support mandataire pour propriétaires absents
- **États des résolutions**: Pending → Adopted/Rejected (calcul automatique selon majorité requise)
- **Audit complet**: Traçabilité GDPR-compliant de tous les votes (création, modification, clôture)
- Domain entities: `backend/src/domain/entities/resolution.rs`, `vote.rs`
- Repositories: `backend/src/infrastructure/database/repositories/resolution_repository_impl.rs`, `vote_repository_impl.rs`
- Use cases: `backend/src/application/use_cases/resolution_use_cases.rs` (14 méthodes)
- API handlers: `backend/src/infrastructure/web/handlers/resolution_handlers.rs` (9 endpoints REST)
- DTOs: `backend/src/application/dto/resolution_dto.rs`, `vote_dto.rs`
- Migration: `backend/migrations/20251115120000_create_resolutions_and_votes.sql` (10 contraintes + 8 index)
- Tests: 27 tests unitaires domain + use cases avec mocks
- Audit events: `ResolutionCreated`, `ResolutionDeleted`, `VoteCast`, `VoteChanged`, `VotingClosed`

### ✅ NOUVEAU: Ticket Management System - Issue #85 (Phase 2)

- Système de gestion des demandes de maintenance et interventions
- **États du workflow**: Open → Assigned → InProgress → Resolved → Closed/Cancelled
- **Priorités**: Low (7 jours), Medium (3 jours), High (24h), Urgent (4h), Critical (1h)
- **Catégories**: Plumbing, Electrical, Heating, Cleaning, Security, General, Emergency
- **Due dates automatiques**: Calculées selon priorité (Critical: 1h, Urgent: 4h, High: 24h, etc.)
- **Gestion contractor**: Assignment, start work, resolution workflow
- **Statistiques complètes**: Count par statut, temps moyen résolution, tickets en retard
- **Multi-tenancy**: Isolation par organisation avec permissions
- Domain entity: `backend/src/domain/entities/ticket.rs` (310 lines)
- Repository: `backend/src/infrastructure/database/repositories/ticket_repository_impl.rs` (18 methods)
- Use cases: `backend/src/application/use_cases/ticket_use_cases.rs` (18 methods)
- API handlers: `backend/src/infrastructure/web/handlers/ticket_handlers.rs` (17 endpoints)
- Migration: `backend/migrations/20251117000000_create_tickets.sql` (custom ENUMs, 8 indexes)
- Audit events: `TicketCreated`, `TicketAssigned`, `TicketStatusChanged`, `TicketResolved`, `TicketClosed`, `TicketCancelled`, `TicketReopened`, `TicketDeleted`

### ✅ NOUVEAU: Multi-Channel Notification System - Issue #86 (Phase 2)

- Système de notifications multi-canal (Email, SMS, Push, In-App)
- **Types de notifications**: MeetingReminder, PaymentDue, DocumentShared, TicketUpdate, SystemAlert, etc. (22 types)
- **Canaux**: Email (primary), SMS (urgent), Push (mobile), InApp (web dashboard)
- **Préférences utilisateur**: Configuration granulaire par type de notification et canal
- **États**: Pending → Sent → Delivered/Failed, Read tracking
- **Métadonnées**: Support JSON pour contexte spécifique (meeting_id, ticket_id, payment_id, etc.)
- **Statistiques**: Total count, unread count, count par type/canal
- Domain entities: `backend/src/domain/entities/notification.rs`, `notification_preference.rs`
- Repositories: `backend/src/infrastructure/database/repositories/notification_repository_impl.rs`, `notification_preference_repository_impl.rs`
- Use cases: `backend/src/application/use_cases/notification_use_cases.rs` (13 methods)
- API handlers: `backend/src/infrastructure/web/handlers/notification_handlers.rs` (11 endpoints)
- Migration: `backend/migrations/20251117000001_create_notifications.sql` (2 tables, custom ENUMs, 9 indexes)
- Audit events: `NotificationCreated`, `NotificationRead`, `NotificationDeleted`, `NotificationPreferenceUpdated`

### ✅ NOUVEAU: Payment Integration System - Issue #84 (Phase 2)

- Système de paiement intégré avec Stripe Payment Intents et SEPA Direct Debit
- **Lifecycle de transaction**: Pending → Processing → RequiresAction → Succeeded/Failed/Cancelled/Refunded
- **Types de paiement**: Card (Visa, Mastercard), SepaDebit (IBAN belge), BankTransfer, Cash
- **Idempotency keys**: Prévention des charges dupliquées sur retry (minimum 16 chars)
- **PCI-DSS Compliance**: Pas de stockage de données carte raw, uniquement Stripe tokens (pm_xxx, sepa_debit_xxx)
- **Remboursements**: Support partiel/complet avec tracking (`refunded_amount_cents`) et validation anti-over-refund
- **Payment Methods**: Gestion cartes et mandats SEPA stockés avec expiration, activation, default management
- **Atomic Default Management**: Un seul payment method default par owner (transaction PostgreSQL)
- **Statistiques complètes**: Total paid, succeeded count, net amount (amount - refunded), par owner/building/expense/organization
- **Multi-tenancy**: Isolation EUR-only pour contexte belge
- Domain entities: `backend/src/domain/entities/payment.rs` (530 lines), `payment_method.rs` (273 lines)
- Repositories: `backend/src/infrastructure/database/repositories/payment_repository_impl.rs` (21 methods), `payment_method_repository_impl.rs` (13 methods)
- Use cases: `backend/src/application/use_cases/payment_use_cases.rs` (26 methods), `payment_method_use_cases.rs` (14 methods)
- API handlers: `backend/src/infrastructure/web/handlers/payment_handlers.rs` (22 endpoints), `payment_method_handlers.rs` (16 endpoints)
- DTOs: `backend/src/application/dto/payment_dto.rs`, `payment_method_dto.rs` (4 DTOs)
- Migration: `backend/migrations/20251118000000_create_payments.sql` (2 tables, custom ENUMs, 10 indexes, constraints)
- Total: ~5,500 lines of code, 38 REST endpoints
- Audit events: `PaymentCreated`, `PaymentProcessing`, `PaymentRequiresAction`, `PaymentSucceeded`, `PaymentFailed`, `PaymentCancelled`, `PaymentRefunded`, `PaymentDeleted`, `PaymentMethodCreated`, `PaymentMethodUpdated`, `PaymentMethodSetDefault`, `PaymentMethodDeactivated`, `PaymentMethodReactivated`, `PaymentMethodDeleted`

### ✅ NOUVEAU: Automatic AG Convocations System - Issue #88 (Phase 2)

- Système de convocations automatiques pour assemblées générales avec conformité légale belge
- **Délais légaux obligatoires**: Ordinary AG (15 jours minimum avant réunion), Extraordinary AG (8 jours), Second Convocation (8 jours après quorum non atteint)
- **Validation multi-niveaux**: Domain entity validation, repository checks, database constraints (minimum_send_date calculation)
- **Workflow complet**: Draft → Scheduled → Sent → Cancelled
- **Email tracking**: email_sent_at, email_opened_at (tracking pixel/link click), email_failed (bounce handling)
- **Reminder automation**: J-3 reminders automatiques pour emails non ouverts (3 jours avant meeting)
- **Attendance workflow**: Pending → WillAttend/WillNotAttend → Attended/DidNotAttend (post-meeting)
- **Proxy delegation**: Support procuration belge (proxy_owner_id) pour délégation de pouvoir de vote
- **Multi-language**: Support FR/NL/DE/EN pour génération PDF selon langue du destinataire
- **Bulk operations**: create_many avec transaction PostgreSQL pour création atomique de recipients
- **Tracking metrics**: opening_rate, attendance_rate, computed fields in DTOs
- **Background job support**: process_scheduled_convocations (envoyer convocations schedulées), process_reminder_sending (reminders J-3)
- Domain entities: `backend/src/domain/entities/convocation.rs` (440 lines), `convocation_recipient.rs` (260 lines)
- Repositories: `backend/src/infrastructure/database/repositories/convocation_repository_impl.rs` (600 lines, 13 methods), `convocation_recipient_repository_impl.rs` (750 lines, 18 methods)
- Use cases: `backend/src/application/use_cases/convocation_use_cases.rs` (430 lines, 21 methods avec multi-repo orchestration)
- API handlers: `backend/src/infrastructure/web/handlers/convocation_handlers.rs` (435 lines, 14 endpoints)
- DTOs: `backend/src/application/dto/convocation_dto.rs`, `convocation_recipient_dto.rs` (4 DTOs avec computed fields)
- Migration: `backend/migrations/20251119000000_create_convocations.sql` (2 tables, 3 custom ENUMs, 14 indexes, 10 constraints)
- Repository tracking: `RecipientTrackingSummary` struct (8 metrics: total, opened, will_attend, will_not_attend, attended, did_not_attend, pending, failed)
- Total: ~3,650 lines of code, 14 REST endpoints, 19 unit tests domain
- Audit events: `ConvocationCreated`, `ConvocationScheduled`, `ConvocationSent`, `ConvocationCancelled`, `ConvocationDeleted`, `ConvocationReminderSent`, `ConvocationAttendanceUpdated`, `ConvocationProxySet`

### ✅ NOUVEAU: GDPR Complementary Articles System - Issue #90 (Phase 2)

- Système complet de conformité GDPR avec Articles 16, 18, 21 (complémentaires à 15 & 17 existants)
- **Database fields** (migration 20251120000000_add_gdpr_complementary_fields.sql):
  * `processing_restricted` (BOOLEAN, default FALSE) - Article 18
  * `processing_restricted_at` (TIMESTAMPTZ) - Audit trail for restriction
  * `marketing_opt_out` (BOOLEAN, default FALSE) - Article 21
  * `marketing_opt_out_at` (TIMESTAMPTZ) - Audit trail for opt-out
  * 2 partial indexes for admin queries (WHERE processing_restricted = TRUE, WHERE marketing_opt_out = TRUE)
- **User domain entity methods** (user.rs):
  * Article 16: `rectify_data(email?, first_name?, last_name?)` - Correct inaccurate data with validation
  * Article 18: `restrict_processing()`, `unrestrict_processing()` - Limit data processing temporarily
  * Article 21: `set_marketing_opt_out(opt_out: bool)` - Object to marketing/profiling
  * Helpers: `can_process_data()`, `can_send_marketing()` - Check restrictions
  * 11 unit tests (3 Article 16, 4 Article 18, 3 Article 21, 1 defaults)
- **GDPR use cases** (gdpr_use_cases.rs):
  * Requires 2 repositories: GdprRepository + UserRepository
  * Article 16: `rectify_user_data()` - Fetch user, apply rectifications, persist
  * Article 18: `restrict_user_processing()`, `unrestrict_user_processing()` - Apply restrictions
  * Article 21: `set_marketing_preference()` - Set opt-out preference
  * All methods include authorization (users can only modify their own data)
  * Audit trail preservation (timestamps kept when unrestricting or opting back in)
- **REST API handlers** (gdpr_handlers.rs - 320 lines):
  * PUT /gdpr/rectify - Article 16 (200 OK, 400 Bad Request validation, 403 Forbidden, 404 Not Found)
  * PUT /gdpr/restrict-processing - Article 18 (200 OK, 400 already restricted, 403, 404)
  * PUT /gdpr/marketing-preference - Article 21 (200 OK, 403, 404)
  * All handlers: async audit logging, IP/user-agent tracking, user-friendly messages
- **DTOs** (gdpr_dto.rs):
  * GdprRectifyRequest (email?, first_name?, last_name?)
  * GdprRestrictProcessingRequest (empty body)
  * GdprMarketingPreferenceRequest (opt_out: bool)
  * GdprActionResponse (success, message, updated_at) - Generic success response
- **Audit events** (7 types added to audit.rs):
  * Article 16: GdprDataRectified, GdprDataRectificationFailed
  * Article 18: GdprProcessingRestricted, GdprProcessingRestrictionFailed
  * Article 21: GdprMarketingOptOut, GdprMarketingOptIn, GdprMarketingPreferenceChangeFailed
- **Complete GDPR Compliance**:
  * ✅ Article 15: Right to Access (GET /gdpr/export) - Existing
  * ✅ Article 16: Right to Rectification (PUT /gdpr/rectify) - NEW
  * ✅ Article 17: Right to Erasure (DELETE /gdpr/erase) - Existing
  * ✅ Article 18: Right to Restriction of Processing (PUT /gdpr/restrict-processing) - NEW
  * ✅ Article 21: Right to Object to Marketing (PUT /gdpr/marketing-preference) - NEW
  * ✅ Article 30: Records of Processing (all actions logged with IP/user-agent)
- **Architecture**:
  * Hexagonal: Domain validation → Use Cases authorization → REST handlers
  * Authorization: Self-service (users modify their own data)
  * Async: Non-blocking audit logging with spawn()
  * Validation: Email format, domain business rules enforced
- **Total**: 1 migration, 4 User methods, 11 unit tests, 4 Use Case methods, 4 DTOs, 3 REST handlers (320 lines), 7 audit events
- **Belgian Legal Compliance**: Full GDPR compliance for Belgian ASBL operations

### ✅ NOUVEAU: Contractor Quotes Module - Issue #91 (Phase 2)

- Système complet de gestion des devis entrepreneurs avec conformité légale belge
- **Belgian Legal Requirement**: 3 quotes mandatory for construction works >5000€
- **Automatic Scoring Algorithm**:
  * Price: 40% weight (lower price = higher score, inverted normalization)
  * Delay: 30% weight (shorter estimated_duration_days = higher score)
  * Warranty: 20% weight (longer warranty_years = higher score)
  * Reputation: 10% weight (contractor_rating 0-100 scale)
- **Quote Workflow State Machine**: Requested → Received → UnderReview → Accepted/Rejected/Expired/Withdrawn
- **Belgian VAT Rates**: 6% reduced (renovations), 21% standard (new construction)
- **Belgian Warranty Standards**: 2 years (apparent defects), 10 years (structural - "décennale")
- **Expiration Tracking**: validity_date field with automatic expiration detection (find_expired query)
- **Decision Audit Trail**: decision_at, decision_by, decision_notes fields for legal compliance
- Domain entity: `backend/src/domain/entities/quote.rs` (661 lines, 7 state transitions, validation logic)
- Repository: `backend/src/infrastructure/database/repositories/quote_repository_impl.rs` (373 lines, 15 methods)
- Use cases: `backend/src/application/use_cases/quote_use_cases.rs` (433 lines, 20 methods including compare_quotes)
- API handlers: `backend/src/infrastructure/web/handlers/quote_handlers.rs` (376 lines, 15 REST endpoints)
- DTOs: `backend/src/application/dto/quote_dto.rs` (227 lines, 5 DTOs including QuoteComparisonResponseDto)
- Migration: `backend/migrations/20251120150000_create_quotes.sql` (91 lines, custom quote_status ENUM, 8 indexes, 4 constraints, trigger)
- Total: ~2,161 lines of code, 15 REST endpoints, 20 use case methods
- Audit events: `QuoteCreated`, `QuoteSubmitted`, `QuoteUnderReview`, `QuoteAccepted`, `QuoteRejected`, `QuoteWithdrawn`, `QuoteExpired`, `QuoteRatingUpdated`, `QuoteComparisonPerformed`, `QuoteDeleted`

### ✅ NOUVEAU: Public Syndic Information Page - Issue #92 (Phase 2)

- Système de page publique d'information syndic conforme à la loi belge
- **Exigence légale belge**: Les syndics doivent afficher publiquement leurs coordonnées de contact
- **SEO-friendly URLs**: Génération automatique de slugs URL à partir du nom + ville du bâtiment
- **Slug generation**: Normalisation des accents (é→e, à→a, etc.), conversion en minuscules, séparation par tirets
- **Public endpoint**: GET /api/v1/public/buildings/{slug}/syndic (aucune authentification requise)
- **7 champs syndic publics**: syndic_name, syndic_email, syndic_phone, syndic_address, syndic_office_hours, syndic_emergency_contact, slug
- **Migration** (20251120120000_add_syndic_public_info_to_buildings.sql):
  * 7 colonnes syndic + slug UNIQUE
  * 2 indexes (idx_buildings_slug, idx_buildings_syndic_name)
  * Commentaires de colonnes pour documentation légale
- **Building domain entity** (building.rs):
  * `generate_slug(name, address, city)` - Génération SEO-friendly avec gestion accents
  * `update_syndic_info()` - Mise à jour information syndic
  * `has_public_syndic_info()` - Vérification disponibilité info publique
  * Slug généré automatiquement lors de la création du bâtiment
- **DTO** (public_dto.rs):
  * `PublicSyndicInfoResponse` - DTO sans authentification pour endpoint public
  * `From<Building>` conversion avec computed field `has_syndic_info`
  * 2 unit tests (avec/sans info syndic)
- **BuildingRepository** (building_repository.rs):
  * `find_by_slug(slug: &str)` - Nouvelle méthode trait
  * PostgresBuildingRepository: TOUS les queries SQL mis à jour (create, find_by_id, find_all, find_all_paginated, update, find_by_slug)
  * 19 colonnes totales (12 existantes + 7 syndic fields)
  * 359 lignes totales de repository (refactoring complet)
- **BuildingUseCases** (building_use_cases.rs):
  * `find_by_slug()` - Wrapper use case pour endpoint public
- **Public handler** (public_handlers.rs):
  * `get_public_syndic_info()` - Handler Actix-web sans authentification
  * Codes HTTP: 200 OK, 404 Not Found, 500 Internal Server Error
  * Documentation inline avec exemple de réponse JSON
- **Routes** (routes.rs):
  * Endpoint public ajouté en tête de scope (avant middleware auth)
  * Position stratégique pour éviter interception par middleware
- **Conformité**: Loi belge sur transparence des syndics de copropriété
- **Total**: 1 migration, 7 DB fields, 3 Building methods, 1 DTO, 1 Use Case method, 1 REST handler, 1 public endpoint

### Multi-owner support

- Junction table `unit_owners` (see `backend/migrations/20250127000000_refactor_owners_multitenancy.sql`) enables many-to-many between units and owners.
- Domain entity: `backend/src/domain/entities/unit_owner.rs` (pourcentage `0.0 < p ≤ 1.0`, timestamps, primary contact).
- Use cases: `backend/src/application/use_cases/unit_owner_use_cases.rs` (somme des quotes-parts ≤ 100 %, transfert, historique, contact principal unique).
- Web handlers: `backend/src/infrastructure/web/handlers/unit_owner_handlers.rs` exposent les routes `/api/v1/units/{id}/owners`, `/unit-owners/{id}`, `/units/{id}/owners/transfer`, etc.
- Tests : `backend/tests/integration_unit_owner.rs` (PostgreSQL) + BDD multi-tenant.
- Frontend Svelte : `frontend/src/components/UnitOwners.svelte`, `OwnerList.svelte`, `OwnerCreateModal.svelte`, `OwnerEditModal.svelte`.
- Documentation produit : `docs/MULTI_OWNER_SUPPORT.md`.

### Multi-role support

- Table `user_roles` (migration `20250130000000_add_user_roles.sql`).
- Domain entity: `backend/src/domain/entities/user_role_assignment.rs`.
- Repository: `PostgresUserRoleRepository` (création, switch primary, liste).
- Use cases: `AuthUseCases::login`, `AuthUseCases::switch_active_role`, `AuthUseCases::refresh_token`.
- Middleware `AuthenticatedUser` expose `role_id`.
- Endpoints `/auth/login`, `/auth/switch-role`, `/auth/me` (JWT avec rôle actif).
- Frontend: `authStore.switchRole`, `Navigation.svelte` (sélecteur multi-rôle).
- Tests:
  - E2E: `tests/e2e_auth.rs` (scénario multi-rôles).
  - BDD: `features/auth.feature` (issue #28).
  - Docs: `docs/MULTI_ROLE_SUPPORT.md`.

## Performance Targets

- **Latency P99**: < 5ms
- **Throughput**: > 100k req/s
- **Memory**: < 128MB per instance
- **Connection Pool**: Max 10 PostgreSQL connections
- **Compilation**: Release mode with `opt-level = 3`, `lto = true`, `codegen-units = 1`

## Database Configuration

PostgreSQL 15 via Docker:
- Database: `koprogo_db`
- User: `koprogo` / Password: `koprogo123` (dev only)
- Port: `5432`
- Connection string: `postgresql://koprogo:koprogo123@localhost:5432/koprogo_db`

Migrations managed via `sqlx migrate` in `backend/migrations/`.

## Testing Philosophy

Follow TDD (Test-Driven Development):
1. Write tests first (especially for domain logic)
2. Implement to pass tests
3. Refactor with test safety net

**BDD Features**: Write Gherkin scenarios in `backend/tests/features/` for user-facing behaviors.

**Integration Tests**: Use testcontainers for real PostgreSQL instances, ensuring tests are isolated.

## Frontend (Astro + Svelte)

- **SSG (Static Site Generation)**: Astro builds static pages
- **Islands Architecture**: Interactive Svelte components in `frontend/src/components/`
- **Layouts**: `frontend/src/layouts/`
- **Pages**: `frontend/src/pages/`
- **Styling**: Tailwind CSS

Frontend commands (in `frontend/`):
```bash
npm run dev      # Dev server
npm run build    # Production build (runs astro check first)
npm run preview  # Preview production build
```

## Environment Variables

Backend (`backend/.env`):
- `DATABASE_URL`: PostgreSQL connection string
- `SERVER_HOST`: Default `127.0.0.1`
- `SERVER_PORT`: Default `8080`
- `RUST_LOG`: Log level (e.g., `info`, `debug`)

Frontend (`frontend/.env`):
- `PUBLIC_API_URL`: Backend API URL (e.g., `http://localhost:8080/api/v1`)

## CI/CD

GitHub Actions workflows configured for:
- Running tests (unit, integration, BDD, E2E)
- Linting and formatting checks
- Building release artifacts

**Hooks Git locaux** :
- Installer via `make install-hooks` (alias `./scripts/install-hooks.sh`)
- `pre-commit` ⇒ `make format` + `make lint`
- `pre-push` ⇒ `make lint` + `make test` (unit + BDD + build frontend)
- Dépannage détaillé dans `docs/GIT_HOOKS.rst`

**Current branch**: `chore/new-branch-workflow`
**Main branch**: `main`

## Key Dependencies

**Backend**:
- `actix-web` 4.9: Web framework
- `sqlx` 0.8: Database with compile-time query verification
- `tokio` 1.41: Async runtime
- `uuid`, `chrono`: Data types
- `serde`, `serde_json`: Serialization
- `cucumber` 0.21: BDD testing
- `testcontainers` 0.23: Integration testing
- `criterion` 0.5: Benchmarking

**Frontend**:
- `astro` 4.x: SSG framework
- `svelte` 4.x: Component framework
- `tailwindcss` 3.x: Styling
- `@playwright/test`: E2E testing

## Roadmap Context

**Current Phase (MVP)**: Core CRUD operations, complete test suite, Docker setup
**Next Phases**: ScyllaDB/DragonflyDB integration, Kubernetes deployment, JWT auth, advanced features (document generation, real-time notifications)
