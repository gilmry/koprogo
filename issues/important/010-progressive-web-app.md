# Issue #010 - Progressive Web App (PWA)

**Priorité**: 🟡 IMPORTANT
**Estimation**: 10-12 heures
**Labels**: `enhancement`, `frontend`, `important`, `pwa`, `offline`

---

## 📋 Description

Transformer l'application frontend en Progressive Web App pour permettre :
- Installation sur mobile/desktop
- Mode hors-ligne complet
- Synchronisation en arrière-plan
- Notifications push

**Note** : Infrastructure de base existe déjà (`stores/sync.ts`, `stores/db.ts`) mais non connectée.

---

## 🎯 Objectifs

- [ ] Manifest.json pour installation
- [ ] Service Worker pour cache
- [ ] Stratégie cache-first pour assets
- [ ] Network-first pour API avec fallback
- [ ] Background sync
- [ ] Push notifications support
- [ ] Update prompts

---

## 📐 Architecture

```
Frontend
  ├── public/
  │   ├── manifest.json (PWA config)
  │   ├── sw.js (Service Worker)
  │   ├── icons/ (192x192, 512x512)
  ├── src/stores/
  │   ├── sync.ts (✅ existe, à compléter)
  │   └── db.ts (✅ existe, à compléter)
```

---

## 🔧 Implémentation

### 1. Manifest.json

```json
{
  "name": "KoproGo - Gestion de Copropriété",
  "short_name": "KoproGo",
  "description": "Plateforme de gestion de copropriété",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#2563eb",
  "icons": [
    {
      "src": "/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any maskable"
    },
    {
      "src": "/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ],
  "categories": ["productivity", "business"],
  "screenshots": [
    {
      "src": "/screenshots/dashboard.png",
      "sizes": "1280x720",
      "type": "image/png"
    }
  ]
}
```

### 2. Service Worker

**Fichier** : `public/sw.js`

```javascript
const CACHE_NAME = 'koprogo-v1';
const RUNTIME_CACHE = 'koprogo-runtime';

const STATIC_ASSETS = [
  '/',
  '/buildings',
  '/expenses',
  '/owners',
  '/offline.html',
  '/icons/icon-192.png',
];

// Install
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(STATIC_ASSETS);
    })
  );
  self.skipWaiting();
});

// Activate
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME && name !== RUNTIME_CACHE)
          .map((name) => caches.delete(name))
      );
    })
  );
  self.clients.claim();
});

// Fetch
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // API requests: Network-first
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirst(request));
  }
  // Static assets: Cache-first
  else {
    event.respondWith(cacheFirst(request));
  }
});

async function cacheFirst(request) {
  const cached = await caches.match(request);
  if (cached) return cached;

  try {
    const response = await fetch(request);
    const cache = await caches.open(RUNTIME_CACHE);
    cache.put(request, response.clone());
    return response;
  } catch (error) {
    const fallback = await caches.match('/offline.html');
    if (fallback) return fallback;
    throw error;
  }
}

async function networkFirst(request) {
  try {
    const response = await fetch(request);
    const cache = await caches.open(RUNTIME_CACHE);
    cache.put(request, response.clone());
    return response;
  } catch (error) {
    const cached = await caches.match(request);
    if (cached) return cached;
    throw error;
  }
}

// Background Sync
self.addEventListener('sync', (event) => {
  if (event.tag === 'sync-data') {
    event.waitUntil(syncData());
  }
});

async function syncData() {
  // Récupérer données IndexedDB
  // Envoyer au serveur
  console.log('Background sync triggered');
}

// Push notifications
self.addEventListener('push', (event) => {
  const data = event.data.json();

  event.waitUntil(
    self.registration.showNotification(data.title, {
      body: data.body,
      icon: '/icons/icon-192.png',
      badge: '/icons/badge-72.png',
      data: { url: data.url },
    })
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(
    clients.openWindow(event.notification.data.url)
  );
});
```

### 3. Enregistrement SW

**Fichier** : `frontend/src/lib/registerSW.ts`

```typescript
export async function registerServiceWorker() {
  if ('serviceWorker' in navigator) {
    try {
      const registration = await navigator.serviceWorker.register('/sw.js');

      console.log('Service Worker registered:', registration);

      // Écouter mises à jour
      registration.addEventListener('updatefound', () => {
        const newWorker = registration.installing;

        newWorker?.addEventListener('statechange', () => {
          if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
            // Nouvelle version disponible
            if (confirm('Nouvelle version disponible. Mettre à jour ?')) {
              newWorker.postMessage({ type: 'SKIP_WAITING' });
              window.location.reload();
            }
          }
        });
      });

      return registration;
    } catch (error) {
      console.error('SW registration failed:', error);
    }
  }
}
```

