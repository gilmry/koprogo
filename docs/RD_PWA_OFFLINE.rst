==========================================================
R&D: PWA offline sync — Architecture de synchronisation
==========================================================

Issue: #234
Status: Design Phase (SUPERSEDED)
Phase: Jalon 4 (Automation & Intégrations)
Date: 2026-03-23

.. contents::
   :depth: 3

Executive Summary
=================

**Status**: This document describes the original PWA offline-first approach from Issue #234. However, **this architecture has been superseded by Tauri desktop application** (see `docs/TAURI_OFFLINE_SYNC.rst` for current approach).

**Key Finding**: Tauri provides native SQLite + Rust sync engine with better security and UX than browser-based PWA Service Workers + IndexedDB.

**Exception**: Contractor reports PWA (Issue #275 - BC16 "Rapports de Travaux") remains as pure PWA using this architecture since contractors often work on-site with poor connectivity.

Original PWA Architecture (Reference)
=====================================

Problem Statement
-----------------

Original challenge (Q4 2025):

* Syndics and accountants need to work offline during power outages
* Mobile users have intermittent connectivity
* Contractor reports from jobsites (no WiFi available)
* Current API-only design fails without internet

**Solution at the time**: PWA (Progressive Web App) with Service Workers + IndexedDB.

**Current status**: Most offline needs now met by Tauri desktop app. However, contractor field work remains PWA-only.

Why Tauri Won
==============

**Tauri** (Desktop Application) selected for main offline sync over PWA because:

+-------------------------+--------+---------+------+
| Criterion               | Tauri  | PWA    | Winner|
+=========================+========+=========+======+
| Offline SQL persistence | ✓ SQLite| ✗ IndexedDB | Tauri |
+-------------------------+--------+---------+------+
| Sync engine reliability | ✓ Rust | ✗ JS  | Tauri |
+-------------------------+--------+---------+------+
| Conflict resolution     | ✓ Auto | ⚠ Manual | Tauri |
+-------------------------+--------+---------+------+
| Security (key storage)  | ✓ OS   | ✗ Cookies | Tauri |
+-------------------------+--------+---------+------+
| Bundle size             | 50MB   | 2MB (PWA) | PWA |
+-------------------------+--------+---------+------+
| Works offline (no URL)  | ✓ Yes  | ✗ Requires URL | Tauri |
+-------------------------+--------+---------+------+
| Self-updates           | ✓ Auto | ⚠ Browser | Tauri |
+-------------------------+--------+---------+------+

**Decision**: Tauri for syndics/accountants (main app), PWA for contractors (field work).

Contractor Reports PWA (Issue #275 - BC16)
===========================================

**Why PWA for contractors**:

1. No app install friction (open link, work offline)
2. Form auto-saves to browser local storage
3. Sync when internet available
4. Magic JWT link (72-hour validity per Issue #222)

**Architecture**:

.. code-block:: text

    Contractor receives magic link email:
    "https://app.koprogo.be/contractor/reports/magic/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

         ↓

    Opens in browser (mobile/tablet on jobsite)
         ↓
    Service Worker intercepts (cache-first for offline)
         ↓
    PWA manifest enables offline HTML
         ↓
    Contractor fills form (work description, photos, parts)
         ↓
    IndexedDB stores draft locally (even if offline)
         ↓
    Submit button:
         - If online: POST to /api/v1/contractor-reports/magic/{token}
         - If offline: Show "Will sync when online"

         ↓

    When internet available:
         Background sync queues submission
         Server validates token, creates report

Service Workers Strategy
========================

**Cache Strategies**:

1. **Cache-First** (for assets):

   .. code-block:: javascript

    // Cache static assets: HTML, CSS, JS (vendor stuff)
    self.addEventListener('fetch', (event) => {
        if (isStaticAsset(event.request.url)) {
            event.respondWith(
                caches.match(event.request)
                    .then(response => response || fetch(event.request))
            );
        }
    });

   Use for:
   * JS bundles (Svelte compiled)
   * CSS stylesheets
   * Web fonts
   * PNG/SVG icons

2. **Network-First** (for API):

   .. code-block:: javascript

    // Try network first, fall back to cache
    self.addEventListener('fetch', (event) => {
        if (isApiRequest(event.request.url)) {
            event.respondWith(
                fetch(event.request)
                    .then(response => {
                        // Cache successful response
                        const cache = caches.open('api-v1');
                        cache.then(c => c.put(event.request, response.clone()));
                        return response;
                    })
                    .catch(() => {
                        // Fall back to cached response if offline
                        return caches.match(event.request)
                            .then(r => r || createOfflineResponse());
                    })
            );
        }
    });

   Use for:
   * API GET requests (read-only)
   * Data endpoints

3. **Stale-While-Revalidate** (for user data):

   .. code-block:: javascript

    // Return cached version, update in background
    self.addEventListener('fetch', (event) => {
        if (isUserDataRequest(event.request.url)) {
            event.respondWith(
                caches.match(event.request)
                    .then(cached => {
                        const fresh = fetch(event.request)
                            .then(response => {
                                caches.open('user-data-v1').then(cache =>
                                    cache.put(event.request, response.clone())
                                );
                                return response;
                            });
                        return cached || fresh;
                    })
            );
        }
    });

   Use for:
   * Owner lists
   * Building info (rarely changes)

Service Worker Registration
----------------------------

.. code-block:: javascript

    // frontend/src/lib/sw-registration.js
    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register('/sw.js')
                .then(registration => {
                    console.log('SW registered', registration);

                    // Listen for updates
                    registration.addEventListener('updatefound', () => {
                        const newWorker = registration.installing;
                        newWorker.addEventListener('statechange', () => {
                            if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                                // New version available
                                notifyUserOfUpdate();
                            }
                        });
                    });

                    // Check for updates periodically
                    setInterval(() => registration.update(), 60000);
                })
                .catch(err => console.error('SW registration failed', err));
        });
    }

