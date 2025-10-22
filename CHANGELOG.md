# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed - API Configuration Centralization (2025-10-22)

#### Frontend (Astro + Svelte)

- **Centralized API Configuration** (`src/lib/config.ts`)
  - Created configuration module that reads API URL from environment variables
  - `API_URL` constant with fallback to `http://127.0.0.1:8080` for development
  - `apiEndpoint(path)` helper function for constructing API endpoints
  - Supports SSR (Server-Side Rendering) with safe environment variable access

- **Environment Variables**
  - `.env` file with `PUBLIC_API_URL` variable
  - `.env.example` template for documentation
  - Production-ready: Change `PUBLIC_API_URL` to configure backend URL

- **Removed Hardcoded URLs**
  - `LoginForm.svelte` - Now uses `apiEndpoint('/api/v1/auth/login')`
  - `AdminDashboard.svelte` - Uses `apiEndpoint()` for seed endpoints
  - `BuildingList.svelte` - Removed local API_URL constant, uses `apiEndpoint()`
  - `sync.ts` - Reads from `API_URL` from config module
  - E2E tests - Created `tests/e2e/config.ts` with test-specific API configuration

- **Benefits**
  - Single source of truth for API URL configuration
  - Easy deployment to different environments (dev, staging, prod)
  - No code changes needed for deployment
  - Supports environment-specific configuration

