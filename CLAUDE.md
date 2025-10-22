# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KoproGo is a SaaS property management platform built with **Hexagonal Architecture** (Ports & Adapters) and **Domain-Driven Design (DDD)**. The system emphasizes performance (P99 < 5ms latency), testability, security (GDPR compliant), and ecological sustainability (< 0.5g CO2/request target).

**Stack**: Rust + Actix-web (backend), Astro + Svelte (frontend), PostgreSQL 15

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
**Units**: `/units` (GET, POST), `/buildings/:id/units` (GET), `/units/:id/assign-owner/:owner_id` (PUT)
**Owners**: `/owners` (GET, POST), `/owners/:id` (GET)
**Expenses**: `/expenses` (GET, POST), `/buildings/:id/expenses` (GET), `/expenses/:id/mark-paid` (PUT)
**Health**: `/health` (GET)

## Domain Entities

The system models property management with these aggregates:

- **Building**: Main aggregate (name, address, total_units, construction_year)
- **Unit**: Lots within buildings (unit_number, floor, area, owner relationship)
- **Owner**: Co-owners (name, email, phone, GDPR-sensitive data)
- **Expense**: Charges (amount, description, due_date, paid status)
- **Meeting**: General assemblies (date, agenda, minutes)
- **Document**: File storage (title, file_path, document_type)

All entities use UUID for IDs and include `created_at`/`updated_at` timestamps.

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

**Current branch**: `claude/add-ci-workflows-011CUMhvUnFsKBJoJ9rbWXoN`
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
