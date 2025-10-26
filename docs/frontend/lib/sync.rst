sync.ts - Service de Synchronisation Offline
============================================

**Localisation** : ``frontend/src/lib/sync.ts``

Service de synchronisation bidirectionnelle entre IndexedDB local et API backend, permettant fonctionnalité offline-first.

Architecture Offline-First
--------------------------

**Principe** : L'application fonctionne d'abord avec données locales, synchronise avec backend quand disponible.

**Flow** :

.. code-block:: text

   Online                          Offline
   ------                          -------
   API → IndexedDB → UI            IndexedDB → UI
         ↓                         ↓
   sync_queue vide                 sync_queue remplie
                                   ↓
                                   Retour online
                                   ↓
                                   sync_queue → API
                                   ↓
                                   IndexedDB mise à jour

**Avantages** :

- ✅ Fonctionne sans connexion internet
- ✅ Améliore performance (pas de latence réseau)
- ✅ Meilleure UX (pas de loading spinners)
- ✅ Progressive Web App ready

Classe SyncService
------------------

Propriétés
^^^^^^^^^^

.. code-block:: typescript

   export class SyncService {
     private isOnline: boolean;          // Statut connexion
     private syncInProgress: boolean;    // Empêche syncs concurrentes
     private token: string | null;       // JWT token
   }

**isOnline** : Détecté via ``navigator.onLine`` (peut être imprécis).

**syncInProgress** : Mutex pour éviter syncs concurrentes.

**token** : JWT nécessaire pour authentifier requêtes API.

Constructor
^^^^^^^^^^^

Écoute les événements online/offline du navigateur.

.. code-block:: typescript

   constructor() {
     if (typeof window !== "undefined") {
       window.addEventListener("online", () => {
         console.log("🟢 Application is online");
         this.isOnline = true;
         this.sync();  // Synchroniser automatiquement
       });

       window.addEventListener("offline", () => {
         console.log("🔴 Application is offline");
         this.isOnline = false;
       });
     }
   }

**⚠️ navigator.onLine** : Pas 100% fiable (peut dire online même si backend inaccessible).

Méthodes Publiques
------------------

initialize(token)
^^^^^^^^^^^^^^^^^

Initialise le service avec JWT token et lance première sync.

.. code-block:: typescript

   async initialize(token: string): Promise<void> {
     this.setToken(token);
     await localDB.init();

     if (this.isOnline) {
       await this.sync();
     }
   }

**Appel au login** :

.. code-block:: typescript

   // Component login
   const response = await api.post('/auth/login', { email, password });
   const token = response.token;

   localStorage.setItem('koprogo_token', token);
   await syncService.initialize(token);

   // → Toutes les données synchronisées en background

sync()
^^^^^^

Synchronise toutes les modifications locales vers backend, puis télécharge données fraîches.

.. code-block:: typescript

   async sync(): Promise<void> {
     if (!this.isOnline || this.syncInProgress || !this.token) {
       return;
     }

     this.syncInProgress = true;
     console.log("🔄 Starting synchronization...");

     try {
       // 1. Pousser modifications locales
       const queue = await localDB.getSyncQueue();
       const unsyncedItems = queue.filter((item) => !item.synced);

       for (const item of unsyncedItems) {
         try {
           await this.syncItem(item);
           await localDB.markSynced(item.id!);
         } catch (error) {
           console.error(`Failed to sync item:`, error);
           // Continue même si erreur (retry au prochain sync)
         }
       }

       // 2. Nettoyer queue
       await localDB.clearSyncedItems();

       // 3. Télécharger données fraîches
       await this.fetchAllData();

       console.log("✅ Synchronization completed");
     } catch (error) {
       console.error("❌ Synchronization failed:", error);
     } finally {
       this.syncInProgress = false;
     }
   }

**Déclencheurs** :

- Au retour online (``window.addEventListener('online')``)
- Au login (``initialize()``)
- Manuellement (bouton refresh)
- Périodiquement (setInterval, optionnel)

clearLocalData()
^^^^^^^^^^^^^^^^

Vide toutes les données locales (logout).

.. code-block:: typescript

   async clearLocalData(): Promise<void> {
     this.token = null;
     await localDB.clear("users");
     await localDB.clear("buildings");
     await localDB.clear("owners");
     await localDB.clear("units");
     await localDB.clear("expenses");
     await localDB.clear("sync_queue");
   }

**Appel au logout** :

.. code-block:: typescript

   async function logout() {
     await syncService.clearLocalData();
     localStorage.removeItem('koprogo_token');
     window.location.href = '/login';
   }

Méthodes API avec Fallback
---------------------------

getBuildings()
^^^^^^^^^^^^^^

Récupère immeubles avec fallback offline.

.. code-block:: typescript

   async getBuildings(): Promise<Building[]> {
     if (this.isOnline && this.token) {
       try {
         const response = await this.fetchWithAuth('/buildings');
         if (response.ok) {
           const result = await response.json();
           const buildings = result.data || result;
           await localDB.saveBuildings(buildings);
           return buildings;
         }
       } catch (error) {
         console.log("Falling back to local data");
       }
     }

     // Fallback local
     return localDB.getBuildings();
   }

