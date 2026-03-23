===========================================================
Tauri Architecture — KoproGo Desktop & Mobile App
===========================================================

:Date: March 2026
:Version: 1.0.0 - **ARCHITECTURAL DESIGN** (Not yet implemented)
:Related Issues: #295 (Workspace Setup), #296 (SQLite Adapters), #297 (Offline Sync), #298 (Mobile), #299 (Desktop)
:Status: Architecture Documentation
:Author: Claude Code (Anthropic)

.. contents:: Table of Contents
   :depth: 3

Overview
========

KoproGo is expanding from a web-only SaaS platform to include native desktop and mobile applications using **Tauri 2.0**. This architecture document describes:

1. **Why Tauri** over competing frameworks
2. **Cargo workspace restructuring** to support multiple distribution channels
3. **SQLite adapter pattern** for offline-first local storage
4. **Offline sync engine** for conflict-free data synchronization
5. **Authentication & security** for native apps
6. **Mobile deployment** strategy for iOS and Android

This document guides the implementation of issues #295–#299 (Tauri Phase 1–2).

Why Tauri?
----------

Tauri is chosen over alternatives for the following reasons:

**vs. Electron**

* **Size**: Tauri apps are 10–100x smaller than Electron (50–150 MB vs. 500+ MB)

  * No bundled Chromium; uses system WebView
  * KoproGo desktop would be ~60 MB instead of 500+ MB

* **Performance**: Native WebView rendering + Rust core = P99 < 16ms for local operations
* **Memory footprint**: 30–80 MB vs. 200–500 MB for Electron
* **Update speed**: Differential updates only (~1–5 MB per release)
* **Security posture**: Rust memory safety; no Node.js attack surface

**vs. React Native / Flutter**

* **Code sharing**: Tauri allows sharing **Rust domain logic** with the backend

  * Entities, validation rules, business logic can be compiled to iOS/Android
  * React Native requires JavaScript; Flutter requires Dart
  * KoproGo can reuse `Building`, `Unit`, `Owner`, `Expense` entities across all platforms

* **Single codebase**: One Rust core + one TypeScript frontend (Svelte)

  * React Native requires React for mobile + separate web stack
  * Flutter is entirely separate from existing Rust/TypeScript stack

* **Offline-first architecture**: Tauri's SQLite integration is native; React Native requires external libraries
* **Cost**: React Native and Flutter require specialized developer skills; Tauri leverages existing Rust + TypeScript expertise

**vs. PWA (Progressive Web App)**

* **Native features**: Access to OS keychain, file system, notifications without WebView limitations
* **Offline-first**: SQLite local database; PWA relies on IndexedDB (limited to ~50 MB in most browsers)
* **Distribution**: App stores (Windows, macOS, Linux, iOS, Android); PWA only via web
* **Performance**: Native binary execution; PWA is browser-dependent

**Tauri 2.0 advantages** (2025 release):

* ✅ iOS support (alpha → stable in Q1 2025)
* ✅ Android support (stable as of Tauri 2.0)
* ✅ Unified desktop + mobile codebase
* ✅ SQLite with offline sync out of the box
* ✅ Built-in plugin ecosystem (keychain, notifications, file system, camera, geolocation)
* ✅ Rust security guarantees (no unsafe code in core)

Architecture Overview
======================

**Philosophy: Rust Core + TypeScript UI + SQLite Local State**

