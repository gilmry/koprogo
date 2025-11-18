// KoproGo Service Worker
// Handles offline caching and background sync

const CACHE_NAME = "koprogo-v1";
const OFFLINE_CACHE = "koprogo-offline-v1";
const API_CACHE = "koprogo-api-v1";

// Assets to cache immediately
const PRECACHE_ASSETS = [
  "/",
  "/dashboard",
  "/buildings",
  "/documents",
  "/offline.html",
  "/icons/icon-192x192.png",
  "/icons/icon-512x512.png",
];

// API endpoints to cache
const API_CACHE_PATTERNS = [
  "/api/v1/buildings",
  "/api/v1/units",
  "/api/v1/owners",
  "/api/v1/documents",
];

// Install event - cache assets
self.addEventListener("install", (event) => {
  console.log("[Service Worker] Installing...");

  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => {
        console.log("[Service Worker] Precaching assets");
        return cache.addAll(PRECACHE_ASSETS);
      })
      .then(() => self.skipWaiting()), // Activate immediately
  );
});

// Activate event - clean up old caches
self.addEventListener("activate", (event) => {
  console.log("[Service Worker] Activating...");

  event.waitUntil(
    caches
      .keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames
            .filter(
              (name) =>
                name !== CACHE_NAME &&
                name !== OFFLINE_CACHE &&
                name !== API_CACHE,
            )
            .map((name) => {
              console.log("[Service Worker] Deleting old cache:", name);
              return caches.delete(name);
            }),
        );
      })
      .then(() => self.clients.claim()), // Take control immediately
  );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener("fetch", (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== "GET") {
    return;
  }

  // Skip chrome-extension and other non-http(s) requests
  if (!request.url.startsWith("http")) {
    return;
  }

  // Handle API requests (Network First strategy)
  if (url.pathname.startsWith("/api/")) {
    event.respondWith(networkFirstStrategy(request));
    return;
  }

  // Handle static assets (Cache First strategy)
  event.respondWith(cacheFirstStrategy(request));
});

// Cache First Strategy - for static assets
async function cacheFirstStrategy(request) {
  try {
    const cache = await caches.open(CACHE_NAME);
    const cachedResponse = await cache.match(request);

    if (cachedResponse) {
      console.log("[Service Worker] Serving from cache:", request.url);
      return cachedResponse;
    }

    console.log("[Service Worker] Fetching from network:", request.url);
    const networkResponse = await fetch(request);

    // Cache successful responses
    if (networkResponse && networkResponse.status === 200) {
      cache.put(request, networkResponse.clone());
    }

    return networkResponse;
  } catch (error) {
    console.error("[Service Worker] Cache First failed:", error);

    // Return offline page for navigation requests
    if (request.destination === "document") {
      const offlineCache = await caches.open(OFFLINE_CACHE);
      return offlineCache.match("/offline.html");
    }

    throw error;
  }
}

// Network First Strategy - for API calls
async function networkFirstStrategy(request) {
  try {
    console.log("[Service Worker] API call - Network First:", request.url);
    const networkResponse = await fetch(request);

    // Cache successful GET responses
    if (
      networkResponse &&
      networkResponse.status === 200 &&
      request.method === "GET"
    ) {
      const cache = await caches.open(API_CACHE);
      cache.put(request, networkResponse.clone());
    }

    return networkResponse;
  } catch (error) {
    console.error(
      "[Service Worker] Network First failed, trying cache:",
      error,
    );

    // Fallback to cache if network fails
    const cache = await caches.open(API_CACHE);
    const cachedResponse = await cache.match(request);

    if (cachedResponse) {
      console.log("[Service Worker] Serving API from cache:", request.url);
      // Add custom header to indicate cached response
      const headers = new Headers(cachedResponse.headers);
      headers.set("X-From-Cache", "true");

      return new Response(cachedResponse.body, {
        status: cachedResponse.status,
        statusText: cachedResponse.statusText,
        headers: headers,
      });
    }

    throw error;
  }
}

// Background Sync - for POST/PUT/DELETE when offline
self.addEventListener("sync", (event) => {
  console.log("[Service Worker] Background sync:", event.tag);

  if (event.tag === "sync-data") {
    event.waitUntil(syncData());
  }
});

async function syncData() {
  console.log("[Service Worker] Syncing offline data...");

  try {
    // Open IndexedDB and get pending requests
    const db = await openIndexedDB();
    const pendingRequests = await getPendingRequests(db);

    // Replay pending requests
    for (const request of pendingRequests) {
      try {
        await fetch(request.url, {
          method: request.method,
          headers: request.headers,
          body: request.body,
        });

        // Remove from pending queue
        await removePendingRequest(db, request.id);
        console.log("[Service Worker] Synced request:", request.url);
      } catch (error) {
        console.error("[Service Worker] Failed to sync request:", error);
      }
    }

    console.log("[Service Worker] Sync complete");
  } catch (error) {
    console.error("[Service Worker] Sync failed:", error);
  }
}

// IndexedDB helpers
function openIndexedDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("koprogo-offline", 1);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = (event) => {
      const db = event.target.result;

      if (!db.objectStoreNames.contains("pendingRequests")) {
        db.createObjectStore("pendingRequests", {
          keyPath: "id",
          autoIncrement: true,
        });
      }

      if (!db.objectStoreNames.contains("cachedData")) {
        db.createObjectStore("cachedData", { keyPath: "key" });
      }
    };
  });
}

function getPendingRequests(db) {
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(["pendingRequests"], "readonly");
    const store = transaction.objectStore("pendingRequests");
    const request = store.getAll();

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function removePendingRequest(db, id) {
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(["pendingRequests"], "readwrite");
    const store = transaction.objectStore("pendingRequests");
    const request = store.delete(id);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
  });
}

// Push notification handler
self.addEventListener("push", (event) => {
  if (!event.data) return;

  const data = event.data.json();
  const title = data.title || "KoproGo";
  const options = {
    body: data.body || "",
    icon: "/icons/icon-192x192.png",
    badge: "/icons/icon-72x72.png",
    vibrate: [200, 100, 200],
    data: data.url ? { url: data.url } : undefined,
    actions: data.actions || [],
  };

  event.waitUntil(self.registration.showNotification(title, options));
});

// Notification click handler
self.addEventListener("notificationclick", (event) => {
  event.notification.close();

  if (event.notification.data && event.notification.data.url) {
    event.waitUntil(clients.openWindow(event.notification.data.url));
  }
});

console.log("[Service Worker] Loaded");
