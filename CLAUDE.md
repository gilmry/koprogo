# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🗺️ Roadmap

**📅 For the complete development roadmap, see [ROADMAP_PAR_CAPACITES.rst](docs/ROADMAP_PAR_CAPACITES.rst)**

The roadmap follows a **capacity-based progression** (not fixed dates):
- **Jalon 0 ✅**: Fondations Techniques (COMPLÉTÉ - Architecture, 73 endpoints API, tests)
- **Jalon 1 🔒**: Sécurité & GDPR → Débloque 50-100 copros (beta publique)
- **Jalon 2 📋**: Conformité Légale Belge → Débloque 200-500 copros (production)
- **Jalon 3 🎯**: Features Différenciantes (Voting, SEL, Contractor) → Débloque 500-1,000 copros
- **Jalon 4 📅**: Automation & Intégrations → Débloque 1,000-2,000 copros
- **Jalon 5 📱**: Mobile & API Publique → Débloque 2,000-5,000 copros
- **Jalon 6-7 🤖**: PropTech 2.0 (IA, IoT, Blockchain) → Débloque 5,000+ copros

**Philosophy**: "We deliver when ready, not according to arbitrary dates" - See [JALONS_MIGRATION.rst](docs/JALONS_MIGRATION.rst)

All issues tracked in [GitHub Projects](https://github.com/users/gilmry/projects) and [GitHub Milestones](https://github.com/gilmry/koprogo/milestones)

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
**✅ NOUVEAU: Local Exchanges (SEL - Système d'Échange Local)** (Issue #49 - Phase 1 - Belgian Community Features):
   - `POST /exchanges` - Create exchange offer (Service/ObjectLoan/SharedPurchase)
   - `GET /exchanges/:id` - Get exchange details with provider/requester names
   - `GET /buildings/:building_id/exchanges` - List all building exchanges
   - `GET /buildings/:building_id/exchanges/available` - List available exchanges (status = Offered)
   - `GET /owners/:owner_id/exchanges` - List owner exchanges (as provider OR requester)
   - `GET /buildings/:building_id/exchanges/type/:exchange_type` - List by type (Service/ObjectLoan/SharedPurchase)
   - `POST /exchanges/:id/request` - Request exchange (Offered → Requested, sets requester_id)
   - `POST /exchanges/:id/start` - Start exchange (Requested → InProgress, provider only)
   - `POST /exchanges/:id/complete` - Complete exchange (InProgress → Completed, automatic credit balance updates)
   - `POST /exchanges/:id/cancel` - Cancel exchange (cancellation reason required)
   - `PUT /exchanges/:id/rate-provider` - Rate provider (1-5 stars, requester only, completed exchanges)
   - `PUT /exchanges/:id/rate-requester` - Rate requester (1-5 stars, provider only, completed exchanges)
   - `DELETE /exchanges/:id` - Delete exchange (provider only, not completed)
   - `GET /owners/:owner_id/buildings/:building_id/credit-balance` - Get credit balance (time-based currency: 1h = 1 credit)
   - `GET /buildings/:building_id/leaderboard` - Leaderboard (top contributors, ordered by balance DESC, limit=10 default)
   - `GET /buildings/:building_id/sel-statistics` - Statistics (total/active/completed exchanges, total credits exchanged, active participants, average rating, most popular type)
   - `GET /owners/:owner_id/exchange-summary` - Owner summary (total offered/requested/completed, credits earned/spent/balance, average rating, participation level)
**✅ NOUVEAU: Gamification & Achievements** (Issue #49 - Phase 6/6 - Belgian Community Engagement):
   - **Achievements** (7 endpoints):
     - `POST /achievements` - Create achievement (8 categories, 5 tiers, 0-1000 points, secret/repeatable flags)
     - `GET /achievements/:id` - Get achievement details
     - `GET /organizations/:organization_id/achievements` - List all achievements
     - `GET /organizations/:organization_id/achievements/category/:category` - List by category (Community/Sel/Booking/Sharing/Skills/Notice/Governance/Milestone)
     - `GET /organizations/:organization_id/achievements/visible` - List visible achievements (non-secret + earned secret achievements)
     - `PUT /achievements/:id` - Update achievement
     - `DELETE /achievements/:id` - Delete achievement
   - **User Achievements** (3 endpoints):
     - `POST /users/achievements` - Award achievement (supports repeatable achievements, tracks times_earned)
     - `GET /users/:user_id/achievements` - List user earned achievements
     - `GET /users/:user_id/achievements/recent` - Recent achievements (limit query param)
   - **Challenges** (9 endpoints):
     - `POST /challenges` - Create challenge (Individual/Team/Building types, target metrics, 0-10000 reward points)
     - `GET /challenges/:id` - Get challenge details
     - `GET /organizations/:organization_id/challenges` - List all challenges
     - `GET /organizations/:organization_id/challenges/status/:status` - List by status (Draft/Active/Completed/Cancelled)
     - `GET /organizations/:organization_id/challenges/active` - List active challenges (start_date <= NOW < end_date)
     - `PUT /challenges/:id/activate` - Activate challenge (Draft → Active)
     - `PUT /challenges/:id/complete` - Complete challenge (Active → Completed)
     - `PUT /challenges/:id/cancel` - Cancel challenge (→ Cancelled)
     - `DELETE /challenges/:id` - Delete challenge
   - **Challenge Progress** (4 endpoints):
     - `GET /challenges/:challenge_id/progress/:user_id` - Get user progress
     - `GET /challenges/:challenge_id/progress` - List all challenge progress
     - `GET /users/:user_id/challenges/active` - List active user challenges
     - `POST /challenges/:challenge_id/progress/increment` - Increment progress (auto-completes if target reached)
   - **Gamification Statistics** (2 endpoints):
     - `GET /users/:user_id/gamification/stats` - User stats (total points, achievements count, challenges completed)
     - `GET /organizations/:organization_id/gamification/leaderboard` - Leaderboard (top users by points, building filter, limit query param)
**✅ NOUVEAU: Public Syndic Information** (Issue #92 - Phase 2 - Belgian Legal Requirement):
   - `GET /public/buildings/:slug/syndic` - Get public syndic contact info (no authentication required)
**✅ NOUVEAU: Polls (Board Decision Polling System)** (Issue #51 - Phase 2 - Belgian Consultation Between Assemblies):
   - `POST /polls` - Create poll (YesNo/MultipleChoice/Rating/OpenEnded types)
   - `GET /polls/:id` - Get poll details with options and vote counts
   - `GET /buildings/:building_id/polls` - List all building polls
   - `GET /buildings/:building_id/polls/active` - List active polls (status = Active)
   - `GET /buildings/:building_id/polls/status/:status` - List by status (Draft/Active/Closed/Cancelled)
   - `PUT /polls/:id/publish` - Publish poll (Draft → Active, enables voting)
   - `PUT /polls/:id/close` - Close poll (Active → Closed, calculate results)
   - `PUT /polls/:id/cancel` - Cancel poll (→ Cancelled)
   - `DELETE /polls/:id` - Delete poll
   - `POST /polls/:id/vote` - Cast vote (YesNo: yes/no, MultipleChoice: option_id, Rating: 1-5, OpenEnded: text)
   - `GET /polls/:id/votes` - List all poll votes (admin/syndic only)
   - `GET /polls/:id/results` - Get poll results with statistics (vote counts, percentages, participation rate)
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
- **✅ NOUVEAU: LocalExchange**: SEL time-based exchange system (building_id, provider_id, requester_id, exchange_type, title, description, credits, status, ratings, timestamps) - Issue #49 Phase 1
- **✅ NOUVEAU: OwnerCreditBalance**: Time-based currency balance per owner (owner_id, building_id, credits_earned, credits_spent, balance, total_exchanges, average_rating, participation_level) - Issue #49 Phase 1
- **✅ NOUVEAU: Achievement**: Achievement definitions (organization_id, category, tier, name, description, icon, points_value, requirements, is_secret, is_repeatable, display_order) - Issue #49 Phase 6
- **✅ NOUVEAU: UserAchievement**: User-earned achievements (user_id, achievement_id, earned_at, progress_data, times_earned) - Issue #49 Phase 6
- **✅ NOUVEAU: Challenge**: Time-bound challenges (organization_id, building_id, challenge_type, status, title, description, icon, start_date, end_date, target_metric, target_value, reward_points) - Issue #49 Phase 6
- **✅ NOUVEAU: ChallengeProgress**: Challenge progress tracking (challenge_id, user_id, current_value, completed, completed_at) - Issue #49 Phase 6
- **Document**: File storage (title, file_path, document_type)
- **✅ NOUVEAU: Poll**: Board decision polls for owner consultations (building_id, poll_type, question, description, status, starts_at, ends_at, is_anonymous, total_eligible_voters, total_votes_cast, allow_multiple_votes, min_rating, max_rating) - Issue #51
- **✅ NOUVEAU: PollOption**: Poll answer options (poll_id, option_text, option_value, display_order, vote_count) - Issue #51
- **✅ NOUVEAU: PollVote**: Individual votes on polls (poll_id, owner_id, option_id, vote_value, vote_text, ip_address, is_anonymous) - Issue #51

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

### ✅ NOUVEAU: SEL - Système d'Échange Local (Local Exchange Trading System) - Issue #49 (Phase 1)

- Système d'échange local à base de temps (time-based currency: 1 heure = 1 crédit)
- **Belgian Legal Compliance**: SELs are legal in Belgium, non-taxable if non-commercial
- **Trust Model**: Negative credit balances allowed (community trust-based system)
- **3 Exchange Types**:
  * **Service**: Skills exchange (plumbing, gardening, tutoring, babysitting, IT help, etc.)
  * **ObjectLoan**: Temporary loan (tools, books, equipment, appliances)
  * **SharedPurchase**: Co-buying (bulk food, equipment rental sharing, group purchases)
- **5-State Exchange Workflow**:
  * **Offered**: Provider creates exchange offer (marketplace listing)
  * **Requested**: Requester requests exchange (requester_id set, provider can accept/reject)
  * **InProgress**: Provider starts exchange (service/loan/purchase in progress)
  * **Completed**: Exchange completed (automatic credit balance updates for both parties)
  * **Cancelled**: Exchange cancelled (cancellation reason required for audit trail)
- **Automatic Credit Balance Management**:
  * Completing exchange automatically updates both parties' balances atomically
  * Provider earns credits (+credits), Requester spends credits (-credits)
  * Balance = credits_earned - credits_spent (can be negative)
- **Mutual Rating System**:
  * Provider rates requester (1-5 stars, trust/reliability)
  * Requester rates provider (1-5 stars, quality/service)
  * Only completed exchanges can be rated
  * Average rating tracked in OwnerCreditBalance for reputation
- **Participation Levels** (gamification):
  * New: 0 exchanges (beginner)
  * Beginner: 1-5 exchanges (starting to engage)
  * Active: 6-20 exchanges (regular participant)
  * Veteran: 21-50 exchanges (experienced contributor)
  * Expert: 51+ exchanges (community pillar)
- **Credit Status**:
  * Positive: net provider (balance > 0, more services offered than received)
  * Balanced: balanced exchanges (balance ≈ 0)
  * Negative: net receiver (balance < 0, more services received than offered)
- **Community Features**:
  * Leaderboard: Top contributors ordered by balance DESC (limit=10 default)
  * Statistics: Total/active/completed exchanges, total credits exchanged, active participants, average rating, most popular exchange type
  * Owner Summary: Total offered/requested/completed, credits earned/spent/balance, average rating, participation level
- **Domain Entities**:
  * `LocalExchange` (661 lines): Core exchange aggregate with state machine validation
  * `OwnerCreditBalance` (273 lines): Time-based currency balance per owner
- **Repository Pattern**:
  * `LocalExchangeRepository` trait (18 methods)
  * `OwnerCreditBalanceRepository` trait (11 methods)
  * PostgreSQL implementations with complex queries and aggregations
- **Use Cases** (652 lines):
  * `LocalExchangeUseCases` - 20 methods with multi-repository orchestration
  * Requires 3 repositories: LocalExchange, OwnerCreditBalance, Owner (for name lookups)
  * Owner name enrichment for all DTOs (provider_name, requester_name)
- **REST API** (17 endpoints, ~400 lines):
  * Exchange Management: create, get, list (all/available/owner/type), delete
  * Workflow: request, start, complete, cancel
  * Rating: rate-provider, rate-requester
  * Analytics: credit-balance, leaderboard, statistics, owner-summary
- **Migration** (20251120160000_create_local_exchanges.sql, 143 lines):
  * Table: `local_exchanges` (15 columns, 8 indexes, 6 constraints)
  * Table: `owner_credit_balances` (9 columns, 3 indexes, 2 constraints)
  * Custom ENUMs: `exchange_type` (Service, ObjectLoan, SharedPurchase)
  * Custom ENUMs: `exchange_status` (Offered, Requested, InProgress, Completed, Cancelled)
  * Partial indexes for optimization (active exchanges marketplace, leaderboard queries)
  * Constraints: credits 1-100 hours, ratings 1-5 stars, provider != requester
- **DTOs** (8 DTOs, 150 lines):
  * `CreateLocalExchangeDto`, `RequestExchangeDto`, `CompleteExchangeDto`, `CancelExchangeDto`
  * `RateExchangeDto`, `LocalExchangeResponseDto`, `OwnerCreditBalanceDto`
  * `SelStatisticsDto`, `OwnerExchangeSummaryDto`
- **Audit Events** (10 types, GDPR Article 30 compliance):
  * `ExchangeCreated`, `ExchangeRequested`, `ExchangeStarted`, `ExchangeCompleted`, `ExchangeCancelled`
  * `ExchangeProviderRated`, `ExchangeRequesterRated`, `ExchangeDeleted`
  * `CreditBalanceUpdated`, `CreditBalanceCreated`
- **Total Phase 1 Scope**:
  * ~3,000 lines of code across 4 layers (Domain, Application, Infrastructure, Migration)
  * 2 domain entities (31 unit tests)
  * 8 DTOs
  * 2 repository traits + 2 PostgreSQL implementations (29 methods total)
  * 1 use case class (20 methods)
  * 17 REST endpoints
  * 1 migration (2 tables, 2 ENUMs, 11 indexes, 6 constraints)
  * 10 audit event types
- **Files Created**:
  * Domain: `local_exchange.rs` (661 lines), `owner_credit_balance.rs` (273 lines)
  * DTOs: `local_exchange_dto.rs` (150 lines)
  * Ports: `local_exchange_repository.rs`, `owner_credit_balance_repository.rs`
  * Implementations: `local_exchange_repository_impl.rs` (466 lines), `owner_credit_balance_repository_impl.rs` (186 lines)
  * Use Cases: `local_exchange_use_cases.rs` (652 lines)
  * Handlers: `local_exchange_handlers.rs` (~400 lines)
  * Migration: `20251120160000_create_local_exchanges.sql` (143 lines)
- **Commits**:
  * 6aa4698: Foundation (Domain + DTOs + Repositories) - 1,857 LOC
  * 686871c: Use Cases (20 methods) - 609 LOC
  * fc3b325: REST API + Migration (17 endpoints) - 530 LOC

**✅ ALL 6 PHASES COMPLETE** (Issue #49 - Full Community Features):
- **Phase 1**: SEL - Local Exchange System ✅ COMPLETE (3 commits, ~3,000 LOC)
- **Phase 2**: Community Notice Board ✅ COMPLETE (~2,940 LOC)
- **Phase 3**: Skills Directory ✅ COMPLETE (~2,650 LOC)
- **Phase 4**: Object Sharing Library ✅ COMPLETE (2 commits, ~2,905 LOC)
- **Phase 5**: Resource Booking Calendar ✅ COMPLETE (2 commits, ~3,105 LOC)
- **Phase 6**: Gamification & Achievements ✅ COMPLETE (7 commits, ~6,500 LOC)

**Total Scope**: 6 phases, ~21,100 lines of code, 17+ commits, full-stack community engagement platform

### ✅ NOUVEAU: Gamification & Achievements System - Issue #49 (Phase 6/6)

- Système complet de gamification pour engagement communautaire avec achievements et challenges
- **Belgian Context**: Community participation recognition and engagement promotion
- **Achievement System Features**:
  * **8 Achievement Categories**: Community, SEL, Booking, Sharing, Skills, Notice, Governance, Milestone
  * **5 Achievement Tiers**: Bronze (entry-level), Silver (intermediate), Gold (advanced), Platinum (expert), Diamond (exceptional)
  * **Points System**: 0-1000 points per achievement
  * **Secret Achievements**: Hidden until earned (visibility logic based on user progress)
  * **Repeatable Achievements**: Can be earned multiple times with times_earned counter
  * **Requirements**: JSON criteria for achievement validation (e.g., {"action": "booking_created", "count": 1})
  * **Display Ordering**: Custom ordering for UI presentation
- **Challenge System Features**:
  * **3 Challenge Types**: Individual (user-based), Team (not implemented Phase 6), Building (building-wide)
  * **4 Challenge States**: Draft → Active → Completed/Cancelled
  * **Time-bound**: start_date and end_date with CHECK constraint validation
  * **Target Metrics**: Flexible metric tracking (e.g., "bookings_created", "exchanges_completed")
  * **Reward Points**: 0-10,000 points upon completion
  * **Building Scope**: Organization-wide or building-specific challenges
  * **Progress Tracking**: Per-user current_value with auto-completion when target reached
- **Leaderboard System**:
  * Multi-source point aggregation (achievements + challenges)
  * Building filter support for localized competition
  * Top N users ranking (default limit=10, configurable)
  * User stats: total_points, achievements_count, challenges_completed
- **Domain Entities**:
  * `Achievement` (661 lines): Achievement definitions with validation
  * `UserAchievement` (273 lines): User-earned achievements with repeat logic
  * `Challenge` (440 lines): Challenge definitions with state machine
  * `ChallengeProgress` (260 lines): Progress tracking with auto-completion
- **Repository Pattern**:
  * `AchievementRepository` trait (8 methods) - PostgresAchievementRepository
  * `UserAchievementRepository` trait (8 methods) - PostgresUserAchievementRepository
  * `ChallengeRepository` trait (10 methods) - PostgresChallengeRepository
  * `ChallengeProgressRepository` trait (9 methods) - PostgresChallengeProgressRepository
  * Complex queries: secret achievement visibility, leaderboard aggregation, active challenge filtering
- **Use Cases** (654 lines):
  * `AchievementUseCases` - 10 methods (create, award, list, visibility logic)
  * `ChallengeUseCases` - 16 methods (create, state transitions, progress tracking, auto-completion)
  * `GamificationStatsUseCases` - 2 methods (user stats, leaderboard with multi-repo orchestration)
  * Requires 5 repositories: Achievement, UserAchievement, Challenge, ChallengeProgress, User
- **REST API** (22 endpoints, 700 lines):
  * Achievements Management: 7 endpoints (create, get, list, list by category, list visible, update, delete)
  * User Achievements: 3 endpoints (award, list user achievements, recent achievements)
  * Challenges Management: 9 endpoints (create, get, list variants, state transitions, delete)
  * Challenge Progress: 4 endpoints (get progress, list all, list active, increment with auto-complete)
  * Gamification Statistics: 2 endpoints (user stats, leaderboard)
- **Migration** (20251120220000_create_gamification.sql, 233 lines):
  * Table: `achievements` (14 columns, 7 indexes, 3 constraints)
  * Table: `user_achievements` (6 columns, 2 indexes, 1 UNIQUE constraint)
  * Table: `challenges` (15 columns, 7 indexes, 2 CHECK constraints)
  * Table: `challenge_progress` (7 columns, 3 indexes, 2 constraints including completed_at validation)
  * Custom ENUMs: `achievement_category` (8 values), `achievement_tier` (5 values), `challenge_type` (3 values), `challenge_status` (4 values)
  * Partial indexes: idx_challenges_active (WHERE status = 'Active'), idx_challenges_ended_not_completed (WHERE status = 'Active' AND end_date <= NOW())
  * Trigger functions: update_achievement_timestamp, update_challenge_timestamp, update_challenge_progress_timestamp
- **DTOs** (8 DTOs, 200 lines):
  * `CreateAchievementDto`, `UpdateAchievementDto`, `AchievementResponseDto`, `UserAchievementResponseDto`
  * `CreateChallengeDto`, `ChallengeResponseDto`, `ChallengeProgressResponseDto`
  * `LeaderboardEntryDto`, `LeaderboardResponseDto`, `GamificationUserStatsDto`
  * `AwardAchievementRequest`, `IncrementProgressRequest`
- **Audit Events** (12 types, GDPR Article 30 compliance):
  * `AchievementCreated`, `AchievementUpdated`, `AchievementDeleted`, `AchievementAwarded`
  * `ChallengeCreated`, `ChallengeActivated`, `ChallengeUpdated`, `ChallengeCompleted`, `ChallengeCancelled`, `ChallengeDeleted`
  * `ChallengeProgressIncremented`, `ChallengeProgressCompleted`
- **Total Phase 6 Scope**:
  * ~6,500 lines of code across 4 layers (Domain, Application, Infrastructure, Migration)
  * 4 domain entities (31 unit tests)
  * 8 DTOs
  * 4 repository traits + 4 PostgreSQL implementations (35 methods total)
  * 3 use case classes (28 methods)
  * 22 REST endpoints
  * 1 migration (4 tables, 4 ENUMs, 17 indexes, 10 constraints, 3 triggers)
  * 12 audit event types
- **Files Created**:
  * Domain: `achievement.rs` (661 lines), `user_achievement.rs` (273 lines), `challenge.rs` (440 lines), `challenge_progress.rs` (260 lines)
  * DTOs: `gamification_dto.rs` (200 lines)
  * Ports: `achievement_repository.rs`, `user_achievement_repository.rs`, `challenge_repository.rs`, `challenge_progress_repository.rs`
  * Implementations: `achievement_repository_impl.rs` (517 lines), `challenge_repository_impl.rs` (676 lines)
  * Use Cases: `achievement_use_cases.rs`, `challenge_use_cases.rs`, `gamification_stats_use_cases.rs` (total 654 lines)
  * Handlers: `gamification_handlers.rs` (700 lines)
  * Migration: `20251120220000_create_gamification.sql` (233 lines)
- **Commits** (7 commits total):
  * f4cfd6f: WIP: Add Gamification & Achievements - Domain Entities (Issue #49 - Phase 6/6 - Part 1)
  * f13f240: Add Gamification DTOs + Repository Traits (Issue #49 - Phase 6/6 - Part 2)
  * bbc91bc: Add Gamification PostgreSQL Repositories (Issue #49 - Phase 6/6 - Part 3a)
  * 6500fa3: Add Gamification Use Cases (Issue #49 - Phase 6/6 - Part 3b)
  * 1c9086a: Add Gamification Migration + REST Handlers (Issue #49 - Phase 6/6 - Part 4)
  * d96fcc9: Wire Gamification Routes + AppState (Issue #49 - Phase 6/6 - Part 5a)
  * e9d37b8: feat: Complete Gamification & Achievements System (Issue #49 - Phase 6/6 - COMPLETE!)

**Key Implementation Patterns**:
- **Secret Achievement Visibility**: SQL LEFT JOIN to show non-secret achievements OR secret achievements the user has earned
- **Auto-Completion Logic**: When incrementing challenge progress, automatically mark as completed if current_value >= target_value
- **Multi-Source Leaderboard**: Aggregate points from both achievements (points_value × times_earned) and challenges (reward_points)
- **State Machine Validation**: Challenge state transitions enforced in domain layer (Draft → Active → Completed/Cancelled)
- **Repeatable Achievements**: times_earned counter with validation (must be >= 1)
- **Partial Index Optimization**: idx_challenges_active for common query (active challenges WHERE start_date <= NOW AND end_date > NOW)

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

### ✅ NOUVEAU: Board Decision Poll System - Issue #51 (Phase 2)

- Système de sondages pour consultations rapides entre assemblées générales (conforme à la loi belge)
- **Belgian Legal Context**: Article 577-8/4 §4 Code Civil Belge allows syndic consultations between general assemblies
- **4 Poll Types**:
  * **YesNo**: Simple yes/no decisions (e.g., "Should we repaint the lobby in blue?")
  * **MultipleChoice**: Choice between options (e.g., contractor selection) with single or multiple selection support
  * **Rating**: 1-5 star satisfaction surveys (configurable min/max rating)
  * **OpenEnded**: Free-text feedback collection (e.g., improvement suggestions)
- **Poll Lifecycle**: Draft → Active → Closed/Cancelled
  * **Draft**: Initial state, editable, not visible to owners
  * **Active**: Published poll, owners can vote, time-bound (starts_at to ends_at)
  * **Closed**: Voting ended, results calculated (winner, participation rate, vote breakdown)
  * **Cancelled**: Poll cancelled before completion
- **Anonymous Voting Support**:
  * `is_anonymous = true`: owner_id NULL in poll_votes, only ip_address stored for audit
  * `is_anonymous = false`: Full vote attribution with owner_id
  * Belgian privacy compliance (GDPR Article 6 consent-based processing)
- **Duplicate Vote Prevention**:
  * Database UNIQUE constraint on (poll_id, owner_id) for non-anonymous votes
  * Business logic validation for anonymous votes (IP-based rate limiting recommended)
  * Error response: "You have already voted on this poll"
- **Participation Tracking**:
  * `total_eligible_voters`: Count of active unit owners in building (TODO: replace hardcoded value)
  * `total_votes_cast`: Real-time vote count
  * `participation_rate`: (total_votes_cast / total_eligible_voters) * 100
- **Results Calculation**:
  * **YesNo**: Winner = option with most votes, percentage = (votes / total_votes_cast) * 100
  * **MultipleChoice**: Ranked options by vote_count DESC
  * **Rating**: Average rating + distribution histogram
  * **OpenEnded**: Text aggregation + export functionality
- **Multi-Select Support**: `allow_multiple_votes = true` for MultipleChoice polls (e.g., "Select up to 3 amenities")
- **Automatic Expiration**: Background job checks `ends_at <= NOW()` and auto-closes active polls
- **Domain Entities**:
  * `Poll` (572 lines): Core poll aggregate with validation (starts_at < ends_at, rating range 1-5, question non-empty)
  * `PollOption` (188 lines): Poll answer options with vote_count tracking
  * `PollVote` (214 lines): Individual votes with anonymization support
- **Repository Pattern**:
  * `PollRepository` trait (16 methods) - PostgresPollRepository
  * `PollOptionRepository` trait (12 methods) - PostgresPollOptionRepository
  * `PollVoteRepository` trait (10 methods) - PostgresPollVoteRepository
  * Complex queries: active polls, vote aggregation, duplicate detection, result calculation
- **Use Cases** (687 lines):
  * `PollUseCases` - 18 methods (CRUD, lifecycle transitions, voting, results)
  * Requires 3 repositories: Poll, PollOption, PollVote
  * Multi-repository orchestration for voting (check duplicate, increment vote_count, create vote record)
- **REST API** (12 endpoints, ~500 lines):
  * Poll Management: create, get, list (all/active/by-status), delete
  * Lifecycle: publish (Draft → Active), close (Active → Closed), cancel (→ Cancelled)
  * Voting: cast vote (with duplicate prevention), list votes (admin only)
  * Analytics: get results (vote counts, percentages, winner, participation rate)
- **Migration** (20251203120000_create_polls.sql, 156 lines):
  * Table: `polls` (16 columns, 7 indexes, 5 constraints)
  * Table: `poll_options` (7 columns, 3 indexes, 2 constraints)
  * Table: `poll_votes` (9 columns, 4 indexes, 3 constraints including UNIQUE(poll_id, owner_id) for duplicate prevention)
  * Custom ENUMs: `poll_type` (YesNo, MultipleChoice, Rating, OpenEnded), `poll_status` (Draft, Active, Closed, Cancelled)
  * Partial indexes: idx_polls_active (WHERE status = 'Active'), idx_polls_building_active (building_id, status WHERE status = 'Active')
  * Constraints: starts_at < ends_at, total_votes_cast >= 0, rating range validation
- **DTOs** (6 DTOs, ~180 lines):
  * `CreatePollDto`, `PublishPollDto`, `CastVoteDto`
  * `PollResponseDto`, `PollOptionResponseDto`, `PollVoteResponseDto`
  * `PollResultsDto` (winner, vote breakdown, participation rate)
- **Audit Events** (8 types, GDPR Article 30 compliance):
  * `PollCreated`, `PollPublished`, `PollClosed`, `PollCancelled`, `PollDeleted`
  * `PollVoteCast`, `PollResultsCalculated`, `PollExpired` (auto-close job)
- **Total Scope**:
  * ~2,500 lines of code across 4 layers (Domain, Application, Infrastructure, Migration)
  * 3 domain entities (24 unit tests)
  * 6 DTOs
  * 3 repository traits + 3 PostgreSQL implementations (38 methods total)
  * 1 use case class (18 methods)
  * 12 REST endpoints
  * 1 migration (3 tables, 2 ENUMs, 14 indexes, 10 constraints)
  * 8 audit event types
  * 1 BDD feature file (20 Gherkin scenarios covering all workflows)
- **Files Created**:
  * Domain: `poll.rs` (572 lines), `poll_option.rs` (188 lines), `poll_vote.rs` (214 lines)
  * DTOs: `poll_dto.rs` (~180 lines)
  * Ports: `poll_repository.rs`, `poll_option_repository.rs`, `poll_vote_repository.rs`
  * Implementations: `poll_repository_impl.rs` (511 lines), `poll_option_repository_impl.rs` (312 lines), `poll_vote_repository_impl.rs` (248 lines)
  * Use Cases: `poll_use_cases.rs` (687 lines)
  * Handlers: `poll_handlers.rs` (~500 lines)
  * Migration: `20251203120000_create_polls.sql` (156 lines)
  * BDD Tests: `backend/tests/features/polls.feature` (261 lines, 20 scenarios)
- **Commits** (4 commits):
  * 728548f: feat: Poll System - Domain, DTOs, Repositories (Issue #51 Part 1)
  * 48f18f4: feat: Poll System - Use Cases + REST Handlers (Issue #51 Part 2)
  * 23d75df: feat: Poll System - PostgreSQL Repository Implementations (Issue #51 Part 3)
  * d7c3e32: feat: Wire Poll System to AppState (Issue #51 Part 4)
- **TODO**:
  * Replace hardcoded `total_eligible_voters = 10` in poll_use_cases.rs:145 with proper unit_owners count query
  * Implement background job for auto-close expired polls (cron job checking ends_at <= NOW())
  * E2E tests deferred until repository pattern migration complete

**Key Implementation Patterns**:
- **State Machine Validation**: Poll status transitions enforced in domain layer (only Draft → Active, Active → Closed/Cancelled)
- **Duplicate Vote Prevention**: Database-level UNIQUE constraint + business logic validation
- **Anonymous Voting Privacy**: Conditional owner_id NULL with ip_address audit trail
- **Atomic Vote Casting**: Multi-repository transaction (create vote + increment option vote_count + increment poll total_votes_cast)
- **Belgian Legal Advisory**: Poll results documented in next general assembly minutes per Code Civil Article 577-8/4 §4

### Multi-owner support

- Junction table `unit_owners` (see `backend/migrations/20250127000000_refactor_owners_multitenancy.sql`) enables many-to-many between units and owners.
- Domain entity: `backend/src/domain/entities/unit_owner.rs` (pourcentage `0.0 < p ≤ 1.0`, timestamps, primary contact).
- Use cases: `backend/src/application/use_cases/unit_owner_use_cases.rs` (somme des quotes-parts ≤ 100 %, transfert, historique, contact principal unique).
- **✅ NOUVEAU: Validation stricte des quotes-parts** (Issue #29 - migration `20251120230000_add_unit_ownership_validation.sql`):
  * Règle métier belge: Total des quotes-parts actives = 100% (Article 577-2 §4 Code Civil)
  * Trigger PostgreSQL `validate_unit_ownership_total()` avec tolérance ±0.01% pour arrondis
  * BLOQUE les dépassements > 100% (erreur 23514: check_violation)
  * AVERTIT si < 100% (permet états transitoires lors d'ajouts séquentiels)
  * Validation uniquement sur propriétaires actifs (`end_date IS NULL`)
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
- regarde les logs docker compose logs backend qui fait des cargo watch plutot que de faire des cargo build ou cargo check
- il faut utiliser en mode dev localhost pour le frontend et localhost/api/v1 pour le backend