.. code-block:: text

    ┌─────────────────────────────────────────────────────────┐
    │                    Tauri Application                    │
    ├─────────────────────────────────────────────────────────┤
    │                                                          │
    │  ┌──────────────────────────────────────────────────┐   │
    │  │        TypeScript UI Layer (Svelte)              │   │
    │  │  - Responsive layout (desktop/mobile)            │   │
    │  │  - Stores (sync status, offline indicators)      │   │
    │  │  - Real-time conflict resolution UI              │   │
    │  └──────────────────────────────────────────────────┘   │
    │                        ▲ / ▼ (IPC)                      │
    │  ┌──────────────────────────────────────────────────┐   │
    │  │      Rust Backend (Tauri Commands)               │   │
    │  │  - Domain entity validation                       │   │
    │  │  - Sync queue management                         │   │
    │  │  - Offline/online detection                      │   │
    │  │  - Conflict resolution logic                     │   │
    │  └──────────────────────────────────────────────────┘   │
    │                        ▲ / ▼ (SQL)                      │
    │  ┌──────────────────────────────────────────────────┐   │
    │  │      SQLite Local Database                       │   │
    │  │  - Encrypted with SQLCipher (SQLite 3.45+)      │   │
    │  │  - Synced data mirror (buildings, units, etc.)   │   │
    │  │  - Sync queue (pending operations)               │   │
    │  │  - Metadata (sync timestamps, hashes)            │   │
    │  └──────────────────────────────────────────────────┘   │
    │                        ▲ / ▼ (HTTP)                     │
    │  ┌──────────────────────────────────────────────────┐   │
    │  │   KoproGo Backend (REST API / WebSocket)         │   │
    │  │  - Source of truth                               │   │
    │  │  - Receives sync operations from app             │   │
    │  │  - Pushes notifications to app                   │   │
    │  └──────────────────────────────────────────────────┘   │
    │                                                          │
    └─────────────────────────────────────────────────────────┘

Cargo Workspace Restructuring
=============================

Current structure (web-only):

.. code-block:: text

    koprogo/
    ├── backend/           # Actix-web REST API (PostgreSQL)
    │   └── src/
    │       ├── domain/
    │       ├── application/
    │       └── infrastructure/
    ├── frontend/          # Astro + Svelte (browser)
    └── docs/

**New structure (web + desktop + mobile)**:

.. code-block:: text

    koprogo/
    ├── crates/                          # Shared Rust libraries (workspace root)
    │   ├── koprogo-domain/              # Domain entities (no external deps)
    │   │   └── src/
    │   │       ├── entities/            # Building, Unit, Owner, etc.
    │   │       ├── services/            # Domain services
    │   │       └── value_objects/       # UUID, Money, QuotePercentage, etc.
    │   │
    │   ├── koprogo-api-client/          # HTTP client for remote server
    │   │   └── src/
    │   │       ├── client.rs            # Reqwest-based HTTP client
    │   │       ├── models.rs            # DTOs matching backend API contracts
    │   │       └── error.rs             # Unified error types
    │   │
    │   ├── koprogo-sqlite/              # SQLite adapters (implements same ports as PostgreSQL)
    │   │   └── src/
    │   │       ├── connection.rs        # SQLite connection pool
    │   │       ├── repositories/        # BuildingRepository, UnitRepository, etc. (SQLite impls)
    │   │       ├── migrations.rs        # Local SQLite schema management
    │   │       └── encryption.rs        # SQLCipher integration
    │   │
    │   └── koprogo-sync/                # Offline sync engine
    │       └── src/
    │           ├── queue.rs             # Sync queue management
    │           ├── conflict.rs          # Conflict resolution
    │           ├── merkle.rs            # Merkle tree for efficient sync
    │           └── integration.rs       # Integration with api-client & sqlite
    │
    ├── apps/
    │   ├── backend/                     # Actix-web REST API (PostgreSQL) — unchanged
    │   │   └── src/
    │   │
    │   ├── web/                         # Astro + Svelte browser app — unchanged
    │   │   └── src/
    │   │
    │   ├── desktop/                     # Tauri desktop app (Windows/macOS/Linux)
    │   │   ├── src-tauri/               # Rust Tauri commands
    │   │   │   └── src/
    │   │   │       ├── main.rs
    │   │   │       ├── commands/        # Tauri command handlers
    │   │   │       ├── state.rs         # AppState (repositories, sync engine)
    │   │   │       └── events.rs        # Custom Tauri events
    │   │   │
    │   │   └── src/                     # TypeScript/Svelte UI
    │   │       ├── lib/
    │   │       │   ├── api.ts           # Tauri IPC calls
    │   │       │   ├── stores/          # Svelte stores (sync status, user data)
    │   │       │   └── utils/
    │   │       ├── components/          # Svelte components
    │   │       ├── pages/
    │   │       └── App.svelte
    │   │
    │   └── mobile/                      # Tauri mobile app (iOS/Android)
    │       ├── src-tauri/               # Same as desktop
    │       └── src/                     # Responsive UI + touch optimizations
    │
    └── docs/
        ├── TAURI_ARCHITECTURE.rst       # This file
        ├── TAURI_OFFLINE_SYNC.rst       # Sync engine details
        └── ...

