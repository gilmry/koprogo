==============================================================================
Tauri Offline Sync Engine — Data Synchronization & Conflict Resolution
==============================================================================

:Date: March 2026
:Version: 1.0.0 - **ARCHITECTURAL DESIGN** (Not yet implemented)
:Related Issues: #297 (Offline Sync Engine)
:Status: Architecture Documentation
:Author: Claude Code (Anthropic)

.. contents:: Table of Contents
   :depth: 3

Overview
========

The offline sync engine is the core system that keeps local SQLite data synchronized with the KoproGo backend. It handles:

1. **Offline-first local changes**: Store operations locally (create, update, delete) without requiring server connectivity
2. **Conflict detection**: Identify when both local and server modified the same entity
3. **Automatic sync triggers**: Sync on reconnect, app foreground, or on timer (every 5 minutes in background)
4. **Conflict resolution**: Automatic (last-write-wins) for most data, manual resolution in UI for important fields
5. **Retry logic**: Exponential backoff on sync failures with user notification
6. **Data integrity**: Merkle trees for efficient sync of large datasets (Phase 2)

Architecture Overview
======================

**Sync flow**:

.. code-block:: text

    ┌─────────────────────────────────────────────────────────┐
    │           User Action (Create/Update/Delete)           │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      1. Validate via Domain Entity                      │
    │         Building::new(...)                              │
    │         → Enforces business rules                       │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      2. Persist to Local SQLite                         │
    │         INSERT/UPDATE/DELETE in buildings table         │
    │         Set local_version++ (optimistic concurrency)    │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      3. Add to Sync Queue                               │
    │         INSERT INTO sync_queue (operation, payload)     │
    │         operation = 'create' | 'update' | 'delete'      │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      4. Emit IPC Event to UI                            │
    │         'building-created' or 'building-updated'        │
    │         UI updates immediately (optimistic UI)          │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      5. Sync Trigger (if online)                        │
    │         - On reconnect (network.online event)           │
    │         - On app foreground (pause/resume event)        │
    │         - Every 5 minutes (background timer)            │
    │         - On logout (flush pending)                     │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      6. SyncEngine::full_sync() or incremental_sync()   │
    │         a) Compress sync queue (deduplicate ops)        │
    │         b) Upload pending changes                       │
    │         c) Download remote changes                      │
    │         d) Detect conflicts (last_modified_at)          │
    │         e) Resolve conflicts (LWW or manual)            │
    │         f) Merge into local SQLite                      │
    │         g) Mark operations as synced_at = NOW()         │
    └──────────────────┬──────────────────────────────────────┘
                       │
                       ▼
    ┌─────────────────────────────────────────────────────────┐
    │      7. UI Sync Status Update                           │
    │         SyncStatus = 'synced', show timestamp           │
    │         Or show conflicts if detected                   │
    └─────────────────────────────────────────────────────────┘

Sync Queue Design
=================

**Purpose**: Queue local changes for later transmission to backend.

**Table schema**:

.. code-block:: sql

    CREATE TABLE sync_queue (
        id TEXT PRIMARY KEY,                    -- UUID
        entity_type TEXT NOT NULL,              -- 'building', 'unit', 'owner', 'expense', 'ticket', etc.
        entity_id TEXT NOT NULL,                -- UUID of the entity being changed
        operation TEXT NOT NULL,                -- 'create' | 'update' | 'delete'
        payload JSON NOT NULL,                  -- Serialized entity (full state for create/update)
        created_at DATETIME NOT NULL,           -- When operation was queued
        synced_at DATETIME,                     -- NULL = not yet synced, set on successful sync
        retry_count INTEGER DEFAULT 0,          -- Number of failed sync attempts
        last_error TEXT,                        -- Error message from last failed attempt
        FOREIGN KEY (entity_id) REFERENCES buildings(id)
        -- Note: Foreign key is permissive; allows orphaned records during deletes
    );

    CREATE INDEX idx_sync_queue_synced_at ON sync_queue(synced_at);
    CREATE INDEX idx_sync_queue_entity ON sync_queue(entity_type, entity_id);
    CREATE INDEX idx_sync_queue_created ON sync_queue(created_at);

