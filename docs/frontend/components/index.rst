Components - Composants Svelte Réutilisables
=============================================

Vue d'ensemble des composants Svelte interactifs de l'application.

**Localisation** : ``frontend/src/components/``

Organisation
------------

.. code-block:: text

   components/
   ├── dashboards/           # Dashboards spécifiques par rôle
   │   ├── AdminDashboard.svelte
   │   ├── SyndicDashboard.svelte
   │   ├── AccountantDashboard.svelte
   │   └── OwnerDashboard.svelte
   ├── admin/                # Composants admin
   │   └── SeedManager.svelte
   ├── BuildingList.svelte   # Liste immeubles avec pagination
   ├── OwnerList.svelte      # Liste copropriétaires
   ├── UnitList.svelte       # Liste lots
   ├── ExpenseList.svelte    # Liste charges
   ├── MeetingList.svelte    # Liste assemblées
   ├── DocumentList.svelte   # Liste documents
   ├── OrganizationList.svelte  # Liste organisations (superadmin)
   ├── UserListAdmin.svelte  # Liste utilisateurs (admin)
   ├── LoginForm.svelte      # Formulaire authentification
   ├── Navigation.svelte     # Navigation principale
   ├── Pagination.svelte     # Composant pagination réutilisable
   ├── SyncStatus.svelte     # Indicateur online/offline
   └── LanguageSelector.svelte  # Sélecteur de langue

Catégories de Composants
-------------------------

Dashboards (par Rôle)
^^^^^^^^^^^^^^^^^^^^^

**AdminDashboard.svelte**

Dashboard pour **SUPERADMIN** : gestion organisations, utilisateurs, abonnements.

**Features** :

- Stats organisations actives
- Liste utilisateurs récents
- Gestion abonnements (cloud vs self-hosted)
- Seed manager (données de test)

**SyndicDashboard.svelte**

Dashboard pour **SYNDIC** : gestion quotidienne copropriétés.

**Features** :

- Stats immeubles gérés, copropriétaires, charges
- Tâches urgentes (réparations, convocations AG)
- Actions rapides (immeubles, owners, charges, assemblées)
- Prochaine assemblée générale

**Exemple Code** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../../stores/auth';
     $: user = $authStore.user;
   </script>

   <div>
     <h1>Bienvenue, {user?.firstName} 👋</h1>
     <div class="grid grid-cols-4 gap-6">
       <!-- Stats Cards -->
       <div class="bg-white shadow p-6">
         <span>Immeubles gérés</span>
         <p class="text-3xl">8</p>
       </div>
     </div>
   </div>

**AccountantDashboard.svelte**

Dashboard pour **ACCOUNTANT** : consultation comptable, rapports financiers.

**Features** :

- Vue consolidée charges toutes copropriétés
- Rapports financiers (revenus, dépenses, balance)
- Export comptable (PDF, Excel)

**OwnerDashboard.svelte**

Dashboard pour **OWNER** : consultation personnelle.

**Features** :

- Mes lots (appartements, parkings, caves)
- Mes charges à payer
- Mes documents (PCN, PV assemblées)
- Coordonnées syndic

Composants Listes
^^^^^^^^^^^^^^^^^

**BuildingList.svelte**

Liste immeubles avec pagination, création inline.

**Features** :

- ✅ Pagination (20 items par défaut)
- ✅ Formulaire création inline
- ✅ Affichage cartes (nom, adresse, nombre lots)
- ✅ Authentification JWT automatique
- ✅ Gestion erreurs

**Props** :

Aucune prop, composant standalone.

**Exemple** :

.. code-block:: svelte

   <script lang="ts">
     import BuildingList from '../components/BuildingList.svelte';
   </script>

   <BuildingList />

**OwnerList.svelte**

Liste copropriétaires avec recherche.

**Features** :

- Liste complète copropriétaires
- Recherche par nom, email
- Affichage lots possédés
- Coordonnées contact (GDPR protected)

**UnitList.svelte**

Liste lots d'un immeuble.

**Features** :

- Filtre par building_id
- Affichage type (Apartment, Parking, Storage)
- Quote-part millièmes
- Assignation copropriétaire

**ExpenseList.svelte**

Liste charges avec filtres.

**Features** :

- Filtre par immeuble
- Filtre par statut (Pending, Paid, Overdue)
- Catégories (Maintenance, Repair, Insurance, Utilities, Management)
- Marquage "Payé"
- Total charges en attente

**MeetingList.svelte**

Liste assemblées générales.

**Features** :

- Filtres par immeuble
- Statuts (Scheduled, Completed, Cancelled)
- Téléchargement PV (PDF)
- Ordre du jour

**DocumentList.svelte**

Liste documents partagés.

