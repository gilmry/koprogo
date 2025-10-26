db.ts - Client IndexedDB
=========================

**Localisation** : ``frontend/src/lib/db.ts``

Wrapper autour de IndexedDB pour le stockage local offline-first.

Vue d'ensemble
--------------

**IndexedDB** : Base de données NoSQL côté client du navigateur.

**Capacité** :

- Chrome/Firefox : Illimitée (demande permission au-delà de 50 MB)
- Safari : 1 GB maximum
- Mobile : Variable selon appareil

**Cas d'Usage** :

- ✅ Stockage offline des données (buildings, owners, units, expenses)
- ✅ Queue de synchronisation (modifications en attente)
- ✅ Cache pour améliorer performance (éviter requêtes API répétées)
- ✅ Progressive Web App (PWA) fonctionnalité

Configuration Base de Données
------------------------------

.. code-block:: typescript

   const DB_NAME = "koprogo_db";
   const DB_VERSION = 1;

**Object Stores** (équivalent tables SQL) :

- ``users`` : Utilisateurs (cache profile)
- ``buildings`` : Immeubles
- ``owners`` : Copropriétaires
- ``units`` : Lots
- ``expenses`` : Charges
- ``sync_queue`` : Queue de synchronisation

**Schema** :

.. code-block:: typescript

   interface SyncQueue {
     id?: number;               // Auto-incrémenté
     action: "create" | "update" | "delete";
     entity: string;            // "buildings", "owners", etc.
     data: any;                 // Payload JSON
     timestamp: number;         // Date.now()
     synced: boolean;           // false par défaut
   }

Classe LocalDB
--------------

init()
^^^^^^

Initialise la connexion IndexedDB et crée les object stores.

.. code-block:: typescript

   async init(): Promise<void> {
     // Skip sur serveur (SSG/SSR)
     if (typeof indexedDB === "undefined") {
       return Promise.resolve();
     }

     return new Promise((resolve, reject) => {
       const request = indexedDB.open(DB_NAME, DB_VERSION);

       request.onsuccess = () => {
         this.db = request.result;
         resolve();
       };

       request.onupgradeneeded = (event) => {
         const db = (event.target as IDBOpenDBRequest).result;

         // Créer object stores
         if (!db.objectStoreNames.contains("buildings")) {
           db.createObjectStore("buildings", { keyPath: "id" });
         }
         // ...
       };
     });
   }

**onupgradeneeded** : Appelé si ``DB_VERSION`` augmente, permet migrations.

**Exemple d'utilisation** :

.. code-block:: typescript

   import { localDB } from '../lib/db';

   // Initialiser au démarrage app
   await localDB.init();

Opérations CRUD Génériques
---------------------------

get<T>(storeName, id)
^^^^^^^^^^^^^^^^^^^^^

Récupère un élément par ID.

.. code-block:: typescript

   async get<T>(storeName: string, id: string): Promise<T | null>

**Exemple** :

.. code-block:: typescript

   const building = await localDB.get<Building>('buildings', buildingId);

getAll<T>(storeName)
^^^^^^^^^^^^^^^^^^^^

Récupère tous les éléments d'un store.

.. code-block:: typescript

   async getAll<T>(storeName: string): Promise<T[]>

**Exemple** :

.. code-block:: typescript

   const buildings = await localDB.getAll<Building>('buildings');

put<T>(storeName, data)
^^^^^^^^^^^^^^^^^^^^^^^

Insère ou met à jour un élément.

.. code-block:: typescript

   async put<T>(storeName: string, data: T): Promise<void>

**Exemple** :

.. code-block:: typescript

   await localDB.put('buildings', {
     id: '123',
     name: 'Résidence du Parc',
     // ...
   });

**⚠️ put() = INSERT OR UPDATE** : Écrase si ID existe déjà.

delete(storeName, id)
^^^^^^^^^^^^^^^^^^^^^

Supprime un élément par ID.

.. code-block:: typescript

   async delete(storeName: string, id: string): Promise<void>

**Exemple** :

.. code-block:: typescript

   await localDB.delete('buildings', buildingId);

clear(storeName)
^^^^^^^^^^^^^^^^

Vide complètement un object store.

.. code-block:: typescript

   async clear(storeName: string): Promise<void>

**Exemple** :

.. code-block:: typescript

   // Vider toutes les données au logout
   await localDB.clear('buildings');
   await localDB.clear('owners');

Opérations Sync Queue
----------------------

addToSyncQueue(action, entity, data)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Ajoute une modification à la queue de synchronisation.

.. code-block:: typescript

   async addToSyncQueue(
     action: "create" | "update" | "delete",
     entity: string,
     data: any
   ): Promise<void>

**Exemple** :

.. code-block:: typescript

   // Utilisateur crée un immeuble offline
   await localDB.addToSyncQueue('create', 'buildings', {
     name: 'Nouveau immeuble',
     address: '123 Rue Example'
   });

   // Plus tard, quand online, sync.ts traitera cette queue