IndexedDB Schema (Contractor PWA)
=================================

**Offline form storage**:

.. code-block:: javascript

    // frontend/src/lib/indexeddb.js
    const DB_NAME = 'koprogo-contractor';
    const DB_VERSION = 1;
    const STORES = ['reports-draft', 'photos', 'sync-queue'];

    export class ContractorDB {
        constructor() {
            this.db = null;
        }

        async init() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open(DB_NAME, DB_VERSION);

                request.onerror = () => reject(request.error);
                request.onsuccess = () => {
                    this.db = request.result;
                    resolve();
                };

                request.onupgradeneeded = (event) => {
                    const db = event.target.result;

                    // Store 1: Draft report forms
                    if (!db.objectStoreNames.contains('reports-draft')) {
                        const reportStore = db.createObjectStore('reports-draft', {
                            keyPath: 'id',
                            autoIncrement: false,
                        });
                        reportStore.createIndex('ticket_id', 'ticket_id', { unique: false });
                        reportStore.createIndex('status', 'status', { unique: false });
                        reportStore.createIndex('created_at', 'created_at', { unique: false });
                    }

                    // Store 2: Captured photos (base64 or blob)
                    if (!db.objectStoreNames.contains('photos')) {
                        const photoStore = db.createObjectStore('photos', {
                            keyPath: 'id',
                            autoIncrement: false,
                        });
                        photoStore.createIndex('report_id', 'report_id', { unique: false });
                        photoStore.createIndex('photo_type', 'photo_type', { unique: false });
                    }

                    // Store 3: Sync queue (for background sync)
                    if (!db.objectStoreNames.contains('sync-queue')) {
                        const syncStore = db.createObjectStore('sync-queue', {
                            keyPath: 'id',
                            autoIncrement: false,
                        });
                        syncStore.createIndex('status', 'status', { unique: false });
                        syncStore.createIndex('retry_count', 'retry_count', { unique: false });
                    }
                };
            });
        }

        // Save draft report
        async saveDraftReport(report) {
            const tx = this.db.transaction(['reports-draft'], 'readwrite');
            const store = tx.objectStore('reports-draft');
            return new Promise((resolve, reject) => {
                const req = store.put({
                    ...report,
                    saved_at: new Date().toISOString(),
                    status: 'DRAFT',
                });
                req.onsuccess = () => resolve(req.result);
                req.onerror = () => reject(req.error);
            });
        }

        // Get draft report
        async getDraftReport(reportId) {
            const tx = this.db.transaction(['reports-draft'], 'readonly');
            const store = tx.objectStore('reports-draft');
            return new Promise((resolve, reject) => {
                const req = store.get(reportId);
                req.onsuccess = () => resolve(req.result);
                req.onerror = () => reject(req.error);
            });
        }

        // Save photo
        async savePhoto(reportId, photoType, photoBlob) {
            const base64 = await this.blobToBase64(photoBlob);
            const tx = this.db.transaction(['photos'], 'readwrite');
            const store = tx.objectStore('photos');
            return new Promise((resolve, reject) => {
                const req = store.add({
                    id: crypto.randomUUID(),
                    report_id: reportId,
                    photo_type: photoType,  // 'before' or 'after'
                    data: base64,
                    size_bytes: photoBlob.size,
                    created_at: new Date().toISOString(),
                });
                req.onsuccess = () => resolve(req.result);
                req.onerror = () => reject(req.error);
            });
        }

        // List photos for report
        async getPhotosForReport(reportId) {
            const tx = this.db.transaction(['photos'], 'readonly');
            const store = tx.objectStore('photos');
            const index = store.index('report_id');
            return new Promise((resolve, reject) => {
                const req = index.getAll(reportId);
                req.onsuccess = () => resolve(req.result);
                req.onerror = () => reject(req.error);
            });
        }

        // Add to sync queue
        async queueSync(reportId, action) {
            const tx = this.db.transaction(['sync-queue'], 'readwrite');
            const store = tx.objectStore('sync-queue');
            return new Promise((resolve, reject) => {
                const req = store.add({
                    id: crypto.randomUUID(),
                    report_id: reportId,
                    action: action,  // 'SUBMIT' or 'UPDATE'
                    status: 'PENDING',
                    retry_count: 0,
                    created_at: new Date().toISOString(),
                });
                req.onsuccess = () => resolve(req.result);
                req.onerror = () => reject(req.error);
            });
        }

        async blobToBase64(blob) {
            return new Promise((resolve, reject) => {
                const reader = new FileReader();
                reader.onload = () => resolve(reader.result);
                reader.onerror = reject;
                reader.readAsDataURL(blob);
            });
        }
    }