### 4. Sync Service (compléter existant)

**Fichier** : `frontend/src/stores/sync.ts`

```typescript
import { writable } from 'svelte/store';
import { db } from './db';
import { getApiUrl } from './config';
import { authStore } from './auth';

type SyncStatus = 'idle' | 'syncing' | 'error';

export const syncStatus = writable<SyncStatus>('idle');
export const lastSyncTime = writable<Date | null>(null);

export async function syncData() {
  syncStatus.set('syncing');

  try {
    // 1. Récupérer données locales modifiées
    const pendingBuildings = await db.buildings.where('synced').equals(0).toArray();

    // 2. Envoyer au serveur
    for (const building of pendingBuildings) {
      await fetch(`${getApiUrl()}/buildings`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${authStore.get().token}`,
        },
        body: JSON.stringify(building),
      });

      // 3. Marquer comme synchronisé
      await db.buildings.update(building.id, { synced: 1 });
    }

    // 4. Télécharger mises à jour serveur
    const response = await fetch(`${getApiUrl()}/buildings?updated_after=${lastSyncTime.get()?.toISOString()}`);
    const serverBuildings = await response.json();

    // 5. Mettre à jour IndexedDB
    await db.buildings.bulkPut(serverBuildings.map(b => ({ ...b, synced: 1 })));

    syncStatus.set('idle');
    lastSyncTime.set(new Date());
  } catch (error) {
    console.error('Sync error:', error);
    syncStatus.set('error');
  }
}

// Background sync si supporté
if ('serviceWorker' in navigator && 'SyncManager' in window) {
  navigator.serviceWorker.ready.then((registration) => {
    registration.sync.register('sync-data');
  });
}
```

### 5. Offline Page

**Fichier** : `public/offline.html`

```html
<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <title>Hors ligne - KoproGo</title>
  <style>
    body {
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      font-family: system-ui;
      text-align: center;
    }
  </style>
</head>
<body>
  <div>
    <h1>🔌 Vous êtes hors ligne</h1>
    <p>Vérifiez votre connexion Internet</p>
    <button onclick="location.reload()">Réessayer</button>
  </div>
</body>
</html>
```

---

## ✅ Critères d'Acceptation

### Installation
- [ ] Bouton "Installer l'app" apparaît sur Chrome/Edge
- [ ] App installable sur iOS (Add to Home Screen)
- [ ] Icônes 192x192 et 512x512 présentes

### Offline
- [ ] Pages visitées accessibles hors ligne
- [ ] Formulaires sauvegardés en local si offline
- [ ] Synchronisation automatique au retour en ligne
- [ ] Indicateur "Mode hors ligne" visible

### Performance
- [ ] Lighthouse PWA score > 90
- [ ] Assets cachés (chargement instantané)
- [ ] API requests fallback sur cache

### Notifications
- [ ] Push notifications supportées
- [ ] Permission demandée au bon moment
- [ ] Badge non lues sur icône

---

## 🧪 Tests

```javascript
// Cypress E2E
describe('PWA', () => {
  it('installs service worker', () => {
    cy.visit('/');
    cy.window().its('navigator.serviceWorker.controller').should('exist');
  });

  it('works offline', () => {
    cy.visit('/buildings');
    cy.window().then((win) => {
      win.dispatchEvent(new Event('offline'));
    });
    cy.contains('Mode hors ligne').should('be.visible');
    cy.reload(); // Should still work
  });

  it('syncs data when online', () => {
    // Create building offline
    // Go online
    // Verify building sent to server
  });
});
```

---

## 📊 Métriques Lighthouse

| Métrique | Objectif |
|----------|----------|
| PWA Score | > 90 |
| Performance | > 85 |
| Accessibility | > 95 |
| Best Practices | 100 |
| SEO | > 90 |

---

## 🚀 Checklist

- [ ] Créer manifest.json
- [ ] Générer icônes (192, 512, maskable)
- [ ] Implémenter Service Worker
- [ ] Compléter sync.ts
- [ ] Compléter db.ts (IndexedDB schema)
- [ ] Page offline.html
- [ ] Enregistrer SW au démarrage
- [ ] Indicateur sync status UI
- [ ] Tests Lighthouse
- [ ] Tests E2E offline
- [ ] Documentation utilisateur
- [ ] Commit : `feat: implement PWA with offline support`

---

**Créé le** : 2025-10-23
**Bloque** : Expérience mobile