Workspace Configuration (`Cargo.toml` at root):

.. code-block:: toml

    [workspace]
    members = [
        "crates/koprogo-domain",
        "crates/koprogo-api-client",
        "crates/koprogo-sqlite",
        "crates/koprogo-sync",
        "apps/backend",
        "apps/web",
        "apps/desktop",
        "apps/mobile",
    ]
    resolver = "2"

    [workspace.package]
    version = "0.1.0"
    edition = "2021"
    license = "AGPL-3.0-or-later"

Domain Crate: `koprogo-domain`
==============================

**Purpose**: Pure Rust domain entities and business logic, zero external dependencies (except `uuid`, `chrono`, `serde`).

**Rationale**: Share domain logic across backend API, desktop app, and mobile app without compilation overhead.

**Dependencies**:

.. code-block:: toml

    [dependencies]
    uuid = { version = "1.11", features = ["serde", "v4"] }
    chrono = { version = "0.4", features = ["serde"] }
    serde = { version = "1.0", features = ["derive"] }

**Module structure**:

.. code-block:: text

    src/
    ├── lib.rs                  # Public exports
    ├── entities/               # Domain aggregates (no persistence logic)
    │   ├── building.rs         # Building aggregate
    │   ├── unit.rs
    │   ├── owner.rs
    │   ├── unit_owner.rs       # Multi-owner support
    │   ├── expense.rs
    │   ├── ticket.rs
    │   └── ...
    ├── services/               # Domain services (coordination across aggregates)
    │   ├── unit_owner_validator.rs  # Validates ownership percentages sum to 100%
    │   ├── expense_calculator.rs    # VAT calculations, charge distribution
    │   └── ...
    ├── value_objects/          # Immutable value types
    │   ├── money.rs            # Amount in cents + currency
    │   ├── quote_percentage.rs  # 0.0 < p ≤ 1.0
    │   └── ...
    └── errors.rs               # Unified error enum

**Benefits**:

1. **No network or database dependencies** → Fast compilation, testable without infrastructure
2. **Reusable across all platforms** → Desktop, mobile, backend share validation logic
3. **Type safety** → Rust compiler enforces invariants (e.g., ownership % validation)
4. **No serialization/deserialization** in domain layer → Clean separation of concerns

API Client Crate: `koprogo-api-client`
=======================================

**Purpose**: Async HTTP client for the KoproGo backend REST API.

**Dependencies**:

.. code-block:: toml

    [dependencies]
    reqwest = { version = "0.11", features = ["json", "cookies"] }
    serde_json = "1.0"
    tokio = { version = "1.50", features = ["macros", "rt"] }
    thiserror = "1.0"
    koprogo-domain = { path = "../koprogo-domain" }

**Module structure**:

.. code-block:: rust

    // src/lib.rs
    pub mod client;     // HTTP client
    pub mod models;     // DTOs matching backend API
    pub mod auth;       // JWT token management
    pub mod error;      // Unified error types

    // src/client.rs
    pub struct ApiClient {
        http: reqwest::Client,
        base_url: String,
        jwt_token: Option<String>,
    }

    impl ApiClient {
        pub async fn get_buildings(&self, org_id: Uuid) -> Result<Vec<BuildingDto>> { ... }
        pub async fn create_expense(&self, expense: CreateExpenseDto) -> Result<ExpenseDto> { ... }
        pub async fn sync_changes(&self, sync_ops: Vec<SyncOperation>) -> Result<SyncResponse> { ... }
        // ... 50+ methods mapping to backend endpoints
    }

**Key operations**:

1. **Authentication**: Login returns JWT; stored in OS keychain (via `tauri-plugin-keychain`)
2. **Data sync**: Batch API for uploading local changes and downloading remote changes
3. **Conflict resolution hints**: Server returns `last_modified_at` timestamps for conflict detection
4. **Retry logic**: Exponential backoff with Jitter for resilience

SQLite Adapter Crate: `koprogo-sqlite`
=======================================

**Purpose**: Implement the same repository traits as PostgreSQL backend, but backed by SQLite.

**Dependencies**:

.. code-block:: toml

    [dependencies]
    sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "sqlite"] }
    sqlcipher = "3.45"  # SQLite with encryption
    tokio = { version = "1.50", features = ["macros", "rt"] }
    uuid = { version = "1.11", features = ["serde"] }
    chrono = { version = "0.4", features = ["serde"] }
    koprogo-domain = { path = "../koprogo-domain" }

**Architecture**:

Tauri Offline Architecture follows the **Hexagonal/Ports & Adapters** pattern:

.. code-block:: text

    ┌─────────────────────────────────────────┐
    │   Tauri Rust Commands (Use Cases)       │  (Application Layer)
    │   - create_building_offline()            │
    │   - sync_pending_changes()               │
    └──────────────┬──────────────────────────┘
                   │
                   ├─ depends on BuildingRepository trait (Port)
                   │
    ┌──────────────▼──────────────────────────┐
    │  Repository Trait (Port Definition)      │  (Boundary)
    │  pub trait BuildingRepository {           │
    │    async fn create(...) -> Result<...>;  │
    │    async fn find_by_id(...) -> Result;   │
    │  }                                       │
    └──────────────────────────────────────────┘
           ▲                      ▲
           │                      │
    ┌──────┴────────────┐  ┌──────┴──────────────────┐
    │  PostgreSQL Impl  │  │  SQLite Impl (Local)   │  (Adapter Layer)
    │  (Backend API)    │  │  (Offline-first)       │
    └───────────────────┘  └────────────────────────┘

The same `BuildingRepository` trait has two implementations:

1. **`PostgresBuildingRepository`**: HTTP calls to backend (used in `apps/backend`)
2. **`SqliteBuildingRepository`**: SQLite local database (used in `apps/desktop` and `apps/mobile`)

**Module structure**:

.. code-block:: text

    src/
    ├── lib.rs                       # Public exports
    ├── connection.rs                # SQLite connection pool & encryption
    ├── repositories/                # Repository implementations
    │   ├── mod.rs
    │   ├── building_repository.rs   # impl BuildingRepository for SqliteBuildingRepository
    │   ├── unit_repository.rs
    │   ├── owner_repository.rs
    │   ├── expense_repository.rs
    │   └── ...
    ├── migrations.rs                # Local SQLite schema
    ├── encryption.rs                # SQLCipher setup
    └── schema.sql                   # DDL for local database

**SQLite Schema** (key tables):

.. code-block:: sql

    -- Core synced data
    CREATE TABLE buildings (
        id TEXT PRIMARY KEY,
        organization_id TEXT NOT NULL,
        name TEXT NOT NULL,
        address TEXT NOT NULL,
        created_at DATETIME NOT NULL,
        updated_at DATETIME NOT NULL,
        -- Sync metadata
        synced_at DATETIME,              -- Last successful sync from server
        local_version INTEGER DEFAULT 0, -- Local change counter
        server_hash TEXT,                -- Hash of server version for conflict detection
        FOREIGN KEY (organization_id) REFERENCES organizations(id)
    );

    -- Similar for units, owners, unit_owners, expenses, tickets, etc.

    -- Sync queue: pending local changes
    CREATE TABLE sync_queue (
        id TEXT PRIMARY KEY,
        entity_type TEXT NOT NULL,       -- 'building', 'expense', 'ticket', etc.
        entity_id TEXT NOT NULL,
        operation TEXT NOT NULL,         -- 'create', 'update', 'delete'
        payload JSON NOT NULL,           -- Serialized entity data
        created_at DATETIME NOT NULL,
        synced_at DATETIME,              -- NULL until successfully synced
        retry_count INTEGER DEFAULT 0,
        last_error TEXT,
        FOREIGN KEY (entity_id) REFERENCES [table_name](id)
    );

    -- Metadata for tracking sync state
    CREATE TABLE sync_metadata (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at DATETIME NOT NULL
    );
    -- Keys: 'last_sync_timestamp', 'merkle_tree_root', 'device_id', etc.

**Encryption**:

SQLCipher provides transparent AES-256 encryption:

.. code-block:: rust

    // src/connection.rs
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    pub async fn create_encrypted_pool(db_path: &str, password: &str) -> Result<SqlitePool> {
        let connect_options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path))?
            .pragma("key", format!("'{}'", password))  // SQLCipher key
            .pragma("cipher_page_size", "4096")
            .pragma("cipher_kdf_iter", "256000")        // PBKDF2 iterations
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(connect_options)
            .await?;

        Ok(pool)
    }