getSyncQueue()
^^^^^^^^^^^^^^

Récupère tous les éléments de la queue.

.. code-block:: typescript

   async getSyncQueue(): Promise<SyncQueue[]>

**Exemple** :

.. code-block:: typescript

   const queue = await localDB.getSyncQueue();
   const pendingItems = queue.filter(item => !item.synced);

   console.log(`${pendingItems.length} modifications en attente`);

markSynced(id)
^^^^^^^^^^^^^^

Marque un élément de la queue comme synchronisé.

.. code-block:: typescript

   async markSynced(id: number): Promise<void>

**Exemple** :

.. code-block:: typescript

   for (const item of queue) {
     try {
       await syncItemToBackend(item);
       await localDB.markSynced(item.id!);
     } catch (error) {
       console.error('Sync failed:', error);
     }
   }

clearSyncedItems()
^^^^^^^^^^^^^^^^^^

Supprime tous les éléments synchronisés de la queue.

.. code-block:: typescript

   async clearSyncedItems(): Promise<void>

**Exemple** :

.. code-block:: typescript

   // Après synchronisation complète
   await localDB.clearSyncedItems();

Méthodes Spécifiques Entités
-----------------------------

saveBuildings(buildings)
^^^^^^^^^^^^^^^^^^^^^^^^

Sauvegarde plusieurs immeubles.

.. code-block:: typescript

   async saveBuildings(buildings: Building[]): Promise<void> {
     for (const building of buildings) {
       await this.put("buildings", building);
     }
   }

**Exemple** :

.. code-block:: typescript

   // Après fetch API
   const response = await api.get('/buildings');
   await localDB.saveBuildings(response.data);

getBuildings()
^^^^^^^^^^^^^^

Récupère tous les immeubles.

.. code-block:: typescript

   async getBuildings(): Promise<Building[]>

**Méthodes similaires** :

- ``saveOwners(owners)`` / ``getOwners()``
- ``saveUnits(units)`` / ``getUnits()``
- ``saveExpenses(expenses)`` / ``getExpenses()``
- ``saveUser(user)`` / ``getUser(id)``

Pattern d'Utilisation avec sync.ts
-----------------------------------

**Workflow Offline-First** :

.. code-block:: typescript

   // 1. Composant Svelte essaie API
   async function loadBuildings() {
     if (navigator.onLine) {
       try {
         // Essayer API d'abord
         const response = await api.get('/buildings');
         await localDB.saveBuildings(response.data);
         return response.data;
       } catch (error) {
         console.log('API failed, using local cache');
       }
     }

     // 2. Fallback sur IndexedDB
     return await localDB.getBuildings();
   }

**Workflow Create Offline** :

.. code-block:: typescript

   async function createBuilding(data: Partial<Building>) {
     if (navigator.onLine) {
       try {
         // Online: POST direct
         const building = await api.post('/buildings', data);
         await localDB.put('buildings', building);
         return building;
       } catch (error) {
         console.log('API unavailable, queueing...');
       }
     }

     // Offline: Créer localement + queue
     const tempId = `temp-${Date.now()}`;
     const tempBuilding = { id: tempId, ...data } as Building;

     await localDB.put('buildings', tempBuilding);
     await localDB.addToSyncQueue('create', 'buildings', data);

     return tempBuilding;
   }

Debugging IndexedDB
-------------------

**Chrome DevTools** :

1. Ouvrir DevTools (F12)
2. Onglet **Application**
3. Section **IndexedDB** → **koprogo_db**
4. Explorer object stores, inspecter données

**Firefox DevTools** :

1. Ouvrir DevTools (F12)
2. Onglet **Storage**
3. Section **Indexed DB** → **koprogo_db**

**Console Debug** :

.. code-block:: typescript

   // Afficher contenu complet
   const buildings = await localDB.getBuildings();
   console.table(buildings);

   const queue = await localDB.getSyncQueue();
   console.table(queue);

**Vider données** :

.. code-block:: typescript

   // Console browser
   indexedDB.deleteDatabase('koprogo_db');
   // Puis recharger page

Migrations Schema
-----------------

Si besoin d'ajouter un object store ou index :

.. code-block:: typescript

   const DB_VERSION = 2;  // Incrémenter version

   request.onupgradeneeded = (event) => {
     const db = (event.target as IDBOpenDBRequest).result;
     const oldVersion = event.oldVersion;

     // Migration v1 → v2
     if (oldVersion < 2) {
       if (!db.objectStoreNames.contains("meetings")) {
         db.createObjectStore("meetings", { keyPath: "id" });
       }
     }
   };

**⚠️ Incrémenter DB_VERSION** : Déclenche ``onupgradeneeded``.

Limitations IndexedDB
---------------------

1. **Pas de Relations** :

   IndexedDB est NoSQL, pas de JOIN. Nécessite récupérations multiples.

   .. code-block:: typescript

      // Récupérer building + units associés
      const building = await localDB.get('buildings', buildingId);
      const allUnits = await localDB.getAll('units');
      const buildingUnits = allUnits.filter(u => u.building_id === buildingId);