**Features** :

- Filtres par type (PCN, Règlement, Contrat, Facture, Other)
- Upload documents
- Téléchargement
- Prévisualisation PDF inline (iframe)

Composants Utilitaires
^^^^^^^^^^^^^^^^^^^^^^^

**Pagination.svelte**

Composant pagination réutilisable pour toutes les listes.

**Props** :

.. code-block:: typescript

   interface Props {
     currentPage: number;
     totalPages: number;
     totalItems: number;
     perPage: number;
     onPageChange: (page: number) => void;
   }

**Exemple** :

.. code-block:: svelte

   <Pagination
     currentPage={currentPage}
     totalPages={totalPages}
     totalItems={totalItems}
     perPage={perPage}
     onPageChange={handlePageChange}
   />

**Rendu** :

.. code-block:: text

   [<] [1] [2] [3] ... [10] [>]
   Affichage 21-40 sur 200 résultats

**Navigation.svelte**

Navigation principale avec détection rôle utilisateur.

**Features** :

- Menu adapté au rôle (superadmin, syndic, accountant, owner)
- Indicateur utilisateur connecté
- Bouton logout
- Sélecteur de langue
- Badge notifications (future)

**Exemple** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';
     import { UserRole } from '../lib/types';

     $: user = $authStore.user;
     $: isSyndic = user?.role === UserRole.SYNDIC;
   </script>

   <nav>
     <a href="/dashboard">Dashboard</a>
     {#if isSyndic}
       <a href="/buildings">Immeubles</a>
       <a href="/owners">Copropriétaires</a>
     {/if}
   </nav>

**SyncStatus.svelte**

Indicateur statut connexion online/offline.

**Features** :

- 🟢 En ligne / 🔴 Hors ligne
- Bouton synchronisation manuelle
- Badge modifications en attente

**Exemple** :

.. code-block:: svelte

   <script lang="ts">
     import { syncService } from '../lib/sync';
     import { localDB } from '../lib/db';
     import { onMount } from 'svelte';

     let isOnline = syncService.getOnlineStatus();
     let pendingCount = 0;

     async function updateStatus() {
       isOnline = syncService.getOnlineStatus();
       const queue = await localDB.getSyncQueue();
       pendingCount = queue.filter(item => !item.synced).length;
     }

     onMount(() => {
       const interval = setInterval(updateStatus, 1000);
       return () => clearInterval(interval);
     });
   </script>

   <div class="sync-status">
     {#if isOnline}
       <span class="text-green-500">🟢 En ligne</span>
     {:else}
       <span class="text-orange-500">🔴 Hors ligne</span>
     {/if}

     {#if pendingCount > 0}
       <span class="badge">{pendingCount} en attente</span>
     {/if}
   </div>

**LanguageSelector.svelte**

Sélecteur de langue (nl, fr, de, en).

**Features** :

- Dropdown avec drapeaux
- Persistance localStorage
- Mise à jour dynamique traductions

**Exemple** :

.. code-block:: svelte

   <script lang="ts">
     import { locale } from 'svelte-i18n';
     import { languages } from '../lib/i18n';

     function changeLanguage(code: string) {
       $locale = code;
       localStorage.setItem('koprogo_locale', code);
     }
   </script>

   <select bind:value={$locale} on:change={(e) => changeLanguage(e.target.value)}>
     {#each languages as lang}
       <option value={lang.code}>
         {lang.flag} {lang.name}
       </option>
     {/each}
   </select>

Composants Formulaires
^^^^^^^^^^^^^^^^^^^^^^^

**LoginForm.svelte**

Formulaire authentification JWT.

**Features** :

- Email + Password
- Gestion erreurs (401, 500)
- Stockage token localStorage
- Redirection après login
- Initialisation SyncService

**Exemple** :

.. code-block:: svelte

   <script lang="ts">
     import { api } from '../lib/api';
     import { syncService } from '../lib/sync';
     import { authStore } from '../stores/auth';

     let email = '';
     let password = '';
     let error = '';

     async function handleLogin() {
       try {
         const response = await api.post('/auth/login', {
           email,
           password
         });

         const { token, user } = response;

         localStorage.setItem('koprogo_token', token);
         authStore.setUser(user);

         await syncService.initialize(token);

         window.location.href = '/dashboard';
       } catch (e) {
         error = e instanceof Error ? e.message : 'Erreur authentification';
       }
     }
   </script>

   <form on:submit|preventDefault={handleLogin}>
     <input type="email" bind:value={email} required />
     <input type="password" bind:value={password} required />
     <button type="submit">Se connecter</button>
     {#if error}<p class="error">{error}</p>{/if}
   </form>

Composants Admin
^^^^^^^^^^^^^^^^

**SeedManager.svelte**

Composant génération données de test (dev/staging).

**Features** :

- Seed organisations + utilisateurs + immeubles
- Seed complet (all entities)
- Reset database
- Logs generation

**⚠️ Seulement pour environnements non-production !**

**OrganizationList.svelte**

Liste organisations (multi-tenant).

**Features** :

- CRUD organisations (superadmin only)
- Stats utilisateurs par organisation
- Désactivation/Activation compte

**UserListAdmin.svelte**

Liste utilisateurs plateforme.

**Features** :

- CRUD utilisateurs (superadmin only)
- Changement rôle
- Reset password
- Blocage compte

Patterns d'Utilisation
-----------------------

Hydration Astro
^^^^^^^^^^^^^^^

Dans les pages Astro, charger composants Svelte avec directives client:

.. code-block:: astro

   ---
   import BuildingList from '../components/BuildingList.svelte';
   ---
   <Layout>
     <!-- client:load : Hydrate immédiatement -->
     <BuildingList client:load />

     <!-- client:idle : Hydrate après chargement initial -->
     <SyncStatus client:idle />

     <!-- client:visible : Hydrate quand visible viewport -->
     <ExpenseList client:visible />
   </Layout>

Communication Parent-Enfant
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Props Down** :

.. code-block:: svelte

   <!-- Parent -->
   <BuildingCard building={selectedBuilding} />

   <!-- Enfant -->
   <script lang="ts">
     export let building: Building;
   </script>

**Events Up** :

.. code-block:: svelte

   <!-- Enfant -->
   <script lang="ts">
     import { createEventDispatcher } from 'svelte';
     const dispatch = createEventDispatcher();

     function handleClick() {
       dispatch('select', { building });
     }
   </script>

   <!-- Parent -->
   <BuildingCard on:select={handleSelect} />

Stores Partagés
^^^^^^^^^^^^^^^

Pour état global (auth, preferences) :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';

     $: user = $authStore.user;
     $: isLoggedIn = $authStore.isLoggedIn;
   </script>

   {#if isLoggedIn}
     <p>Bienvenue {user.firstName}</p>
   {/if}

Styling avec Tailwind CSS
--------------------------

Tous les composants utilisent Tailwind CSS :

.. code-block:: svelte

   <div class="bg-white rounded-lg shadow p-6 hover:shadow-md transition">
     <h3 class="text-lg font-semibold text-gray-900">{building.name}</h3>
     <p class="text-gray-600 text-sm">📍 {building.address}</p>
   </div>

**Classes Personnalisées** :

.. code-block:: css

   /* frontend/src/styles/global.css */
   .btn-primary {
     @apply bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700;
   }

Tests Composants
----------------

.. code-block:: typescript

   // tests/unit/BuildingList.test.ts
   import { render, screen } from '@testing-library/svelte';
   import { vi } from 'vitest';
   import BuildingList from '../src/components/BuildingList.svelte';

   vi.mock('../src/lib/api', () => ({
     api: {
       get: vi.fn(() => Promise.resolve({
         data: [{ id: '1', name: 'Test Building' }],
         pagination: { current_page: 1, total_pages: 1 }
       }))
     }
   }));

   describe('BuildingList', () => {
     it('should render buildings', async () => {
       render(BuildingList);

       await screen.findByText('Test Building');
       expect(screen.getByText('Test Building')).toBeInTheDocument();
     });
   });

Accessibilité (a11y)
--------------------

Bonnes pratiques :

.. code-block:: svelte

   <!-- Labels pour inputs -->
   <label for="building-name">Nom de l'immeuble</label>
   <input id="building-name" type="text" />

   <!-- Attributs ARIA -->
   <button aria-label="Fermer" on:click={close}>
     <span aria-hidden="true">×</span>
   </button>

   <!-- Navigation clavier -->
   <div role="menu" on:keydown={handleKeydown}>
     <button role="menuitem">Option 1</button>
   </div>

Performance
-----------

**Lazy Loading** :

.. code-block:: svelte

   <script lang="ts">
     import { onMount } from 'svelte';

     let HeavyComponent;

     onMount(async () => {
       HeavyComponent = (await import('./HeavyComponent.svelte')).default;
     });
   </script>

   {#if HeavyComponent}
     <svelte:component this={HeavyComponent} />
   {/if}

**Virtual Scrolling** :

Pour listes > 1000 items, utiliser ``svelte-virtual-list``.

Références
----------

- Pages Astro : ``frontend/src/pages/``
- Stores : ``frontend/src/stores/``
- Lib : ``frontend/src/lib/``
- Svelte Docs : https://svelte.dev/docs
- Tailwind CSS : https://tailwindcss.com/docs
