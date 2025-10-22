# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