**Operations per entity type**:

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Entity Type
     - Max Queue Size
     - Notes
   * - building
     - 100
     - Unlikely to change > 100 times before sync
   * - unit
     - 500
     - Bulk unit creation possible
   * - owner
     - 1,000
     - Bulk owner import
   * - expense
     - 10,000
     - High churn for monthly charges
   * - ticket
     - 5,000
     - Maintenance requests
   * - unit_owner
     - 1,000
     - Multi-owner transfers

**Key operations**:

.. code-block:: rust

    // src/queue.rs (koprogo-sync crate)

    pub struct SyncQueueRepository { pool: SqlitePool }

    impl SyncQueueRepository {
        /// Add operation to queue
        pub async fn enqueue(
            &self,
            entity_type: &str,
            entity_id: Uuid,
            operation: SyncOperation,
            payload: serde_json::Value,
        ) -> Result<()> {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO sync_queue (id, entity_type, entity_id, operation, payload, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(id.to_string())
            .bind(entity_type)
            .bind(entity_id.to_string())
            .bind(operation.to_string())
            .bind(payload.to_string())
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        /// Get all pending (not synced) operations
        pub async fn get_pending(&self) -> Result<Vec<SyncQueueEntry>> {
            let entries = sqlx::query_as::<_, SyncQueueEntry>(
                "SELECT id, entity_type, entity_id, operation, payload, created_at, retry_count
                 FROM sync_queue
                 WHERE synced_at IS NULL
                 ORDER BY created_at ASC"
            )
            .fetch_all(&self.pool)
            .await?;
            Ok(entries)
        }

        /// Mark operation as synced
        pub async fn mark_synced(&self, queue_id: Uuid) -> Result<()> {
            sqlx::query("UPDATE sync_queue SET synced_at = ? WHERE id = ?")
                .bind(Utc::now())
                .bind(queue_id.to_string())
                .execute(&self.pool)
                .await?;
            Ok(())
        }

        /// Mark operation as failed (increment retry_count, set last_error)
        pub async fn mark_failed(
            &self,
            queue_id: Uuid,
            error: &str,
        ) -> Result<()> {
            sqlx::query(
                "UPDATE sync_queue
                 SET retry_count = retry_count + 1,
                     last_error = ?,
                     synced_at = NULL
                 WHERE id = ?"
            )
            .bind(error)
            .bind(queue_id.to_string())
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        /// Delete operations after successful sync (cleanup)
        pub async fn delete_synced(&self, before: DateTime<Utc>) -> Result<u64> {
            let result = sqlx::query(
                "DELETE FROM sync_queue
                 WHERE synced_at IS NOT NULL AND synced_at < ?"
            )
            .bind(before)
            .execute(&self.pool)
            .await?;
            Ok(result.rows_affected())
        }
    }

Queue Deduplication
-------------------

**Problem**: If user creates a building, then edits it 5 times before syncing, we want to send only the final state (not all 6 operations).

**Solution**: Compress queue before upload.

.. code-block:: rust

    pub async fn compress_sync_queue(
        queue: Vec<SyncQueueEntry>,
    ) -> Vec<SyncQueueEntry> {
        use std::collections::HashMap;

        let mut compressed: HashMap<String, SyncQueueEntry> = HashMap::new();

        for entry in queue {
            let key = format!("{}:{}", entry.entity_type, entry.entity_id);

            match compressed.get_mut(&key) {
                Some(existing) => {
                    match (existing.operation, entry.operation) {
                        // create + update = create (send final state)
                        (SyncOperation::Create, SyncOperation::Update) => {
                            existing.payload = entry.payload;
                            existing.created_at = entry.created_at;
                        }
                        // create + delete = DELETE (don't send at all)
                        (SyncOperation::Create, SyncOperation::Delete) => {
                            compressed.remove(&key);
                        }
                        // update + update = update (send latest)
                        (SyncOperation::Update, SyncOperation::Update) => {
                            existing.payload = entry.payload;
                        }
                        // update + delete = delete
                        (SyncOperation::Update, SyncOperation::Delete) => {
                            existing.operation = SyncOperation::Delete;
                            existing.payload = entry.payload;
                        }
                        _ => {} // Keep existing
                    }
                }
                None => {
                    compressed.insert(key, entry);
                }
            }
        }

        compressed.into_values().collect()
    }

Conflict Detection & Resolution
===============================

**Conflict definition**: Local entity was modified **AND** server entity was modified since last successful sync.

**Root cause**:

1. User creates building on device A (offline)
2. User creates same building on device B / web (online)
3. Devices sync → conflict on `building_id`

Or more commonly:

1. Building is synced from server
2. User modifies locally (offline)
3. Server receives update from web (online)
4. Device comes online → conflict

**Detection strategy**:

Each entity stores two timestamps:

.. code-block:: sql

    CREATE TABLE buildings (
        id TEXT PRIMARY KEY,
        -- ... business fields ...
        created_at DATETIME NOT NULL,       -- Original creation time
        updated_at DATETIME NOT NULL,       -- Server-side last modification
        local_version INTEGER DEFAULT 0,    -- Local change counter (optimistic lock)
        synced_at DATETIME,                 -- When last synced from server
        server_hash TEXT,                   -- Hash of server version at last sync
    );

**Conflict detection algorithm**:

.. code-block:: rust

    pub struct Conflict {
        pub entity_id: Uuid,
        pub entity_type: String,
        pub local_version: i32,
        pub local_data: serde_json::Value,
        pub local_modified_at: DateTime<Utc>,
        pub server_version: i32,
        pub server_data: serde_json::Value,
        pub server_modified_at: DateTime<Utc>,
    }

    pub async fn detect_conflicts(
        sync_queue: &[SyncQueueEntry],
        api_client: &ApiClient,
        sqlite_pool: &SqlitePool,
    ) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        for entry in sync_queue {
            // Fetch server version
            let server_entity = api_client
                .get_entity(&entry.entity_type, &entry.entity_id)
                .await?;

            // Fetch local version
            let local_entity: serde_json::Value = sqlx::query_scalar(
                &format!("SELECT json_extract({}, '$') FROM {}", entry.payload, entry.entity_type)
            )
            .fetch_optional(sqlite_pool)
            .await?
            .flatten();

            if let Some(local) = local_entity {
                // Compare hashes to detect if server was modified
                let server_hash = calculate_hash(&server_entity)?;
                let stored_hash: Option<String> = sqlx::query_scalar(
                    &format!("SELECT server_hash FROM {} WHERE id = ?", entry.entity_type)
                )
                .bind(&entry.entity_id.to_string())
                .fetch_optional(sqlite_pool)
                .await?
                .flatten();

                // Conflict: server was modified since our last sync
                if let Some(stored) = stored_hash {
                    if stored != server_hash {
                        conflicts.push(Conflict {
                            entity_id: entry.entity_id,
                            entity_type: entry.entity_type.clone(),
                            local_version: 0, // TODO: fetch from DB
                            local_data: local,
                            local_modified_at: entry.created_at,
                            server_version: server_entity.version,
                            server_data: server_entity.data,
                            server_modified_at: server_entity.updated_at,
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }

**Last-Write-Wins (LWW) Conflict Resolution**:

Default strategy for most fields. Field with later `updated_at` timestamp wins.

.. code-block:: rust

    pub async fn resolve_conflict_lww(
        conflict: &Conflict,
        resolution: &ConflictResolution,
    ) -> Result<serde_json::Value> {
        match resolution {
            ConflictResolution::PreferLocal => {
                // Use local version; ignore server changes
                Ok(conflict.local_data.clone())
            }
            ConflictResolution::PreferServer => {
                // Use server version; discard local changes
                Ok(conflict.server_data.clone())
            }
            ConflictResolution::LwwMerge => {
                // Merge field-by-field, preferring later timestamp
                let mut merged = json!({});
                let local_obj = conflict.local_data.as_object()?;
                let server_obj = conflict.server_data.as_object()?;

                let all_keys: HashSet<_> = local_obj
                    .keys()
                    .chain(server_obj.keys())
                    .cloned()
                    .collect();

                for key in all_keys {
                    let local_val = local_obj.get(&key);
                    let server_val = server_obj.get(&key);

                    let value = match (local_val, server_val) {
                        (Some(l), Some(s)) => {
                            // Both have value; prefer based on timestamp
                            if conflict.local_modified_at > conflict.server_modified_at {
                                l.clone()
                            } else {
                                s.clone()
                            }
                        }
                        (Some(l), None) => l.clone(),
                        (None, Some(s)) => s.clone(),
                        (None, None) => continue,
                    };

                    merged[&key] = value;
                }

                Ok(merged)
            }
            ConflictResolution::Manual(chosen_fields) => {
                // User chose specific fields from local or server
                let mut merged = conflict.server_data.clone();
                for field_choice in chosen_fields {
                    if field_choice.prefer_local {
                        merged[&field_choice.field_name] =
                            conflict.local_data[&field_choice.field_name].clone();
                    }
                }
                Ok(merged)
            }
        }
    }

**Special rules for critical fields**:

Some fields should NOT use LWW. Instead, user intervention is required.

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - Entity
     - Field
     - Conflict Rule
   * - Expense
     - amount_cents
     - Manual: Show both amounts, user chooses
   * - Payment
     - status
     - Manual: Show both status, user chooses (prevents double payment)
   * - Vote
     - choice (Pour/Contre/Abstention)
     - Manual: Cannot auto-resolve voting conflicts
   * - Building
     - name, address
     - LWW: Descriptive fields, LWW acceptable
   * - UnitOwner
     - quote_percentage
     - Manual: Ownership disputes require review

Sync Triggers
=============

**When does sync happen?**

.. list-table::
   :header-rows: 1
   :widths: 30 20 50

   * - Trigger
     - Delay
     - Priority
   * - **Network reconnect**
     - Immediate
     - High (user expects immediate consistency)
   * - **App foreground**
     - 1 second (debounced)
     - High (user brought app to focus)
   * - **Background timer**
     - Every 5 minutes
     - Low (user not actively using app)
   * - **User logout**
     - Synchronous (before logout completes)
     - Critical (flush all pending data)
   * - **Manual "Sync Now"**
     - Immediate
     - User-initiated
   * - **Batched operations**
     - 2 seconds after last operation
     - Medium (debounced to batch local changes)

**Implementation**:

.. code-block:: rust

    // src-tauri/src/sync.rs
    pub struct SyncManager {
        sync_queue: Arc<SyncQueueRepository>,
        api_client: Arc<ApiClient>,
        sqlite_pool: Arc<SqlitePool>,
        sync_state: Arc<RwLock<SyncState>>,
    }

    pub enum SyncState {
        Idle,
        Syncing,
        Paused,
    }

    impl SyncManager {
        pub async fn start_background_sync(&self) {
            let interval = tokio::time::interval(Duration::from_secs(300)); // 5 min

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if self.is_online().await {
                            let _ = self.incremental_sync().await;
                        }
                    }
                    // Listen for network online event
                    _ = self.network_online_event.listen() => {
                        let _ = self.full_sync().await;
                    }
                    // Listen for app foreground event
                    _ = self.app_foreground_event.listen() => {
                        let _ = self.incremental_sync().await;
                    }
                }
            }
        }

        /// Full sync: upload all pending, download all remote changes
        pub async fn full_sync(&self) -> Result<SyncStats> {
            *self.sync_state.write().await = SyncState::Syncing;

            let stats = self._sync_impl(SyncMode::Full).await;

            *self.sync_state.write().await = SyncState::Idle;
            Ok(stats?)
        }

        /// Incremental: only new changes since last sync
        pub async fn incremental_sync(&self) -> Result<SyncStats> {
            if matches!(*self.sync_state.read().await, SyncState::Syncing) {
                return Ok(SyncStats::default()); // Already syncing
            }

            *self.sync_state.write().await = SyncState::Syncing;

            let stats = self._sync_impl(SyncMode::Incremental).await;

            *self.sync_state.write().await = SyncState::Idle;
            Ok(stats?)
        }

        async fn _sync_impl(&self, mode: SyncMode) -> Result<SyncStats> {
            let mut stats = SyncStats::default();

            // 1. Get pending operations
            let mut queue = self.sync_queue.get_pending().await?;
            stats.local_changes_queued = queue.len();

            // 2. Compress queue
            queue = compress_sync_queue(queue);
            stats.local_changes_compressed = queue.len();

            // 3. Upload to server
            let upload_response = self
                .api_client
                .sync(queue.iter().map(|e| e.to_sync_op()).collect())
                .await?;

            stats.local_uploaded = upload_response.processed_count;
            stats.upload_errors = upload_response.failed_count;

            // Mark uploaded operations as synced
            for sync_op in upload_response.processed_ids {
                self.sync_queue.mark_synced(sync_op).await?;
            }

            // 4. Download remote changes
            let last_sync = self.get_last_sync_timestamp().await?;
            let remote_changes = self.api_client
                .get_changes_since(last_sync)
                .await?;

            stats.remote_changes = remote_changes.len();

            // 5. Detect conflicts
            let conflicts = detect_conflicts(&queue, &self.api_client, &self.sqlite_pool).await?;
            stats.conflicts_detected = conflicts.len();

            // 6. Resolve conflicts (LWW or manual)
            for conflict in conflicts {
                let resolution = self.resolve_conflict(&conflict).await?;
                // Store conflict for UI to display
                self.conflict_cache.insert(conflict.entity_id, (conflict, resolution.clone()));
            }

            // 7. Merge into local database
            for remote_change in remote_changes {
                self.apply_remote_change(&remote_change).await?;
            }

            // 8. Update sync metadata
            self.update_last_sync_timestamp(Utc::now()).await?;
            stats.synced_at = Utc::now();

            Ok(stats)
        }
    }

Merkle Tree Sync (Phase 2)
==========================

**Problem**: For large datasets (1 million entities), downloading the entire change log is inefficient.

**Solution**: Use Merkle trees for efficient sync.

**Concept**:

.. code-block:: text

    Server maintains a Merkle tree of all entities in a building:

                    root: hash(left + right)
                            /         \
                         h1              h2
                        /  \            /  \
                       h3   h4         h5   h6
                      / \   / \       / \
                    e1 e2 e3 e4     e5 e6 e7 e8

    Client computes same tree from local data, compares root hashes.

    If mismatch: recursively find differing subtrees.
    Download only changed leaf nodes (entities).

**API endpoint** (Phase 2):

.. code-block:: rust

    GET /api/v1/sync/merkle-tree?building_id=xxx&depth=8
    Response: {
        root_hash: "abc123...",
        depth: 8,
        nodes: [
            { path: "1", hash: "..." },    // Left subtree
            { path: "2", hash: "..." },    // Right subtree
            ...
        ]
    }

**Sync algorithm**:

.. code-block:: rust

    pub async fn merkle_sync(
        client: &ApiClient,
        local_pool: &SqlitePool,
        building_id: Uuid,
    ) -> Result<SyncStats> {
        // 1. Build local Merkle tree
        let local_tree = build_merkle_tree(local_pool, building_id, depth: 8).await?;
        let local_root = local_tree.root_hash();

        // 2. Fetch server Merkle tree
        let server_tree = client.get_merkle_tree(building_id, 8).await?;
        let server_root = server_tree.root_hash();

        // 3. If roots match, no sync needed
        if local_root == server_root {
            return Ok(SyncStats { ... });
        }

        // 4. Find differing nodes recursively
        let differing_nodes = find_differing_nodes(
            &local_tree,
            &server_tree,
            depth: 8,
        );

        // 5. Download only differing leaf nodes (entities)
        let mut remote_changes = Vec::new();
        for node in differing_nodes {
            if node.is_leaf() {
                let entity = client.get_entity(&node.entity_id).await?;
                remote_changes.push(entity);
            }
        }

        // 6. Merge into local database (same as above)
        for change in remote_changes {
            apply_remote_change(local_pool, change).await?;
        }

        Ok(SyncStats { ... })
    }

**Benefits**:

- Full sync of 1,000,000 entities: Download only ~20 KB of Merkle hashes (instead of 1 GB of entity data)
- Incremental sync: Skip unchanged subtrees entirely
- Early termination: Stop recursion when subtree hashes match

Sync Queue Example
==================

**Scenario**: User creates a building, then modifies it 3 times, all offline.

**Step 1: Create building**

.. code-block:: sql

    -- INSERT into buildings
    INSERT INTO buildings (id, name, address, ...)
    VALUES ('uuid-1', 'Rue de la Paix', '1000 Bruxelles', ...);

    -- INSERT into sync_queue
    INSERT INTO sync_queue (id, entity_type, entity_id, operation, payload, created_at)
    VALUES (
        'queue-1',
        'building',
        'uuid-1',
        'create',
        '{"id":"uuid-1","name":"Rue de la Paix",...}',
        '2026-03-23 10:00:00'
    );

**Step 2: Update building name**

.. code-block:: sql

    -- UPDATE buildings
    UPDATE buildings SET name = 'Rue de la Paix Rénovée' WHERE id = 'uuid-1';
    UPDATE buildings SET local_version = local_version + 1 WHERE id = 'uuid-1';

    -- INSERT into sync_queue
    INSERT INTO sync_queue (id, entity_type, entity_id, operation, payload, created_at)
    VALUES (
        'queue-2',
        'building',
        'uuid-1',
        'update',
        '{"id":"uuid-1","name":"Rue de la Paix Rénovée",...}',
        '2026-03-23 10:05:00'
    );

**Step 3: Update building address**

.. code-block:: sql

    UPDATE buildings SET address = '1000 Bruxelles, Belgium' WHERE id = 'uuid-1';
    INSERT INTO sync_queue ... operation = 'update', payload = '...';

**sync_queue state before compression**:

.. code-block:: text

    id        | entity_type | entity_id | operation | payload              | created_at
    ----------|-------------|-----------|-----------|----------------------|------------------
    queue-1   | building    | uuid-1    | create    | {"name":"Rue de..."}| 2026-03-23 10:00:00
    queue-2   | building    | uuid-1    | update    | {"name":"Rue de..."}| 2026-03-23 10:05:00
    queue-3   | building    | uuid-1    | update    | {"address":"1000..."}| 2026-03-23 10:10:00

**After compression** (deduplication):

.. code-block:: text

    entity_type | entity_id | operation | payload                          | created_at
    -----------|-----------|-----------|----------------------------------|------------------
    building    | uuid-1    | create    | {"name":"Rue de...","address":"1000..."} | 2026-03-23 10:00:00

**Upload to server**:

Single request: `POST /api/v1/sync` with compressed queue:

.. code-block:: json

    {
        "operations": [
            {
                "entity_type": "building",
                "entity_id": "uuid-1",
                "operation": "create",
                "payload": {
                    "id": "uuid-1",
                    "name": "Rue de la Paix Rénovée",
                    "address": "1000 Bruxelles, Belgium"
                }
            }
        ]
    }

**Server response**:

.. code-block:: json

    {
        "status": "success",
        "processed_count": 1,
        "processed_ids": ["queue-1", "queue-2", "queue-3"],
        "conflicts": [],
        "server_timestamp": "2026-03-23T12:00:00Z"
    }

**Local update**:

Mark all queue entries as synced:

.. code-block:: sql

    UPDATE sync_queue
    SET synced_at = '2026-03-23T12:00:00Z'
    WHERE id IN ('queue-1', 'queue-2', 'queue-3');

Offline Data Categories
=======================

**Data that syncs offline** (readable & writable without connectivity):

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - Category
     - Examples
     - Rationale
   * - Metadata
     - Buildings, units, owners
     - Core reference data; rarely changes
   * - Local transactions
     - Expenses, tickets, notes
     - User can create locally, sync later
   * - Configuration
     - User preferences, app settings
     - No server dependency
   * - Read-only copies
     - Cached documents, past meetings
     - Downloaded on demand, not modified locally

**Data that requires online** (write requires server):

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - Category
     - Examples
     - Rationale
   * - Payments
     - Payment intents, Stripe tokens
     - Financial data; PCI-DSS compliance
   * - Sensitive forms
     - Votes, GDPR requests
     - Legal audit trail required
   * - Large uploads
     - Documents, contractor photos
     - File uploads require server
   * - Real-time data
     - Board notifications, live quorum
     - Requires WebSocket / polling

**UI indicators for offline data**:

.. code-block:: svelte

    <!-- BuildingDetail.svelte -->
    <script>
        import { syncStatus, isOnline } from '../stores/syncStore.ts';
    </script>

    <div class="building-header">
        <h1>{building.name}</h1>

        {#if !$isOnline}
            <span class="badge badge-warning">OFFLINE MODE</span>
        {/if}

        {#if $syncStatus.syncing}
            <span class="spinner">Syncing...</span>
        {:else if $syncStatus.syncedAt}
            <span class="text-sm text-gray-500">
                Synced: {formatTime($syncStatus.syncedAt)}
            </span>
        {/if}

        {#if $syncStatus.conflictsDetected > 0}
            <span class="badge badge-error">
                ⚠️ {$syncStatus.conflictsDetected} Conflicts
            </span>
        {/if}
    </div>

UI for Conflict Resolution
==========================

**When conflicts are detected, user sees modal**:

.. code-block:: svelte

    <!-- ConflictResolver.svelte -->
    <script>
        import { conflicts, resolveConflict } from '../stores/syncStore.ts';
    </script>

    {#if $conflicts.length > 0}
        <div class="modal">
            <h2>Resolve Conflicts</h2>
            <p>The following fields were modified both locally and on the server.</p>

            {#each $conflicts as conflict (conflict.entity_id)}
                <div class="conflict-card">
                    <h3>{conflict.entity_type}: {conflict.entity_id}</h3>

                    {#each Object.keys(conflict.local_data) as field}
                        {@const local_val = conflict.local_data[field]}
                        {@const server_val = conflict.server_data[field]}

                        {#if local_val !== server_val}
                            <div class="field-conflict">
                                <label>{field}</label>
                                <div class="options">
                                    <label>
                                        <input
                                            type="radio"
                                            name="field_{field}"
                                            value="local"
                                            on:change={() => resolveField(conflict, field, 'local')}
                                        />
                                        Local: {local_val}
                                    </label>
                                    <label>
                                        <input
                                            type="radio"
                                            name="field_{field}"
                                            value="server"
                                            on:change={() => resolveField(conflict, field, 'server')}
                                        />
                                        Server: {server_val}
                                    </label>
                                </div>
                            </div>
                        {/if}
                    {/each}

                    <button on:click={() => submitResolution(conflict)}>
                        Resolve
                    </button>
                </div>
            {/each}
        </div>
    {/if}

**For LWW-resolvable conflicts, auto-resolve without UI**:

.. code-block:: rust

    // Auto-resolve building.name (descriptive field)
    let resolution = resolve_conflict_lww(&conflict, ConflictResolution::LwwMerge)?;
    apply_resolution(sqlite_pool, &conflict, &resolution).await?;

GDPR Considerations
===================

**Local data encryption**:

- SQLCipher encrypts entire database at rest (AES-256)
- Password stored in OS keychain (encrypted by OS)
- User cannot read local data files directly

**Data minimization**:

Only sync necessary data for offline use:

.. code-block:: rust

    pub async fn sync_data_for_building(
        building_id: Uuid,
        api_client: &ApiClient,
    ) -> Result<()> {
        // Download only data relevant to this building
        let buildings = api_client.get_buildings().await?; // Buildings for this org
        let units = api_client.get_units(building_id).await?;
        let owners = api_client.get_owners().await?;
        // Don't download: sensitive documents, payment methods, etc.

        // Store in SQLite
        // ...

        Ok(())
    }

**Right to erasure** (Article 17 GDPR):

User requests deletion → All local data wiped:

.. code-block:: rust

    #[tauri::command]
    pub async fn request_erasure(state: tauri::State<'_, AppState>) -> Result<()> {
        // 1. Call backend: DELETE /gdpr/erase
        state.api_client.request_erasure().await?;

        // 2. Wipe local SQLite database
        let _ = tokio::fs::remove_file(&state.db_path).await;
        let password = generate_random_password(32);
        let new_pool = create_encrypted_pool(&state.db_path, &password).await?;

        // 3. Update app state
        state.sqlite_pool.replace(new_pool);

        Ok(())
    }

**Logout behavior**:

.. code-block:: rust

    #[tauri::command]
    pub async fn logout(state: tauri::State<'_, AppState>) -> Result<()> {
        // 1. Flush pending sync queue (upload any unsaved data)
        state.sync_engine.full_sync().await?;

        // 2. Clear JWT from keychain
        state.keychain.delete("jwt_token").await?;

        // 3. Optional: Clear local database
        // - Some users prefer to keep cache (faster re-login)
        // - Add setting: "Clear offline data on logout"

        Ok(())
    }

Performance Optimization
========================

**Query optimization for local SQLite**:

.. code-block:: sql

    -- Add indexes for common queries
    CREATE INDEX idx_buildings_org ON buildings(organization_id);
    CREATE INDEX idx_units_building ON units(building_id);
    CREATE INDEX idx_owners_org ON owners(organization_id);
    CREATE INDEX idx_expenses_building ON expenses(building_id);

    -- Partial indexes for common filters
    CREATE INDEX idx_expenses_unpaid ON expenses(building_id)
        WHERE status = 'Approved' AND paid_at IS NULL;

    CREATE INDEX idx_tickets_open ON tickets(building_id)
        WHERE status IN ('Open', 'Assigned', 'InProgress');

**Batch operations**:

.. code-block:: rust

    pub async fn sync_many_buildings(
        building_ids: Vec<Uuid>,
        api_client: &ApiClient,
        sqlite_pool: &SqlitePool,
    ) -> Result<()> {
        // Single transaction, not 1000 individual inserts
        let mut tx = sqlite_pool.begin().await?;

        for building_id in building_ids {
            let building = api_client.get_building(building_id).await?;
            sqlx::query("INSERT OR REPLACE INTO buildings (...) VALUES (...)")
                .bind(building.id)
                .bind(building.name)
                // ...
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

Testing Offline Scenarios
==========================

**Test 1: Offline-first workflow**

.. code-block:: text

    1. Launch app (online)
    2. Download building data
    3. Go offline (disable network)
    4. Create building locally → should succeed
    5. Modify building → should succeed
    6. Refresh app → data still there
    7. Go online
    8. Trigger sync → should upload pending changes
    9. Server should see building created

**Test 2: Conflict detection**

.. code-block:: text

    1. Device A: Sync building data
    2. Device B: Sync building data
    3. Device A: Modify building.name offline → queue operation
    4. Device B: Modify building.name online → sync to server
    5. Device A: Go online → detect conflict on building.name
    6. Conflict resolution: Show both versions, user chooses
    7. Resolve with "PreferLocal"
    8. Server updates with Device A's version

**Test 3: Retry logic**

.. code-block:: text

    1. Create expense offline
    2. Go online, but server is down
    3. Sync fails, retry_count = 1, last_error = "connection timeout"
    4. After 5 minutes, retry → connection timeout again
    5. Exponential backoff: wait 10 minutes, retry
    6. Server comes back online
    7. Retry succeeds, synced_at = NOW()

**Test 4: GDPR erasure**

.. code-block:: text

    1. User owns building with 100 units, 500 owners
    2. User requests data export → Download ZIP of local data
    3. User requests erasure → Delete local SQLite + call backend
    4. Backend anonymizes user account, deletes personal data
    5. Local database securely wiped (overwritten)

References
==========

- `Tauri Offline Plugin <https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/sql>`_
- `SQLCipher Documentation <https://www.zetetic.net/sqlcipher/>`_
- `Operational Transformation (Conflict Resolution) <https://en.wikipedia.org/wiki/Operational_transformation>`_
- `Merkle Trees for Data Sync <https://en.wikipedia.org/wiki/Merkle_tree>`_
- `GDPR Article 17 (Right to Erasure) <https://gdpr-info.eu/art-17-gdpr/>`_
- Issue #297: Offline Sync Engine
- `TAURI_ARCHITECTURE.rst`