**Password management**:

- On app launch: Generate random 32-character password, store in OS keychain
- On logout: Wipe password from memory and keychain
- On app restart: Retrieve password from keychain, decrypt local database
- On GDPR erasure: Overwrite database with random data before deletion (secure wipe)

Offline Sync Crate: `koprogo-sync`
==================================

**Purpose**: Orchestrate synchronization between local SQLite and remote KoproGo backend.

**Dependencies**:

.. code-block:: toml

    [dependencies]
    koprogo-domain = { path = "../koprogo-domain" }
    koprogo-api-client = { path = "../koprogo-api-client" }
    koprogo-sqlite = { path = "../koprogo-sqlite" }
    tokio = { version = "1.50", features = ["macros", "rt", "time"] }
    uuid = { version = "1.11", features = ["v4"] }
    chrono = { version = "0.4", features = ["serde"] }

**Key modules** (see `TAURI_OFFLINE_SYNC.rst` for detailed design):

.. code-block:: text

    src/
    ├── lib.rs
    ├── queue.rs              # Sync queue operations (pop, push, retry)
    ├── conflict.rs           # Conflict detection & resolution (last-write-wins)
    ├── merkle.rs             # Merkle tree for efficient sync (not Phase 1)
    ├── integration.rs        # SyncEngine orchestrator
    └── models.rs             # SyncOperation, SyncResponse, etc.

**SyncEngine** (orchestrator):

.. code-block:: rust

    pub struct SyncEngine {
        api_client: ApiClient,
        sqlite_pool: SqlitePool,
        sync_queue: SyncQueueRepository,
        building_repo: SqliteBuildingRepository,
        // ... other repositories
    }

    impl SyncEngine {
        /// Called when device comes online
        pub async fn full_sync(&self) -> Result<SyncStats> {
            // 1. Compress local queue (deduplicate operations)
            // 2. Upload pending changes to backend
            // 3. Download remote changes
            // 4. Merge (with conflict resolution)
            // 5. Update local database
            // 6. Mark sync_queue entries as synced_at = NOW()
        }

        /// Called every 5 minutes (or before logout)
        pub async fn incremental_sync(&self) -> Result<SyncStats> {
            // Lightweight: only new local changes + recent remote changes
        }

        /// Detect and resolve conflicts
        async fn detect_conflicts(&self) -> Result<Vec<Conflict>> {
            // For each pending local change, check if entity was modified on server
            // Return conflicts for UI to display (show both versions)
        }
    }

Desktop App: `apps/desktop`
===========================

**Target platforms**: Windows 10+, macOS 11+, Linux (Ubuntu 18.04+, Fedora 30+)

**Technology stack**:

- **Rust core** (Tauri 2.0): Commands, state management, native features
- **TypeScript/Svelte UI**: Responsive desktop layout
- **Plugins**: keychain, notifications, file system, updater

**Directory structure**:

.. code-block:: text

    apps/desktop/
    ├── src-tauri/                      # Rust Tauri backend
    │   ├── Cargo.toml                  # Depends on all shared crates
    │   ├── tauri.conf.json             # Tauri configuration
    │   └── src/
    │       ├── main.rs
    │       ├── commands.rs             # Tauri command handlers
    │       │   ├── building_commands.rs
    │       │   ├── sync_commands.rs
    │       │   ├── auth_commands.rs
    │       │   └── ...
    │       ├── state.rs                # AppState struct
    │       └── events.rs               # Custom Tauri events
    │
    ├── src/                            # TypeScript/Svelte UI
    │   ├── lib/
    │   │   ├── api.ts                  # IPC calls to Rust backend
    │   │   ├── stores.ts               # Svelte stores
    │   │   │   ├── authStore.ts        # User session
    │   │   │   ├── buildingStore.ts    # Cached buildings
    │   │   │   ├── syncStore.ts        # Sync status
    │   │   │   └── uiStore.ts          # UI state (dark mode, layout)
    │   │   └── utils.ts
    │   ├── components/
    │   │   ├── Sidebar.svelte          # Desktop navigation
    │   │   ├── BuildingList.svelte
    │   │   ├── ExpenseForm.svelte      # Offline-capable form
    │   │   ├── SyncStatus.svelte       # Show online/offline/syncing state
    │   │   └── ConflictResolver.svelte # Conflict UI (rare)
    │   ├── pages/
    │   │   ├── +page.svelte            # Dashboard
    │   │   ├── buildings/+page.svelte
    │   │   └── ...
    │   ├── App.svelte                  # Root layout
    │   └── app.css                     # Tailwind CSS
    │
    ├── package.json
    ├── svelte.config.js
    ├── tsconfig.json
    └── vite.config.ts

