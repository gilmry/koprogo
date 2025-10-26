Module lib/ - Bibliothèques et Utilitaires
==========================================

Le module ``lib/`` contient les bibliothèques réutilisables et utilitaires du frontend.

**Fichiers** :

.. toctree::
   :maxdepth: 1

   api
   types
   i18n
   sync
   config
   db

**Vue d'ensemble** :

+----------------+--------------------------------------+-------------------+
| Fichier        | Description                          | Dépendances       |
+================+======================================+===================+
| ``api.ts``     | Client HTTP REST avec JWT            | svelte-i18n       |
+----------------+--------------------------------------+-------------------+
| ``types.ts``   | Interfaces TypeScript (Building,     | Aucune            |
|                | Owner, Unit, Expense, etc.)          |                   |
+----------------+--------------------------------------+-------------------+
| ``i18n.ts``    | Configuration internationalisation   | svelte-i18n       |
+----------------+--------------------------------------+-------------------+
| ``sync.ts``    | Service synchronisation offline      | db.ts, api.ts     |
+----------------+--------------------------------------+-------------------+
| ``config.ts``  | Configuration runtime (API_URL)      | Aucune            |
+----------------+--------------------------------------+-------------------+
| ``db.ts``      | Client IndexedDB (stockage local)    | Aucune            |
+----------------+--------------------------------------+-------------------+

**Flux de Données** :

.. code-block:: text

   config.ts → API_URL
        ↓
   api.ts → apiFetch() → Backend REST API
        ↓
   types.ts → Building, Owner, etc.
        ↓
   sync.ts → SyncService → db.ts (IndexedDB)
        ↓
   Components Svelte

**Principes de Design** :

1. **Séparation des Préoccupations** :

   - ``config.ts`` : Configuration pure
   - ``api.ts`` : Communication HTTP
   - ``sync.ts`` : Logique offline
   - ``types.ts`` : Contrats de données

2. **Runtime Configuration** :

   Configuration injectable au déploiement sans rebuild (GitOps-friendly).

3. **Progressive Enhancement** :

   Fonctionnalités offline optionnelles, dégradation gracieuse.

4. **Type Safety** :

   Types TypeScript stricts pour toutes les entités du domaine.

**Exemple d'Utilisation** :

.. code-block:: typescript

   // Component Svelte
   import { api } from '../lib/api';
   import type { Building } from '../lib/types';

   async function loadBuildings() {
     const response = await api.get<PageResponse<Building>>('/buildings');
     return response.data;
   }

   async function createBuilding(data: Partial<Building>) {
     const building = await api.post<Building>('/buildings', data);
     return building;
   }