**Flow** :

1. Si online + token → essayer API
2. Si succès → sauvegarder dans IndexedDB + retourner
3. Si échec ou offline → retourner données locales IndexedDB

createBuilding(building)
^^^^^^^^^^^^^^^^^^^^^^^^

Crée immeuble avec queue offline.

.. code-block:: typescript

   async createBuilding(building: Partial<Building>): Promise<Building | null> {
     if (this.isOnline && this.token) {
       try {
         const response = await this.fetchWithAuth('/buildings', {
           method: "POST",
           body: JSON.stringify(building)
         });

         if (response.ok) {
           const newBuilding = await response.json();
           await localDB.put("buildings", newBuilding);
           return newBuilding;
         }
       } catch (error) {
         console.log("Offline: queueing building creation");
       }
     }

     // Queue pour sync ultérieure
     await localDB.addToSyncQueue("create", "buildings", building);

     // Créer record temporaire local
     const tempBuilding = {
       id: `temp-${Date.now()}`,
       ...building,
       createdAt: new Date().toISOString()
     } as Building;

     await localDB.put("buildings", tempBuilding);
     return tempBuilding;
   }

**IDs Temporaires** : Préfixe ``temp-`` pour différencier locaux vs backend.

**Résolution IDs** : Lors de sync, backend retourne vrai ID, remplacer local.

Méthodes Privées
----------------

syncItem(item)
^^^^^^^^^^^^^^

Synchronise un élément de la queue vers backend.

.. code-block:: typescript

   private async syncItem(item: SyncQueue): Promise<void> {
     const { action, entity, data } = item;
     let url = `${API_BASE_URL}/${entity}`;

     switch (action) {
       case "create":
         await this.fetchWithAuth(url, {
           method: "POST",
           body: JSON.stringify(data)
         });
         break;

       case "update":
         url = `${url}/${data.id}`;
         await this.fetchWithAuth(url, {
           method: "PUT",
           body: JSON.stringify(data)
         });
         break;

       case "delete":
         url = `${url}/${data.id}`;
         await this.fetchWithAuth(url, {
           method: "DELETE"
         });
         break;
     }
   }

fetchAllData()
^^^^^^^^^^^^^^

Télécharge toutes les données depuis backend et sauvegarde localement.

.. code-block:: typescript

   private async fetchAllData(): Promise<void> {
     if (!this.isOnline || !this.token) return;

     try {
       // Fetch buildings
       const buildingsRes = await this.fetchWithAuth('/buildings');
       if (buildingsRes.ok) {
         const response = await buildingsRes.json();
         const buildings = response.data || response;
         await localDB.saveBuildings(buildings);
       }

       // Fetch owners
       const ownersRes = await this.fetchWithAuth('/owners');
       if (ownersRes.ok) {
         const response = await ownersRes.json();
         const owners = response.data || response;
         await localDB.saveOwners(owners);
       }

       // Note: Units et expenses nécessitent endpoints spécifiques
     } catch (error) {
       console.error("Failed to fetch data from server:", error);
     }
   }

fetchWithAuth(url, options)
^^^^^^^^^^^^^^^^^^^^^^^^^^^

Wrapper fetch avec JWT automatique.

.. code-block:: typescript

   private async fetchWithAuth(
     url: string,
     options: RequestInit = {}
   ): Promise<Response> {
     const headers = new Headers(options.headers);

     if (this.token) {
       headers.set("Authorization", `Bearer ${token}`);
     }
     headers.set("Content-Type", "application/json");

     return fetch(url, {
       ...options,
       headers
     });
   }

Utilisation dans Components
----------------------------

**Import** :

.. code-block:: svelte

   <script lang="ts">
     import { syncService } from '../lib/sync';
     import { onMount } from 'svelte';

     let buildings: Building[] = [];
     let syncing = false;

     onMount(async () => {
       buildings = await syncService.getBuildings();
     });

     async function refresh() {
       syncing = true;
       await syncService.sync();
       buildings = await syncService.getBuildings();
       syncing = false;
     }
   </script>

**Template** :