**Key Tauri commands**:

.. code-block:: rust

    // src-tauri/src/commands.rs
    #[tauri::command]
    pub async fn login(
        email: String,
        password: String,
        state: tauri::State<'_, AppState>,
    ) -> Result<LoginResponse, String> {
        // 1. Call api_client.login(email, password)
        // 2. Store JWT in OS keychain (via tauri-plugin-keychain)
        // 3. Initialize sync engine
        // 4. Return user + organization info
    }

    #[tauri::command]
    pub async fn get_buildings(
        state: tauri::State<'_, AppState>,
    ) -> Result<Vec<BuildingDto>, String> {
        // 1. Query local SQLite: SELECT * FROM buildings
        // 2. Add sync status (synced_at timestamp)
        // 3. Return to UI
    }

    #[tauri::command]
    pub async fn create_building(
        building: CreateBuildingDto,
        state: tauri::State<'_, AppState>,
    ) -> Result<BuildingDto, String> {
        // 1. Validate via domain entity: Building::new(...)
        // 2. Insert into SQLite
        // 3. Add to sync_queue with operation='create'
        // 4. If online, trigger immediate sync
        // 5. Emit 'building-created' event
    }

    #[tauri::command]
    pub async fn sync_pending_changes(
        state: tauri::State<'_, AppState>,
    ) -> Result<SyncStats, String> {
        // Call SyncEngine::full_sync() or incremental_sync()
        // Emit 'sync-progress' event during upload/download
        // On error, emit 'sync-error' event
    }

Mobile App: `apps/mobile`
=========================

**Target platforms**: iOS 13+, Android 8+ (API level 26+)

**Differences from desktop**:

1. **UI**: Touch-optimized (larger buttons, swipe navigation)
2. **Permissions**: Camera, geolocation, contacts (with user consent)
3. **Notch handling**: Safe area insets for notched phones
4. **Connectivity**: Aggressive background sync (Tauri background tasks)
5. **File storage**: App sandbox (no direct file system access)

**Directory structure**: Same as `apps/desktop`, with responsive UI adjustments.

**Mobile-specific features**:

.. code-block:: rust

    // src-tauri/src/commands.rs — mobile extensions

    #[tauri::command]
    pub async fn get_location() -> Result<Location> {
        // Use tauri-plugin-geolocation
        // Request location permission via system dialog
    }

    #[tauri::command]
    pub async fn take_photo() -> Result<String> {
        // Camera: Use tauri-plugin-camera (Phase 2)
        // Return base64-encoded image path
    }

    #[tauri::command]
    pub async fn enable_background_sync(enabled: bool) {
        // Configure iOS background fetch or Android WorkManager
        // Sync every 15 minutes in background (if online)
    }

**iOS-specific setup**:

.. code-block:: toml

    # Cargo.toml
    [package.metadata.ios]
    deployment_target = "13.0"
    bundle_identifier = "com.koprogo.app"
    team_id = "XXXXXXXXXX"  # Apple Developer Team ID

**Android-specific setup**:

.. code-block:: toml

    # src-tauri/tauri.conf.json
    {
      "android": {
        "useBootFramework": true,
        "minSdkVersion": 26,
        "targetSdkVersion": 34
      }
    }

Authentication & Security
==========================

**JWT Token Management**:

1. **Login**: User enters email + password → API returns JWT (valid 1 hour)
2. **Storage**: JWT stored in OS keychain (encrypted at rest)
3. **Refresh**: Refresh token stored in SQLite (encrypted); used to get new JWT on expiry
4. **Logout**: Clear JWT from keychain + delete refresh token from database + wipe local data (optional)