Contractor Report Form (Svelte Component)
==========================================

.. code-block:: svelte

    <!-- frontend/src/components/ContractorReportForm.svelte -->
    <script>
        import { ContractorDB } from '$lib/indexeddb';
        import Camera from '$lib/Camera.svelte';

        let db;
        let report = {
            id: crypto.randomUUID(),
            ticket_id: '',
            work_date: new Date().toISOString().split('T')[0],
            description: '',
            parts_replaced: [],
            status: 'DRAFT',
        };
        let photos = [];
        let isOnline = navigator.onLine;
        let isSyncing = false;

        onMount(async () => {
            db = new ContractorDB();
            await db.init();

            window.addEventListener('online', () => { isOnline = true; });
            window.addEventListener('offline', () => { isOnline = false; });
        });

        async function capturePhoto(photoType) {
            const blob = await Camera.capture();  // Custom camera UI
            photos.push({
                type: photoType,  // 'before' or 'after'
                blob,
            });
            await db.savePhoto(report.id, photoType, blob);
            photos = photos;  // Reactivity
        }

        async function saveAsDraft() {
            await db.saveDraftReport(report);
            alert('Draft saved locally');
        }

        async function submitReport() {
            if (!isOnline) {
                // Queue for later sync
                await db.queueSync(report.id, 'SUBMIT');
                alert('Report queued. Will submit when online.');
                return;
            }

            isSyncing = true;
            try {
                // Upload photos first
                const photoUrls = [];
                for (const photo of photos) {
                    const fd = new FormData();
                    fd.append('file', photo.blob);
                    const res = await fetch('/api/v1/uploads', {
                        method: 'POST',
                        body: fd,
                        headers: {
                            'Authorization': `Bearer ${window.contractorToken}`,
                        },
                    });
                    const { url } = await res.json();
                    photoUrls.push(url);
                }

                // Submit report
                const res = await fetch(
                    `/api/v1/contractor-reports/magic/${window.contractorToken}`,
                    {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'Authorization': `Bearer ${window.contractorToken}`,
                        },
                        body: JSON.stringify({
                            ...report,
                            photos: photoUrls,
                        }),
                    }
                );

                if (res.ok) {
                    alert('Report submitted successfully!');
                    // Clear draft from IndexedDB
                    await db.clearDraft(report.id);
                } else {
                    throw new Error(await res.text());
                }
            } catch (err) {
                console.error('Submit failed', err);
                alert('Submit failed. Will retry when online.');
                await db.queueSync(report.id, 'SUBMIT');
            } finally {
                isSyncing = false;
            }
        }
    </script>

    <form on:submit|preventDefault={submitReport}>
        <h2>Contractor Report - {report.ticket_id}</h2>

        <div class="status-bar">
            {#if isOnline}
                <span class="online">● Online</span>
            {:else}
                <span class="offline">● Offline (will sync later)</span>
            {/if}
        </div>

        <label>
            Work Date
            <input type="date" bind:value={report.work_date} required />
        </label>

        <label>
            Description
            <textarea bind:value={report.description} placeholder="What was done?"></textarea>
        </label>

        <label>
            Parts Replaced
            <input type="text" bind:value={report.parts_replaced} placeholder="e.g., heating valve, filters" />
        </label>

        <div class="photo-section">
            <h3>Before Photos</h3>
            <button type="button" on:click={() => capturePhoto('before')}>
                📷 Take Photo (Before)
            </button>
            <div class="photo-list">
                {#each photos.filter(p => p.type === 'before') as photo (photo)}
                    <img src={photo.blob} alt="before" />
                {/each}
            </div>

            <h3>After Photos</h3>
            <button type="button" on:click={() => capturePhoto('after')}>
                📷 Take Photo (After)
            </button>
            <div class="photo-list">
                {#each photos.filter(p => p.type === 'after') as photo (photo)}
                    <img src={photo.blob} alt="after" />
                {/each}
            </div>
        </div>

        <div class="actions">
            <button type="button" on:click={saveAsDraft} class="btn-secondary">
                💾 Save as Draft
            </button>
            <button type="submit" disabled={isSyncing} class="btn-primary">
                {#if isSyncing}
                    ⏳ Submitting...
                {:else}
                    ✓ Submit Report
                {/if}
            </button>
        </div>
    </form>

    <style>
        .status-bar {
            background: #f0f0f0;
            padding: 8px;
            border-radius: 4px;
            margin-bottom: 1rem;
        }
        .online { color: green; font-weight: bold; }
        .offline { color: red; font-weight: bold; }
        .photo-list { display: flex; gap: 0.5rem; flex-wrap: wrap; }
        .photo-list img { max-width: 150px; border-radius: 4px; }
        .actions { display: flex; gap: 1rem; margin-top: 1rem; }
    </style>

Background Sync API
===================

Modern browsers support **Background Sync** to sync when online:

.. code-block:: javascript

    // When offline submit attempted, queue for sync
    async function queueReportForSync(reportId) {
        if ('serviceWorker' in navigator && 'SyncManager' in window) {
            try {
                const registration = await navigator.serviceWorker.ready;
                await registration.sync.register(`sync-report-${reportId}`);
                console.log('Report queued for background sync');
            } catch (err) {
                console.error('Background sync not available', err);
            }
        }
    }

    // In Service Worker
    self.addEventListener('sync', (event) => {
        if (event.tag.startsWith('sync-report-')) {
            const reportId = event.tag.replace('sync-report-', '');
            event.waitUntil(
                syncReportToServer(reportId)
                    .catch(err => {
                        console.error('Sync failed, will retry', err);
                        throw err;  // Retry by browser
                    })
            );
        }
    });

    async function syncReportToServer(reportId) {
        const db = new ContractorDB();
        await db.init();

        const queue = await db.getQueueItem(reportId);
        const report = await db.getDraftReport(reportId);

        const res = await fetch('/api/v1/contractor-reports/submit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(report),
        });

        if (res.ok) {
            await db.removeFromQueue(reportId);
        } else {
            throw new Error(`Sync failed: ${res.status}`);
        }
    }

PWA Manifest
============

.. code-block:: json

    // frontend/public/manifest.json
    {
      "name": "KoproGo Contractor Reports",
      "short_name": "KoproGo Field",
      "description": "Submit contractor work reports offline",
      "start_url": "/contractor",
      "scope": "/contractor/",
      "display": "standalone",
      "theme_color": "#1a365d",
      "background_color": "#ffffff",
      "orientation": "portrait-primary",
      "icons": [
        {
          "src": "/icons/icon-192x192.png",
          "sizes": "192x192",
          "type": "image/png",
          "purpose": "any"
        },
        {
          "src": "/icons/icon-512x512.png",
          "sizes": "512x512",
          "type": "image/png",
          "purpose": "any"
        },
        {
          "src": "/icons/maskable-192x192.png",
          "sizes": "192x192",
          "type": "image/png",
          "purpose": "maskable"
        }
      ],
      "screenshots": [
        {
          "src": "/images/screenshot-narrow.png",
          "sizes": "540x720",
          "type": "image/png",
          "form_factor": "narrow"
        }
      ],
      "shortcuts": [
        {
          "name": "New Report",
          "short_name": "Report",
          "description": "Create new work report",
          "url": "/contractor/new",
          "icons": [
            {
              "src": "/icons/icon-new-report.png",
              "sizes": "192x192",
              "type": "image/png"
            }
          ]
        }
      ]
    }

Conflict Resolution
===================

**When syncing offline changes to server**:

**Scenario**: Contractor submits report offline, then submits again online with conflicting edits.

**Solution**:

.. code-block:: rust

    // backend/src/application/use_cases/conflict_resolution_use_cases.rs

    pub struct ConflictResolution;

    impl ConflictResolution {
        /**
         * Three-way merge: base version + local edits + server version
         * Uses Last-Write-Wins (LWW) strategy for simplicity
         */
        pub fn merge_contractor_report(
            base: &ContractorReport,       // Original from server
            local: &ContractorReport,      // Offline edits
            remote: &ContractorReport,     // Current server version
        ) -> Result<ContractorReport> {
            let mut merged = remote.clone();

            // Merge simple fields (LWW)
            if local.updated_at > remote.updated_at {
                merged.description = local.description.clone();
                merged.work_date = local.work_date;
                merged.parts_replaced = local.parts_replaced.clone();
            }

            // Photos: combine (no conflict possible)
            let mut all_photos = remote.photos.clone();
            for local_photo in &local.photos {
                if !all_photos.iter().any(|p| p.id == local_photo.id) {
                    all_photos.push(local_photo.clone());
                }
            }
            merged.photos = all_photos;

            // Status: only allow transitions (no downgrade)
            if self.is_valid_transition(&remote.status, &local.status) {
                merged.status = local.status.clone();
            }

            Ok(merged)
        }

        fn is_valid_transition(from: &str, to: &str) -> bool {
            matches!(
                (from, to),
                ("Draft", "Draft") |
                ("Draft", "Submitted") |
                ("Submitted", "Submitted") |
                ("Submitted", "Validated") |
                (same, same)  // No change
            )
        }
    }

Limitations & Trade-offs
========================

**PWA limitations** (why Tauri is better for main app):

1. **IndexedDB quota**: ~50MB on most browsers (small reports OK, not for bulk data)
2. **Sync complexity**: Requires complex queue logic, retry logic, conflict resolution
3. **Service Worker debugging**: Hard to debug cache issues (browser DevTools limited)
4. **Security**: JWT stored in localStorage (vulnerable to XSS, unlike Tauri's OS keychain)
5. **Offline capability**: Works offline but degraded (no full app functionality)

**Why PWA still works for contractors**:

1. **Single form, simple data**: Contractor report = just text + photos (small payload)
2. **One-time sync**: Submit once, done (not continuous sync like main app)
3. **No installation**: Email link → work immediately (better UX for field workers)
4. **Throwaway device**: Often contractors use shared tablets (no persistent login needed)

Deployment
==========

**PWA Deployment Steps**:

1. Enable HTTPS (required for Service Workers)
2. Create `manifest.json` in `frontend/public/`
3. Create Service Worker file `frontend/public/sw.js`
4. Register in HTML: `<link rel="manifest" href="/manifest.json">`
5. Add to `index.html` (from Astro layout):

   .. code-block:: html

       <meta name="theme-color" content="#1a365d">
       <meta name="apple-mobile-web-app-capable" content="yes">
       <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
       <meta name="apple-mobile-web-app-title" content="KoproGo Contractor">
       <link rel="apple-touch-icon" href="/icons/apple-touch-icon.png">

6. Test with Chrome DevTools → "Lighthouse" → PWA audit
7. Deploy (already included in Astro build)

Performance Metrics
===================

**Offline-first PWA** (Contractor Reports):

* **Load time (first visit)**: ~3s (requires download)
* **Load time (cached)**: ~200ms (from Service Worker cache)
* **Offline availability**: Immediate (cached assets)
* **Storage used**: ~10MB per report (text + photos)
* **Battery impact**: Minimal (no continuous sync)

**Compared to Tauri** (Syndic Desktop):

* Load time: ~500ms (native app)
* Storage: Unlimited (native SQLite)
* Battery: Better (no browser overhead)
* Offline: Full functionality (all data synced)

Monitoring & Debugging
======================

**Service Worker Debugging** (Chrome DevTools):

.. code-block:: javascript

    // View in: DevTools → Application → Service Workers
    navigator.serviceWorker.getRegistrations()
        .then(registrations => {
            registrations.forEach(reg => {
                console.log('Active SW:', reg.active);
                console.log('Waiting SW:', reg.waiting);
                console.log('Installing SW:', reg.installing);
            });
        });

    // Clear cache manually
    caches.keys().then(cacheNames => {
        cacheNames.forEach(name => caches.delete(name));
    });

**IndexedDB Debugging**:

.. code-block:: javascript

    // View in: DevTools → Application → IndexedDB
    indexedDB.databases().then(databases => {
        databases.forEach(db => {
            console.log('Database:', db.name, 'Version:', db.version);
        });
    });

**Analytics**:

* Track sync success rate (CloudFlare Workers analytics)
* Monitor offline form submissions
* Measure time-to-submit offline vs. online

Future Work
===========

1. **Encryption at rest**: Encrypt IndexedDB data client-side (adds complexity)
2. **Delta sync**: Only sync changed fields (reduce bandwidth)
3. **Peer sync**: Use WebRTC to sync between devices (experimental)
4. **Analytics**: Track which contractors use offline, for how long
5. **Fallback**: If Service Worker fails, degrade gracefully to online-only

Recommendations
===============

**For Contractor Reports (Issue #275)**:

1. ✓ Implement PWA with Service Workers
2. ✓ Use IndexedDB for draft storage
3. ✓ Enable background sync for reliability
4. ✓ Test on low-bandwidth networks (3G) and offline

**For Other Users**:

1. Use **Tauri desktop** app (main app, better sync)
2. PWA only as **fallback** for web-only users

**Security Notes**:

1. Never store sensitive data in localStorage (use IndexedDB + encryption)
2. Always use HTTPS for PWA (required by browsers anyway)
3. Validate JWT tokens server-side before processing reports
4. Set proper CORS headers on service worker endpoints
5. Consider CSP (Content Security Policy) to prevent malicious scripts

Success Criteria
================

* **Offline availability**: Reports work 100% offline, anytime/anywhere
* **Sync reliability**: 99%+ successful submissions when online
* **User adoption**: 80%+ of contractors access via PWA by EOY 2026
* **Performance**: Form loads < 1s on cached visit
* **Storage**: Handles 50+ reports with photos per device