**Migration Guide:**
- Development: No changes needed (defaults to http://127.0.0.1:8080)
- Production: Set `PUBLIC_API_URL=https://api.your-domain.com` in `.env`
- Docker: Pass `PUBLIC_API_URL` as environment variable

### Added - Database Seeding System (2025-10-22)

#### Backend (Rust/Actix-web)

- **Database Seeder Module** (`infrastructure/database/seed.rs`)
  - `DatabaseSeeder` class with comprehensive seeding capabilities
  - `seed_superadmin()` - Automatic SuperAdmin account creation on startup
    - Fixed UUID for SuperAdmin: `00000000-0000-0000-0000-000000000001`
    - Default credentials: admin@koprogo.com / admin123
  - `seed_demo_data()` - Creates complete demo dataset:
    - 1 Organization: "Copropriété Démo SAS"
    - 4 Users: Syndic, Accountant, 2 Owners with real credentials
    - 2 Buildings: "Résidence Les Champs" (Paris), "Le Jardin Fleuri" (Lyon)
    - 3 Owners with full contact details (address, city, postal_code)
    - 4 Units: Apartments with floor, surface_area, quota
    - 4 Expenses: Mixed paid/pending with suppliers and invoice numbers
  - `clear_demo_data()` - Removes all demo data while preserving SuperAdmin
    - Proper deletion order respecting FK constraints

- **API Endpoints** (`handlers/seed_handlers.rs`)
  - `POST /api/v1/seed/demo` - Seeds demo data (SuperAdmin only)
  - `POST /api/v1/seed/clear` - Clears demo data (SuperAdmin only)
  - JWT token verification with role check
  - Returns comprehensive success messages with credentials

- **Application Integration**
  - Updated `AppState` to include database pool for seeding operations
  - Automatic SuperAdmin seeding after migrations in `main.rs`
  - Logging of SuperAdmin creation success/failure

#### Frontend (Astro + Svelte)

- **AdminDashboard Enhancement**
  - New "Gestion de la base de données" section
  - "Générer les données" button with:
    - Real-time loading states
    - Display of created demo account credentials
    - Success/error message handling
  - "Supprimer les données" button with:
    - Confirmation dialog
    - Warning about data deletion
    - Visual feedback
  - Both buttons call real backend API with JWT authentication

- **LoginForm Cleanup**
  - Removed hardcoded demo users object
  - Updated to show only SuperAdmin credentials
  - Added note about generating demo data from dashboard

### Fixed - Docker Build & Rust Nightly (2025-10-22)

#### Backend Dockerfile
- **Rust Nightly Support**
  - Changed from `rust:1.83-slim` to `rustlang/rust:nightly-slim`
  - Resolves `base64ct-1.8.0` dependency requiring Rust edition 2024
  - Edition 2024 only available in nightly builds

- **SQLx Offline Mode**
  - Added `ENV SQLX_OFFLINE=true` to Dockerfile
  - Copied `.sqlx/` cache directory to Docker build context
  - Eliminates need for DATABASE_URL during Docker builds
  - Uses pre-generated query cache for compile-time verification

**Why these changes:**
- Some dependencies require Rust edition 2024 features
- SQLx macros need offline mode in Docker builds (no DB connection available)
- Production-ready: builds work without runtime database access

### Fixed - Docker & SSR Issues (2025-10-22)

#### Docker Build Fixes
- **Backend Dockerfile**
  - Added `COPY tests ./tests` to include BDD tests
  - Added `COPY benches ./benches` to include load tests
  - Resolved Cargo build errors for missing test/bench files

#### SSR (Server-Side Rendering) Fixes
- **Frontend `sync.ts`**
  - Protected `window` access with `typeof window !== 'undefined'` check
  - Protected `navigator` access with `typeof navigator !== 'undefined'` check
  - Event listeners only registered on client side
  - Resolved "window is not defined" errors during SSR

- **Frontend `db.ts`**
  - Protected `indexedDB` access with `typeof indexedDB === 'undefined'` check
  - Skip IndexedDB initialization on server side
  - Graceful degradation for SSR compatibility

#### Frontend Tests
- **E2E Tests (`dashboards.spec.ts`)**
  - Fixed TypeScript error: Changed `page.click().first()` to `page.locator().first().click()`
  - Proper Playwright API usage for element selection

### Changed - Database Schema Compliance (2025-10-22)

- **Seeding Queries Updated**
  - Owners table: Uses `address`, `city`, `postal_code`, `country` fields
  - Units table: Uses `surface_area`, `quota`, `floor`, `unit_type` ENUM
  - Expenses table: Uses `category`, `payment_status` ENUMs, `supplier`, `invoice_number`
  - Changed from `sqlx::query!()` to `sqlx::query()` for ENUM type compatibility

### Security - SuperAdmin Protection (2025-10-22)

- SuperAdmin-only endpoints with JWT verification
- Role-based access control for seeding operations
- Demo data deletion preserves SuperAdmin account
- Fixed UUID prevents accidental SuperAdmin deletion

### Demo Credentials Available After Seeding

```
SuperAdmin (always available):
- Email: admin@koprogo.com
- Password: admin123

Demo Users (created via seed):
- Syndic: syndic@copro-demo.fr / syndic123
- Comptable: comptable@copro-demo.fr / comptable123
- Propriétaire 1: proprietaire1@copro-demo.fr / owner123
- Propriétaire 2: proprietaire2@copro-demo.fr / owner123
```

---

### Added - Authentication & Multi-tenancy System

#### Backend (Rust/Actix-web)
- **Domain Layer**
  - `User` entity with role-based permissions (SuperAdmin, Syndic, Accountant, Owner)
  - `Organization` entity with subscription plans (Free, Starter, Professional, Enterprise)
  - Full validation and business logic in domain entities

- **Database**
  - SQL migrations for `users`, `organizations`, and `user_building_access` tables
  - Multi-tenancy support with organization isolation
  - User authentication with bcrypt password hashing

- **Repositories**
  - `PostgresUserRepository` with full CRUD operations
  - `PostgresOrganizationRepository` with slug-based lookup
  - Email-based user lookup for authentication

- **Application Layer**
  - `AuthUseCases` with login, register, and token verification
  - JWT token generation with 24-hour expiration
  - Password hashing with bcrypt (cost factor 12)
  - DTOs: `LoginRequest`, `RegisterRequest`, `LoginResponse`, `Claims`

- **API Endpoints**
  - `POST /api/v1/auth/register` - User registration
  - `POST /api/v1/auth/login` - User login with JWT token
  - `GET /api/v1/auth/me` - Get current user from token

#### Frontend (Astro + Svelte)

- **Authentication System**
  - Login page with real backend API integration
  - Auth store with localStorage and IndexedDB persistence
  - Automatic token management and refresh
  - Role-based redirects (SuperAdmin → /admin, Syndic → /syndic, etc.)

- **Multi-role Dashboards**
  - SuperAdmin Dashboard: Platform overview with organizations and users
  - Syndic Dashboard: Property management with buildings and tasks
  - Accountant Dashboard: Financial management with transactions
  - Owner Dashboard: Personal space for co-owners

- **Type System**
  - Complete TypeScript types for User, Building, Owner, Unit, Expense
  - Role-based permission helpers
  - User role enum (SUPERADMIN, SYNDIC, ACCOUNTANT, OWNER)

- **Navigation Component**
  - Dynamic menu based on user role
  - User profile dropdown
  - Logout functionality
  - Sync status indicator

### Added - PWA (Progressive Web App) Support

- **Service Worker**
  - Automatic installation via `@vite-pwa/astro`
  - Workbox strategies for caching
  - NetworkFirst strategy for API calls
  - Asset caching for offline support

- **Manifest**
  - PWA manifest with app icons
  - Standalone display mode
  - Theme colors and branding

- **Offline Functionality**
  - IndexedDB for local data storage
  - Automatic data synchronization
  - Online/offline detection
  - Queue for offline changes

- **Local Database (`src/lib/db.ts`)**
  - IndexedDB wrapper with CRUD operations
  - Object stores: users, buildings, owners, units, expenses, sync_queue
  - Sync queue for offline modifications
  - Helper methods for all entities

- **Sync Service (`src/lib/sync.ts`)**
  - Bidirectional synchronization with backend
  - Automatic sync when back online
  - Manual sync button
  - Fallback to local data when offline
  - Queue management for pending changes

- **UI Components**
  - `SyncStatus.svelte` - Online/offline indicator with animated LED
  - Manual sync button
  - Integrated into Navigation component

### Added - E2E Testing with Video Documentation

#### Playwright Configuration
- Complete Playwright setup with TypeScript
- Video recording enabled for ALL tests (documentation purpose)
- HTML report with embedded videos
- Screenshots on failure
- Trace collection for debugging

#### Test Suites (24 tests total)

**Authentication Tests (`auth.spec.ts`)** - 8 tests
- Landing page for unauthenticated users
- Navigation to login page
- Demo credentials display
- Successful login with backend API
- Error handling for invalid credentials
- Session persistence after page reload
- Logout functionality
- Role-based access (Syndic, Accountant, Owner, SuperAdmin)

**Dashboard Tests (`dashboards.spec.ts`)** - 8 tests
- Syndic dashboard with specific sections
- Navigation menu with role-specific items
- Navigation to buildings page
- User menu with profile options
- Accountant dashboard with financial focus
- Owner dashboard with personal information
- SuperAdmin dashboard with platform overview
- Navigation flow between pages

**PWA & Offline Tests (`pwa-offline.spec.ts`)** - 8 tests
- Valid manifest.json
- Service Worker registration
- Online status indicator
- Offline status detection
- IndexedDB usage
- Data caching in IndexedDB
- Manual synchronization
- Offline functionality after initial load

#### CI/CD Integration
- GitHub Actions workflow for E2E tests
- Automatic backend startup with PostgreSQL
- Video artifacts saved for 30 days
- HTML report artifacts
- PR comments with test results and video links
- Cross-browser testing support (Chromium, Firefox, WebKit)

### Added - Make Commands

#### Setup & Installation
- `make setup` - Complete project setup (dependencies + migrations + Playwright)
- `make install` - Install frontend dependencies
- `make install-all` - Install all dependencies including Playwright

#### Development
- `make dev-frontend` - Start frontend development server

#### E2E Testing Commands
- `make test-e2e-install` - Install Playwright browsers (run once)
- `make test-e2e-full` - Run full E2E tests with video generation
- `make test-e2e-ui` - Interactive UI mode
- `make test-e2e-headed` - Run tests with visible browser
- `make test-e2e-debug` - Step-by-step debug mode
- `make test-e2e-report` - Open HTML report with videos
- `make test-e2e-backend` - Run backend E2E tests only

#### Build & Clean
- `make clean` - Now also removes test-results and playwright-report

### Changed

- **Auth Store** - Updated to handle tokens and initialize IndexedDB
- **LoginForm** - Now calls real backend API instead of mock data
- **Navigation** - Added SyncStatus component
- **Main Backend** - Initialize AuthUseCases and add to AppState
- **Routes** - Added authentication endpoints
- **Makefile** - Updated help command to support numeric characters in target names
- **Test Command** - `make test` now includes full E2E tests

### Documentation Added

- `E2E_TESTING_GUIDE.md` - Complete guide for E2E testing
- `MAKEFILE_GUIDE.md` - Comprehensive Make commands documentation
- `frontend/tests/e2e/README.md` - Detailed E2E tests documentation
- `CHANGELOG.md` - This file

### Technical Details

#### Authentication Flow
1. User submits login form
2. Frontend calls `POST /api/v1/auth/login`
3. Backend validates credentials with bcrypt
4. Backend generates JWT token (24h expiration)
5. Frontend stores token in localStorage
6. Frontend initializes IndexedDB with user data
7. Sync service starts automatic synchronization

#### PWA Architecture
```
Browser → Service Worker (Workbox) → IndexedDB
                ↓                        ↓
          Backend API                Sync Queue
                ↓
          PostgreSQL
```

#### E2E Testing Architecture
```
Playwright Tests
      ↓
Service Worker → Frontend (Astro/Svelte) → Backend API → PostgreSQL
      ↓                                           ↑
IndexedDB ← Sync Service ───────────────────────┘
```

#### Video Documentation
- All E2E tests generate videos (1280x720, WebM format)
- Videos serve as living documentation
- Automatically uploaded to GitHub Actions artifacts
- Retained for 30 days
- Accessible via HTML report

### Migration Guide

#### For New Developers
```bash
git clone <repository>
cd koprogo
make setup          # Installs everything
make dev            # Start backend
make dev-frontend   # Start frontend (in another terminal)
```

#### For Existing Developers
```bash
git pull
cd frontend
npm install
npm run test:install     # Install Playwright
cd ../backend
sqlx migrate run         # Run new migrations
```

### Security Considerations

- Passwords hashed with bcrypt (cost factor 12)
- JWT tokens with 24-hour expiration
- HttpOnly cookies recommended for production
- CORS configured (currently allow-all for development)
- Input validation with `validator` crate
- SQL injection prevention with SQLx parameterized queries

### Performance Considerations

- Service Worker caches assets for instant loading
- IndexedDB for fast local data access
- NetworkFirst strategy reduces API calls
- Lazy loading of dashboard components
- Optimized Playwright tests (parallel execution)

### Browser Compatibility

- Chrome/Chromium: Full support
- Firefox: Full support (commented in Playwright config)
- Safari: Full support (commented in Playwright config)
- Mobile browsers: PWA installable

### Known Issues

- Service Worker only works in production build or with HTTPS in development
- Video recordings require system dependencies (installed via Playwright)
- IndexedDB not available in private/incognito mode

### Breaking Changes

None - This is the first release of these features.

### Deprecations

None

### Removed

None

### Fixed

- Backend compilation errors with auth handlers
- DbPool import path corrections
- Service Worker registration in development mode

---

## [Previous Versions]

See git history for previous changes.

---

**Note**: This changelog focuses on the authentication, PWA, and E2E testing features added in this release. For the complete project history, see the git commit log.