.. code-block:: rust

    // src-tauri/src/commands/auth_commands.rs
    #[tauri::command]
    pub async fn login(email: String, password: String) -> Result<LoginResponse> {
        let response = api_client.login(email, password).await?;

        // Store JWT in OS keychain
        window.invoke('cmd', {
            cmd: 'store_jwt_in_keychain',
            token: response.jwt,
        }).await;

        Ok(LoginResponse {
            user_id: response.user_id,
            organization_id: response.organization_id,
        })
    }

**Password security**:

- Minimum 12 characters (enforced by backend API)
- No password stored locally (only JWT)
- Bcrypt hashing on backend (Issue #36)

**Encrypted local database**:

- SQLCipher encrypts entire SQLite file with AES-256
- Password: Random 32-char string stored in OS keychain
- User cannot access local data without unlocking app (biometric or PIN on mobile)

**Biometric authentication** (Phase 2):

.. code-block:: rust

    // src-tauri/src/commands/auth_commands.rs (Phase 2)
    #[tauri::command]
    pub async fn enable_biometric_unlock() -> Result<bool> {
        use tauri_plugin_biometric::*;

        let is_available = biometric::is_available().await?;
        if is_available {
            // Prompt user to scan fingerprint/face
            // On success: Unlock app without requiring password
        }
        Ok(is_available)
    }

**GDPR compliance**:

- User can request data export → Download ZIP of all local data
- User can request erasure → Secure wipe of local database + call backend DELETE /gdpr/erase
- User logout → Clear local data (configurable: keep cache or full wipe)

Push Notifications
==================

**Architecture**:

1. **Backend** (existing Notification system, Issue #86): Sends notifications via Firebase Cloud Messaging (FCM)
2. **Mobile app**: Registers FCM token with backend on startup
3. **Desktop app**: Uses OS notification service (Windows Toast, macOS Notification Center)

**Implementation**:

.. code-block:: rust

    // src-tauri/src/commands.rs
    #[tauri::command]
    pub async fn register_push_notifications(
        device_type: String,  // 'ios', 'android', 'desktop'
        fcm_token: String,
    ) -> Result<()> {
        // Call backend: POST /notifications/device-tokens
        // Backend stores device token per user
    }

    #[tauri::command]
    pub async fn handle_notification(notification: NotificationPayload) {
        // Called when app receives push notification
        // Parse payload: { type: 'payment-due', expense_id: 'xxx', amount: 500.00 }
        // Update local store + emit IPC event to UI
        // Navigate to relevant page (expense detail, meeting agenda, etc.)
    }

**Tauri plugins**:

- `tauri-plugin-notification`: Desktop notifications (Windows, macOS)
- Firebase Cloud Messaging SDK (iOS/Android): Mobile push notifications
- Both integrated via tauri-plugin-notification wrapper

Migration Path: Web → Tauri
============================

**Phase 1 (Q2 2026): Desktop MVP**

.. code-block:: text

    1. Restructure workspace (Issues #295)
       └─ Create crates/ folder
       └─ Move domain to koprogo-domain (no db deps)
       └─ Create koprogo-api-client, koprogo-sqlite, koprogo-sync crates

    2. Implement SQLite adapters (Issue #296)
       └─ Build BuildingRepository, UnitRepository, etc. for SQLite
       └─ Implement sync_queue table
       └─ Add encryption (SQLCipher)

    3. Implement offline sync (Issue #297)
       └─ Build SyncEngine (queue operations, conflict resolution)
       └─ Implement merkle tree (optional, Phase 2)

    4. Build desktop Tauri app (Issue #299)
       └─ Create apps/desktop with Tauri 2.0
       └─ Implement Tauri commands for core workflows
       └─ Build Svelte UI (desktop layout)
       └─ Add auth (JWT + keychain)
       └─ Test on Windows/macOS/Linux

    5. Add push notifications (Phase 1b)
       └─ Windows Toast notifications
       └─ macOS Notification Center

**Phase 2 (Q3–Q4 2026): Mobile**

.. code-block:: text

    1. Build mobile app (Issue #298)
       └─ Create apps/mobile with same Rust core
       └─ Responsive UI for mobile (touch optimizations)
       └─ Platform-specific: iOS Xcode, Android Android Studio

    2. Mobile-specific features
       └─ Biometric unlock (fingerprint, Face ID)
       └─ Camera integration (take photos of work)
       └─ Geolocation (contractor location tracking)
       └─ Firebase Cloud Messaging (push notifications)
       └─ Background sync (iOS background fetch, Android WorkManager)

    3. App store distribution
       └─ Windows Store, macOS App Store, iOS App Store, Google Play

**Phase 3+ (2027): Advanced features**

- Real-time collaborative editing (WebSocket + Operational Transformation)
- Blockchain-verified voting (Issue #46 extension)
- IoT device integration (Linky, door locks, thermostats)
- AR viewing (visualize building floor plans)

Testing Strategy
================

**Unit tests** (domain layer):

.. code-block:: bash

    # Test domain entities in isolation
    cargo test -p koprogo-domain

**Integration tests** (SQLite + API client):

.. code-block:: bash

    # Test SQLite repository implementations
    cargo test -p koprogo-sqlite -- --test-threads 1

**E2E tests** (Tauri app):

.. code-block:: bash

    # Test Tauri commands and UI
    cargo test -p koprogo-desktop

**Offline scenario tests**:

.. code-block:: text

    Test 1: Create building offline, then sync
    Test 2: Modify building locally, then sync (no conflicts)
    Test 3: Sync error → retry on reconnect
    Test 4: Conflict detection: local & server both modified
    Test 5: Logout → clear local data
    Test 6: Network switch: WiFi → cellular → offline → WiFi

Deployment & Distribution
==========================

**Desktop**:

- **Windows**: MSI installer via Windows App Installer or direct .exe
- **macOS**: Universal binary (Intel + ARM64), DMG or App Store
- **Linux**: AppImage, snap, deb, rpm

**Mobile**:

- **iOS**: TestFlight (beta) → Apple App Store
- **Android**: Google Play Console (beta) → Google Play

**Update strategy**:

- Tauri built-in updater: Check backend for new versions
- Differential updates: Only download changed files (~1–5 MB per update)
- Auto-update on app restart or manual "Check for Updates" button

**Signing & notarization**:

- **Windows**: Code signing with EV certificate (Sectigo or similar)
- **macOS**: Notarization via Apple Developer account (required for Big Sur+)
- **iOS**: App Store review process
- **Android**: Signed APK/AAB with keystore

Performance Targets
===================

**Local operations** (offline):

- P99 < 5ms for list operations (buildings, units)
- P99 < 10ms for create/update operations
- P99 < 50ms for complex queries (expenses with owner details)

**Sync operations** (online):

- Upload 1,000 local changes: < 10 seconds
- Download 1,000 remote changes: < 10 seconds
- Conflict detection: < 5 seconds for 1,000 changes

**Memory footprint**:

- Desktop app: < 200 MB RAM (including SQLite + cache)
- Mobile app: < 150 MB RAM
- SQLite database: < 100 MB for typical organization (5,000 units)

**Storage**:

- Desktop app binary: ~ 60 MB
- Mobile app binary: ~ 80 MB (iOS), ~ 120 MB (Android)
- Local SQLite database: Configurable, ~100 MB typical

Known Limitations & Future Work
================================

**Phase 1 (Current)**:

- ❌ No real-time collaboration (CRDT support deferred to Phase 2)
- ❌ No offline voting (requires client-side voting, then sync & conflict resolution)
- ❌ No offline document uploads (large files require server connectivity)
- ❌ Limited mobile platform (iOS/Android in Tauri alpha; target stable Q2 2026)

**Phase 2+**:

- ✅ Real-time collaborative editing (WebSocket + Operational Transformation)
- ✅ Offline voting (with conflict resolution for overlapping votes)
- ✅ Document uploads to local cache, background sync when online
- ✅ Advanced sync: Merkle trees for efficient sync of large datasets
- ✅ Blockchain-verified voting (experimental; not MVP)

References
==========

- `Tauri 2.0 Documentation <https://tauri.app/>`_
- `Tauri Plugins <https://tauri.app/docs/v1/features/plugins/>`_
- `SQLCipher <https://www.zetetic.net/sqlcipher/>`_
- `Operational Transformation (CRDT) <https://en.wikipedia.org/wiki/Operational_transformation>`_
- Issue #295: Tauri Workspace Setup
- Issue #296: SQLite Adapters
- Issue #297: Offline Sync Engine
- Issue #298: Mobile App (iOS/Android)
- Issue #299: Desktop App (Windows/macOS/Linux)

See also: `TAURI_OFFLINE_SYNC.rst`
