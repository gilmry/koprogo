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
