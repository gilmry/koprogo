api.ts - Client API REST
=========================

**Localisation** : ``frontend/src/lib/api.ts``

Client HTTP pour communiquer avec le backend REST API, avec gestion automatique JWT et i18n.

Fonctionnalités
---------------

**1. Configuration API_URL Dynamique**

.. code-block:: typescript

   const API_BASE_URL =
     (typeof window !== "undefined" && (window as any).__ENV__?.API_URL) ||
     import.meta.env.PUBLIC_API_URL ||
     "http://localhost:8080/api/v1";

**Priorité** :

1. ``window.__ENV__.API_URL`` (runtime, injecté par Docker/Ansible)
2. ``import.meta.env.PUBLIC_API_URL`` (build-time, .env)
3. ``http://localhost:8080/api/v1`` (fallback développement)

**2. Headers Automatiques**

.. code-block:: typescript

   function getHeaders(): HeadersInit {
     const token = localStorage.getItem("koprogo_token");

     return {
       "Content-Type": "application/json",
       "Accept-Language": getCurrentLanguage(), // nl, fr, de, en
       "Authorization": token ? `Bearer ${token}` : undefined
     };
   }

**Headers injectés** :

- ``Content-Type: application/json`` : Format JSON
- ``Accept-Language: nl|fr|de|en`` : Langue utilisateur (svelte-i18n)
- ``Authorization: Bearer <token>`` : JWT token (si connecté)

**3. Gestion d'Erreurs**

.. code-block:: typescript

   if (!response.ok) {
     const error = await response.text();
     throw new Error(error || `API Error: ${response.status}`);
   }

Les erreurs sont propagées aux composants Svelte pour affichage utilisateur.

API Publique
------------

apiFetch<T>(endpoint, options)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Fonction de base avec headers automatiques.

**Paramètres** :

- ``endpoint`` (string) : Chemin API (``/buildings``) ou URL complète
- ``options`` (RequestInit) : Options fetch standard

**Retour** : ``Promise<T>``

**Exemple** :

.. code-block:: typescript

   const buildings = await apiFetch<Building[]>('/buildings');

api.get<T>(endpoint, options?)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Requête GET.

**Exemple** :

.. code-block:: typescript

   // Simple GET
   const buildings = await api.get<Building[]>('/buildings');

   // Avec pagination
   const response = await api.get<PageResponse<Building>>(
     '/buildings?page=1&per_page=20'
   );

api.post<T>(endpoint, data?, options?)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Requête POST avec données JSON.

**Exemple** :

.. code-block:: typescript

   const newBuilding = await api.post<Building>('/buildings', {
     name: 'Résidence du Parc',
     address: '123 Rue Example',
     city: 'Bruxelles',
     postal_code: '1000',
     country: 'Belgique',
     total_units: 50
   });

api.put<T>(endpoint, data?, options?)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Requête PUT pour mise à jour.

**Exemple** :

.. code-block:: typescript

   const updated = await api.put<Building>(`/buildings/${id}`, {
     name: 'Nouveau nom'
   });

api.delete<T>(endpoint, options?)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Requête DELETE.

**Exemple** :

.. code-block:: typescript

   await api.delete(`/buildings/${id}`);

api.download(endpoint, filename)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Téléchargement de fichiers (PDF, Excel).

**Exemple** :

.. code-block:: typescript

   // Télécharger rapport PCN PDF
   await api.download(
     `/pcn/export/pdf/${buildingId}`,
     'rapport-pcn.pdf'
   );

   // Télécharger export Excel
   await api.download(
     `/expenses/export/excel?building_id=${buildingId}`,
     'charges.xlsx'
   );

**Implémentation** :

.. code-block:: typescript

   const blob = await response.blob();
   const downloadUrl = window.URL.createObjectURL(blob);
   const link = document.createElement("a");
   link.href = downloadUrl;
   link.download = filename;
   link.click();

Utilisation dans Composants Svelte
-----------------------------------

**Pattern Standard** :