2. **Pas de Requêtes Complexes** :

   Pas de WHERE, ORDER BY, GROUP BY natifs. Filtrer en JavaScript.

   .. code-block:: typescript

      const expenses = await localDB.getAll('expenses');
      const unpaidExpenses = expenses
        .filter(e => e.payment_status === 'Pending')
        .sort((a, b) => new Date(a.due_date) - new Date(b.due_date));

3. **Performance avec Gros Volumes** :

   getAll() charge tout en mémoire. Pour > 10,000 items, utiliser cursor.

4. **Pas de Full-Text Search** :

   Pas d'indexation texte. Pour recherche, utiliser bibliothèque externe (Fuse.js).

5. **API Asynchrone Complexe** :

   Callbacks IDBRequest, pas de Promise native (wrapper requis).

Extensions Futures
------------------

1. **Indexes** :

   Créer indexes pour requêtes performantes.

   .. code-block:: typescript

      const store = db.createObjectStore("expenses", { keyPath: "id" });
      store.createIndex("building_id", "building_id", { unique: false });
      store.createIndex("payment_status", "payment_status", { unique: false });

2. **Cursors** :

   Itérer gros datasets sans charger tout en mémoire.

   .. code-block:: typescript

      const transaction = db.transaction("buildings", "readonly");
      const store = transaction.objectStore("buildings");
      const request = store.openCursor();

      request.onsuccess = (event) => {
        const cursor = event.target.result;
        if (cursor) {
          console.log(cursor.value);
          cursor.continue();
        }
      };

3. **Compression** :

   Compresser données avant stockage (LZ-string).

   .. code-block:: typescript

      import LZString from 'lz-string';

      const compressed = LZString.compress(JSON.stringify(buildings));
      await localDB.put('cache', { key: 'buildings', data: compressed });

4. **Encryption** :

   Chiffrer données sensibles (crypto-js).

   .. code-block:: typescript

      import CryptoJS from 'crypto-js';

      const encrypted = CryptoJS.AES.encrypt(
        JSON.stringify(owner),
        'secret-key'
      ).toString();

Tests IndexedDB
---------------

**Mock IndexedDB** :

.. code-block:: typescript

   // vitest.setup.ts
   import 'fake-indexeddb/auto';

   // tests/unit/db.test.ts
   import { describe, it, expect, beforeEach } from 'vitest';
   import { localDB } from '../src/lib/db';

   describe('localDB', () => {
     beforeEach(async () => {
       await localDB.init();
     });

     it('should save and retrieve building', async () => {
       const building = {
         id: '123',
         name: 'Test Building',
         address: '123 Main St'
       };

       await localDB.put('buildings', building);
       const retrieved = await localDB.get('buildings', '123');

       expect(retrieved).toEqual(building);
     });

     it('should queue offline modifications', async () => {
       await localDB.addToSyncQueue('create', 'buildings', {
         name: 'New Building'
       });

       const queue = await localDB.getSyncQueue();
       expect(queue).toHaveLength(1);
       expect(queue[0].action).toBe('create');
       expect(queue[0].synced).toBe(false);
     });
   });

Sécurité
--------

**⚠️ Données Non Chiffrées** :

IndexedDB stocke données en clair sur l'appareil.

**Recommandations** :

- ❌ Ne pas stocker mots de passe
- ❌ Ne pas stocker tokens JWT long terme
- ⚠️ Chiffrer données GDPR (emails, téléphones)
- ✅ Vider données au logout

.. code-block:: typescript

   // Logout
   async function logout() {
     await localDB.clear('users');
     await localDB.clear('buildings');
     await localDB.clear('owners');
     await localDB.clear('units');
     await localDB.clear('expenses');
     await localDB.clear('sync_queue');
     localStorage.removeItem('koprogo_token');
   }

GDPR Compliance
---------------

**Droit à l'Effacement** :

.. code-block:: typescript

   async function deleteUserData(userId: string) {
     // Supprimer toutes les données locales
     indexedDB.deleteDatabase('koprogo_db');

     // Appeler API backend
     await api.delete(`/users/${userId}/gdpr-delete`);
   }

**Droit à la Portabilité** :

.. code-block:: typescript

   async function exportUserData() {
     const buildings = await localDB.getBuildings();
     const owners = await localDB.getOwners();
     const units = await localDB.getUnits();
     const expenses = await localDB.getExpenses();

     const data = { buildings, owners, units, expenses };
     const blob = new Blob([JSON.stringify(data, null, 2)], {
       type: 'application/json'
     });

     const url = URL.createObjectURL(blob);
     const link = document.createElement('a');
     link.href = url;
     link.download = 'koprogo-data.json';
     link.click();
   }

Références
----------

- Sync Service : ``frontend/src/lib/sync.ts``
- Types : ``frontend/src/lib/types.ts``
- API Client : ``frontend/src/lib/api.ts``
- MDN IndexedDB : https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
