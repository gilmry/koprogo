// Story 3.3 — Service Worker for KoproGo Contractor PWA (scope: /c/).
//
// Cache strategy:
//   - network-first for `POST /magic-links` and any mutation (never cache mutations).
//   - cache-first (short TTL ~5 min) for `GET /c/<token>` HTML (the contractor
//     revisits the page during the flow — going through a slow link must not
//     show a stale "already consumed" error each time).
//   - cache-only for static assets under /icons/* and /manifest.webmanifest.
//
// @negative (cf. apprentissage #549): a new release can keep serving an
// obsolete SW from cache. Mitigation:
//   1. bump `SW_VERSION` at each release (CI gate to come).
//   2. `skipWaiting` + `clients.claim` for IMMEDIATE activation.
//   3. cache name embeds SW_VERSION → old caches are purged on activate.
//
// IMPORTANT for downstream consumers: any browser consumer SHOULD call
// `/c/<token>?v=<sw-version>` on a re-visit after an upgrade to bypass the
// HTTP cache for the initial document fetch (the SW will then drop the query
// string before serving from its own cache).

const SW_VERSION = "2026-06-07-1";
const CACHE_NAME = `koprogo-c-${SW_VERSION}`;

const STATIC_ASSETS = [
  "/manifest.webmanifest",
  "/icons/pwa-192.png",
  "/icons/pwa-512.png",
  "/icons/pwa-maskable-512.png",
];

// Short TTL for HTML responses (5 minutes) — enough to survive a slow signal
// while filling a form, not enough to mask a backend deploy.
const HTML_CACHE_TTL_MS = 5 * 60 * 1000;

// --------------------------------------------------------------------------
// Lifecycle
// --------------------------------------------------------------------------

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) =>
        // Static assets — best-effort; if any is missing in this build we
        // don't want install to fail and lock the page out of the SW.
        Promise.all(
          STATIC_ASSETS.map((url) =>
            cache.add(url).catch(() => {
              // Asset not present (e.g. icons not generated yet) — ignore.
            }),
          ),
        ),
      )
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

// --------------------------------------------------------------------------
// Fetch strategy router
// --------------------------------------------------------------------------

self.addEventListener("fetch", (event) => {
  const req = event.request;
  const url = new URL(req.url);

  // Only handle same-origin requests under our scope (/c/) + the static
  // assets explicitly cached above. Anything else passes through.
  const inScope =
    url.pathname.startsWith("/c/") ||
    STATIC_ASSETS.includes(url.pathname) ||
    url.pathname.startsWith("/icons/");

  if (!inScope) return;

  // Mutations: network-first, never cache the response.
  if (req.method !== "GET") {
    event.respondWith(networkOnly(req));
    return;
  }

  // Static assets: cache-only fast path, falling back to network if missing.
  if (
    STATIC_ASSETS.includes(url.pathname) ||
    url.pathname.startsWith("/icons/")
  ) {
    event.respondWith(cacheFirstStatic(req));
    return;
  }

  // /c/<token> HTML: cache-first with short TTL.
  if (url.pathname.startsWith("/c/")) {
    event.respondWith(cacheFirstHtml(req));
    return;
  }
});

// --------------------------------------------------------------------------
// Strategies
// --------------------------------------------------------------------------

async function networkOnly(req) {
  try {
    return await fetch(req);
  } catch (err) {
    // Offline mutation: the page's JS handles the IndexedDB draft fallback.
    // Returning 503 lets the page detect the failure and queue.
    return new Response(
      JSON.stringify({ error: "offline", kind: "sw_offline_mutation" }),
      {
        status: 503,
        headers: { "Content-Type": "application/json" },
      },
    );
  }
}

async function cacheFirstStatic(req) {
  const cache = await caches.open(CACHE_NAME);
  const hit = await cache.match(req);
  if (hit) return hit;
  try {
    const resp = await fetch(req);
    if (resp.ok) {
      cache.put(req, resp.clone());
    }
    return resp;
  } catch (err) {
    return new Response("", { status: 504 });
  }
}

async function cacheFirstHtml(req) {
  const cache = await caches.open(CACHE_NAME);
  // Normalize: drop the `?v=<sw-version>` cache-bust query so re-visits hit
  // the same cache entry as the original fetch.
  const normalized = new Request(stripVersionQuery(req.url), {
    method: "GET",
    headers: req.headers,
    credentials: req.credentials,
    redirect: req.redirect,
  });

  const hit = await cache.match(normalized);
  if (hit) {
    const cachedAt = Number(hit.headers.get("x-koprogo-sw-cached-at") || "0");
    if (cachedAt && Date.now() - cachedAt < HTML_CACHE_TTL_MS) {
      return hit;
    }
    // Stale: try to refresh, but fall back to stale on offline.
    try {
      const fresh = await fetch(req);
      if (fresh.ok) {
        cache.put(normalized, await stampResponse(fresh.clone()));
      }
      return fresh;
    } catch (err) {
      return hit;
    }
  }

  try {
    const resp = await fetch(req);
    if (resp.ok) {
      cache.put(normalized, await stampResponse(resp.clone()));
    }
    return resp;
  } catch (err) {
    return new Response(
      "<!doctype html><html><body><p>Hors ligne — relancez la page lorsque vous serez reconnecté.</p></body></html>",
      { status: 503, headers: { "Content-Type": "text/html; charset=utf-8" } },
    );
  }
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

function stripVersionQuery(rawUrl) {
  const u = new URL(rawUrl);
  u.searchParams.delete("v");
  return u.toString();
}

async function stampResponse(resp) {
  const headers = new Headers(resp.headers);
  headers.set("x-koprogo-sw-cached-at", String(Date.now()));
  const body = await resp.arrayBuffer();
  return new Response(body, {
    status: resp.status,
    statusText: resp.statusText,
    headers,
  });
}