.. code-block:: svelte

   <script lang="ts">
     import { onMount } from 'svelte';
     import { api } from '../lib/api';
     import type { Building } from '../lib/types';

     let buildings: Building[] = [];
     let loading = true;
     let error = '';

     onMount(async () => {
       try {
         const response = await api.get<PageResponse<Building>>('/buildings');
         buildings = response.data;
       } catch (e) {
         error = e instanceof Error ? e.message : 'Erreur API';
       } finally {
         loading = false;
       }
     });
   </script>

**Pattern CRUD Complet** :

.. code-block:: typescript

   // CREATE
   async function createBuilding(data: Partial<Building>) {
     const building = await api.post<Building>('/buildings', data);
     await loadBuildings(); // Refresh
   }

   // READ
   async function loadBuildings() {
     const response = await api.get<PageResponse<Building>>(
       `/buildings?page=${currentPage}&per_page=${perPage}`
     );
     buildings = response.data;
   }

   // UPDATE
   async function updateBuilding(id: string, data: Partial<Building>) {
     const building = await api.put<Building>(`/buildings/${id}`, data);
     await loadBuildings();
   }

   // DELETE
   async function deleteBuilding(id: string) {
     await api.delete(`/buildings/${id}`);
     await loadBuildings();
   }

Gestion de la Langue
---------------------

Le header ``Accept-Language`` est automatiquement synchronisé avec le store ``svelte-i18n`` :

.. code-block:: typescript

   import { get } from "svelte/store";
   import { locale } from "svelte-i18n";

   function getCurrentLanguage(): string {
     const currentLocale = get(locale);
     return currentLocale || "nl"; // Fallback néerlandais
   }

Le backend reçoit la langue et peut renvoyer des erreurs/messages localisés.

Sécurité
--------

**JWT Token** :

- Stocké dans ``localStorage`` avec clé ``koprogo_token``
- Injecté automatiquement dans header ``Authorization: Bearer <token>``
- Accessible côté client (XSS risk, envisager httpOnly cookie)

**CORS** :

Le backend doit configurer CORS pour accepter ``Accept-Language`` et ``Authorization`` :

.. code-block:: rust

   // backend/src/main.rs
   Cors::default()
     .allow_any_origin()
     .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
     .allowed_headers(vec![
       header::AUTHORIZATION,
       header::ACCEPT,
       header::CONTENT_TYPE,
       HeaderName::from_static("accept-language")
     ])

Limitations Connues
-------------------

1. **Token localStorage** :

   Vulnérable XSS. Recommandation : migrer vers httpOnly cookie.

2. **Pas de Retry Logic** :

   Les erreurs réseau ne sont pas retentées automatiquement.

3. **Pas de Request Cancellation** :

   Les requêtes ne peuvent pas être annulées (AbortController non implémenté).

4. **Pas de Rate Limiting Client** :

   Pas de throttling/debouncing des requêtes.

Extensions Futures
------------------

- **AbortController** : Annulation de requêtes (navigation rapide)
- **Retry avec Exponential Backoff** : Résilience réseau
- **Request Deduplication** : Éviter requêtes identiques simultanées
- **Cache HTTP** : Utiliser headers ``Cache-Control``
- **WebSocket Support** : Notifications temps réel
- **GraphQL Support** : Alternative REST pour requêtes complexes

Tests
-----

Le client API doit être testé avec :

.. code-block:: typescript

   // tests/unit/api.test.ts
   import { describe, it, expect, vi } from 'vitest';
   import { api } from '../src/lib/api';

   describe('api.get', () => {
     it('should add JWT header if token exists', async () => {
       localStorage.setItem('koprogo_token', 'test-token');

       global.fetch = vi.fn(() => Promise.resolve({
         ok: true,
         json: () => Promise.resolve({ data: [] })
       }));

       await api.get('/buildings');

       expect(fetch).toHaveBeenCalledWith(
         expect.any(String),
         expect.objectContaining({
           headers: expect.objectContaining({
             'Authorization': 'Bearer test-token'
           })
         })
       );
     });
   });

Références
----------

- Backend API : ``backend/src/infrastructure/web/handlers/``
- Types : ``frontend/src/lib/types.ts``
- Config : ``frontend/src/lib/config.ts``
- Sync Service : ``frontend/src/lib/sync.ts``