.. code-block:: svelte

   <button on:click={refresh} disabled={syncing}>
     {syncing ? 'Synchronisation...' : 'Rafraîchir'}
   </button>

   {#each buildings as building}
     <BuildingCard {building} />
   {/each}

SyncStatus Component
--------------------

Indicateur visuel statut connexion.

.. code-block:: svelte

   <script lang="ts">
     import { syncService } from '../lib/sync';
     import { onMount } from 'svelte';

     let isOnline = syncService.getOnlineStatus();

     onMount(() => {
       const interval = setInterval(() => {
         isOnline = syncService.getOnlineStatus();
       }, 1000);

       return () => clearInterval(interval);
     });
   </script>

   <div class="sync-status">
     {#if isOnline}
       <span class="text-green-500">🟢 En ligne</span>
     {:else}
       <span class="text-orange-500">🔴 Hors ligne</span>
     {/if}
   </div>

Synchronisation Périodique
---------------------------

**Auto-sync toutes les 5 minutes** :

.. code-block:: typescript

   // Component racine ou Layout
   onMount(() => {
     const syncInterval = setInterval(async () => {
       if (syncService.getOnlineStatus()) {
         await syncService.sync();
       }
     }, 5 * 60 * 1000);  // 5 minutes

     return () => clearInterval(syncInterval);
   });

Gestion Conflits
----------------

**Problème** : Données modifiées offline + backend modifié entretemps = conflit.

**Stratégie Actuelle** : Last-Write-Wins (dernière écriture gagne).

**Amélioration Future** :

1. **Timestamps** : Comparer ``updated_at`` local vs backend

   .. code-block:: typescript

      if (local.updated_at > backend.updated_at) {
        // Modification locale plus récente
        await api.put(`/buildings/${id}`, local);
      } else {
        // Backend plus récent
        await localDB.put('buildings', backend);
      }

2. **Version Vectors** : Détecter modifications concurrentes

3. **UI Résolution Manuelle** : Afficher dialogue à l'utilisateur

   .. code-block:: svelte

      {#if conflict}
        <ConflictResolutionDialog
          local={conflict.local}
          remote={conflict.remote}
          on:resolve={handleResolve}
        />
      {/if}

Limitations Connues
-------------------

1. **navigator.onLine imprécis** :

   Peut dire online même si backend inaccessible (DNS résout, mais serveur down).

   **Solution** : Ping health check périodique.

   .. code-block:: typescript

      async function checkBackendAvailable(): Promise<boolean> {
        try {
          const response = await fetch(`${API_URL}/health`, {
            method: 'HEAD',
            timeout: 5000
          });
          return response.ok;
        } catch {
          return false;
        }
      }

2. **Pas de Retry Automatique** :

   Si sync échoue, attendre prochain trigger manuel.

   **Solution** : Exponential backoff retry.

3. **Pas de Résolution Conflits** :

   Last-write-wins seulement.

4. **Sync Complète** :

   ``fetchAllData()`` télécharge tout, pas de delta sync.

   **Solution** : Endpoint ``/sync?since=timestamp`` pour delta.

5. **Pas de Webhooks/WebSockets** :

   Pas de push notifications quand backend change.

   **Solution** : WebSocket ou Server-Sent Events.

Tests Sync Service
------------------

.. code-block:: typescript

   // tests/unit/sync.test.ts
   import { describe, it, expect, vi, beforeEach } from 'vitest';
   import { syncService } from '../src/lib/sync';
   import { localDB } from '../src/lib/db';

   describe('syncService', () => {
     beforeEach(async () => {
       await localDB.init();
       await localDB.clear('sync_queue');
     });

     it('should queue offline modifications', async () => {
       // Simuler offline
       vi.spyOn(syncService, 'getOnlineStatus').mockReturnValue(false);

       await syncService.createBuilding({
         name: 'Test Building'
       });

       const queue = await localDB.getSyncQueue();
       expect(queue).toHaveLength(1);
       expect(queue[0].action).toBe('create');
     });

     it('should sync queue when back online', async () => {
       // Ajouter item à la queue
       await localDB.addToSyncQueue('create', 'buildings', {
         name: 'Test'
       });

       // Mock API
       global.fetch = vi.fn(() => Promise.resolve({
         ok: true,
         json: () => Promise.resolve({ id: '123', name: 'Test' })
       }));

       // Synchroniser
       await syncService.sync();

       // Vérifier queue vide
       const queue = await localDB.getSyncQueue();
       expect(queue).toHaveLength(0);
     });
   });

Performance Optimisations
-------------------------

1. **Debounce Sync** : Éviter syncs trop fréquentes

   .. code-block:: typescript

      let syncTimeout: NodeJS.Timeout;

      function debouncedSync() {
        clearTimeout(syncTimeout);
        syncTimeout = setTimeout(() => {
          syncService.sync();
        }, 2000);  // 2 secondes après dernière modification
      }

2. **Sync Partielle** : Synchroniser seulement entités modifiées

   .. code-block:: typescript

      async syncBuildings() {
        const queue = await localDB.getSyncQueue();
        const buildingItems = queue.filter(item => item.entity === 'buildings');
        // Sync uniquement buildings
      }

3. **Background Sync API** : Service Worker background sync

   .. code-block:: typescript

      // Service Worker
      self.addEventListener('sync', (event) => {
        if (event.tag === 'koprogo-sync') {
          event.waitUntil(syncService.sync());
        }
      });

Extensions Futures
------------------

1. **Conflict Resolution UI** : Dialogue résolution manuelle
2. **Delta Sync** : Endpoint ``/sync?since=timestamp``
3. **WebSocket Real-time** : Push notifications changements backend
4. **Offline Indicators** : Badges "Non synchronisé" sur éléments
5. **Selective Sync** : Utilisateur choisit quelles données synchroniser
6. **Encryption** : Chiffrer données sensibles dans IndexedDB

Références
----------

- IndexedDB Client : ``frontend/src/lib/db.ts``
- API Client : ``frontend/src/lib/api.ts``
- SyncStatus Component : ``frontend/src/components/SyncStatus.svelte``
- Service Worker : ``frontend/public/sw.js`` (à créer)
