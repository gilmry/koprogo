Stores - État Global Svelte
============================

Gestion de l'état global partagé entre composants via Svelte stores.

**Localisation** : ``frontend/src/stores/``

Vue d'ensemble
--------------

**Svelte Stores** : Système réactif d'état global.

**Types de Stores** :

- **Writable** : Lecture/écriture (``writable()``)
- **Readable** : Lecture seule (``readable()``)
- **Derived** : Dérivé d'autres stores (``derived()``)

**Avantages** :

- ✅ État partagé sans prop drilling
- ✅ Réactivité automatique (``$store``)
- ✅ Persistance localStorage facile
- ✅ Type-safe avec TypeScript

authStore
---------

**Localisation** : ``frontend/src/stores/auth.ts``

Store d'authentification et profil utilisateur.

Structure
^^^^^^^^^

.. code-block:: typescript

   import { writable } from 'svelte/store';
   import type { User } from '../lib/types';

   interface AuthState {
     user: User | null;
     token: string | null;
     isLoggedIn: boolean;
   }

   function createAuthStore() {
     const { subscribe, set, update } = writable<AuthState>({
       user: null,
       token: null,
       isLoggedIn: false
     });

     return {
       subscribe,
       setUser: (user: User) => update(state => ({
         ...state,
         user,
         isLoggedIn: true
       })),
       setToken: (token: string) => update(state => ({
         ...state,
         token
       })),
       logout: () => set({
         user: null,
         token: null,
         isLoggedIn: false
       }),
       init: () => {
         const token = localStorage.getItem('koprogo_token');
         if (token) {
           // Fetch user profile from API
           // ...
         }
       }
     };
   }

   export const authStore = createAuthStore();

Méthodes
^^^^^^^^

**setUser(user)** :

Définit l'utilisateur connecté.

.. code-block:: typescript

   authStore.setUser({
     id: '123',
     email: 'user@example.com',
     firstName: 'John',
     lastName: 'Doe',
     role: UserRole.SYNDIC
   });

**setToken(token)** :

Définit le JWT token.

.. code-block:: typescript

   authStore.setToken('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...');

**logout()** :

Déconnecte l'utilisateur.

.. code-block:: typescript

   authStore.logout();
   localStorage.removeItem('koprogo_token');
   window.location.href = '/login';

**init()** :

Initialise le store depuis localStorage (au chargement app).

.. code-block:: typescript

   // Component racine ou Layout
   onMount(() => {
     authStore.init();
   });

Utilisation dans Composants
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Lecture Réactive** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';

     $: user = $authStore.user;
     $: isLoggedIn = $authStore.isLoggedIn;
   </script>

   {#if isLoggedIn}
     <p>Bienvenue {user.firstName} !</p>
   {:else}
     <a href="/login">Se connecter</a>
   {/if}

**Modification** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';

     async function handleLogin(email: string, password: string) {
       const response = await api.post('/auth/login', { email, password });

       authStore.setToken(response.token);
       authStore.setUser(response.user);

       localStorage.setItem('koprogo_token', response.token);
     }
   </script>

**Vérification Permissions** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';
     import { hasPermission, UserRole } from '../lib/types';

     $: user = $authStore.user;
     $: isSyndic = hasPermission(user, UserRole.SYNDIC);
   </script>

   {#if isSyndic}
     <button on:click={createBuilding}>Créer Immeuble</button>
   {/if}

Persistance localStorage
^^^^^^^^^^^^^^^^^^^^^^^^^

**Sauvegarder automatiquement** :

.. code-block:: typescript

   authStore.subscribe(state => {
     if (state.token) {
       localStorage.setItem('koprogo_token', state.token);
     }
     if (state.user) {
       localStorage.setItem('koprogo_user', JSON.stringify(state.user));
     }
   });

**Restaurer au chargement** :

.. code-block:: typescript

   function init() {
     const token = localStorage.getItem('koprogo_token');
     const userJson = localStorage.getItem('koprogo_user');

     if (token && userJson) {
       const user = JSON.parse(userJson);
       authStore.setToken(token);
       authStore.setUser(user);
     }
   }

Autres Stores (Futurs)
----------------------

preferencesStore
^^^^^^^^^^^^^^^^

Préférences utilisateur (langue, thème, notifications).

.. code-block:: typescript

   // stores/preferences.ts
   import { writable } from 'svelte/store';

   interface PreferencesState {
     locale: string;
     theme: 'light' | 'dark';
     notifications: boolean;
   }

   const defaultPreferences: PreferencesState = {
     locale: 'nl',
     theme: 'light',
     notifications: true
   };

   function createPreferencesStore() {
     const { subscribe, set, update } = writable<PreferencesState>(
       defaultPreferences
     );

     return {
       subscribe,
       setLocale: (locale: string) => update(state => ({ ...state, locale })),
       setTheme: (theme: 'light' | 'dark') => update(state => ({ ...state, theme })),
       toggleNotifications: () => update(state => ({
         ...state,
         notifications: !state.notifications
       })),
       load: () => {
         const saved = localStorage.getItem('koprogo_preferences');
         if (saved) {
           set(JSON.parse(saved));
         }
       },
       reset: () => set(defaultPreferences)
     };
   }

   export const preferencesStore = createPreferencesStore();

**Utilisation** :

.. code-block:: svelte

   <script lang="ts">
     import { preferencesStore } from '../stores/preferences';
     import { locale } from 'svelte-i18n';

     $: $locale = $preferencesStore.locale;
   </script>

   <button on:click={() => preferencesStore.setTheme('dark')}>
     Mode sombre
   </button>

notificationsStore
^^^^^^^^^^^^^^^^^^

Notifications toast (succès, erreurs, warnings).

.. code-block:: typescript

   // stores/notifications.ts
   import { writable } from 'svelte/store';

   interface Notification {
     id: string;
     type: 'success' | 'error' | 'warning' | 'info';
     message: string;
     timeout?: number;
   }

   function createNotificationsStore() {
     const { subscribe, update } = writable<Notification[]>([]);

     return {
       subscribe,
       add: (notification: Omit<Notification, 'id'>) => {
         const id = `notif-${Date.now()}`;
         const fullNotification = { id, ...notification };

         update(notifications => [...notifications, fullNotification]);

         if (notification.timeout !== 0) {
           setTimeout(() => {
             update(notifications =>
               notifications.filter(n => n.id !== id)
             );
           }, notification.timeout || 5000);
         }
       },
       remove: (id: string) => {
         update(notifications =>
           notifications.filter(n => n.id !== id)
         );
       },
       clear: () => {
         update(() => []);
       }
     };
   }

   export const notificationsStore = createNotificationsStore();

**Utilisation** :

.. code-block:: svelte

   <script lang="ts">
     import { notificationsStore } from '../stores/notifications';

     async function saveBuilding() {
       try {
         await api.post('/buildings', data);
         notificationsStore.add({
           type: 'success',
           message: 'Immeuble créé avec succès'
         });
       } catch (error) {
         notificationsStore.add({
           type: 'error',
           message: 'Erreur lors de la création'
         });
       }
     }
   </script>

**Component Toast** :

.. code-block:: svelte

   <script lang="ts">
     import { notificationsStore } from '../stores/notifications';
     import { fade } from 'svelte/transition';
   </script>

   <div class="toast-container">
     {#each $notificationsStore as notification (notification.id)}
       <div
         class="toast toast-{notification.type}"
         transition:fade
         on:click={() => notificationsStore.remove(notification.id)}
       >
         {notification.message}
       </div>
     {/each}
   </div>

Derived Stores
--------------

Stores dérivés calculés à partir d'autres stores.

**Exemple** :

.. code-block:: typescript

   import { derived } from 'svelte/store';
   import { authStore } from './auth';
   import { UserRole } from '../lib/types';

   export const isAdmin = derived(
     authStore,
     $authStore => $authStore.user?.role === UserRole.SUPERADMIN
   );

   export const isSyndic = derived(
     authStore,
     $authStore => $authStore.user?.role === UserRole.SYNDIC
   );

**Utilisation** :

.. code-block:: svelte

   <script lang="ts">
     import { isAdmin } from '../stores/auth';
   </script>

   {#if $isAdmin}
     <a href="/admin">Panneau Admin</a>
   {/if}

Custom Stores
-------------

Stores personnalisés avec logique métier.

**Exemple** : Store de cache API

.. code-block:: typescript

   // stores/cache.ts
   import { writable } from 'svelte/store';
   import type { Building } from '../lib/types';

   interface CacheState {
     buildings: Building[];
     buildingsLastFetch: number | null;
   }

   function createCacheStore() {
     const { subscribe, set, update } = writable<CacheState>({
       buildings: [],
       buildingsLastFetch: null
     });

     return {
       subscribe,
       setBuildings: (buildings: Building[]) => update(state => ({
         ...state,
         buildings,
         buildingsLastFetch: Date.now()
       })),
       shouldRefreshBuildings: (maxAge: number = 5 * 60 * 1000) => {
         let shouldRefresh = true;
         subscribe(state => {
           if (state.buildingsLastFetch) {
             shouldRefresh = (Date.now() - state.buildingsLastFetch) > maxAge;
           }
         })();
         return shouldRefresh;
       }
     };
   }

   export const cacheStore = createCacheStore();

Tests Stores
------------

.. code-block:: typescript

   // tests/unit/auth.store.test.ts
   import { describe, it, expect, beforeEach } from 'vitest';
   import { get } from 'svelte/store';
   import { authStore } from '../src/stores/auth';
   import { UserRole } from '../src/lib/types';

   describe('authStore', () => {
     beforeEach(() => {
       authStore.logout();
     });

     it('should set user', () => {
       const user = {
         id: '123',
         email: 'test@example.com',
         firstName: 'John',
         lastName: 'Doe',
         role: UserRole.SYNDIC
       };

       authStore.setUser(user);

       const state = get(authStore);
       expect(state.user).toEqual(user);
       expect(state.isLoggedIn).toBe(true);
     });

     it('should logout', () => {
       authStore.setUser({ id: '123', ... });
       authStore.logout();

       const state = get(authStore);
       expect(state.user).toBeNull();
       expect(state.isLoggedIn).toBe(false);
     });
   });

Debugging Stores
----------------

**Dev Tools Console** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';

     authStore.subscribe(state => {
       console.log('Auth State Changed:', state);
     });
   </script>

**Svelte DevTools Extension** :

Chrome/Firefox extension pour inspecter stores en temps réel.

Bonnes Pratiques
----------------

1. **Un Store = Une Responsabilité** :

   Éviter stores fourre-tout.

2. **Types TypeScript Stricts** :

   .. code-block:: typescript

      interface AuthState { ... }
      const store = writable<AuthState>({ ... });

3. **Persistance Sélective** :

   Ne pas tout persister dans localStorage (sensibilité GDPR).

4. **Cleanup au Logout** :

   Vider tous les stores sensibles.

5. **Subscribe dans onMount** :

   Éviter memory leaks.

   .. code-block:: svelte

      onMount(() => {
        const unsubscribe = authStore.subscribe(state => { ... });
        return unsubscribe;  // Cleanup
      });

Références
----------

- Auth : ``frontend/src/stores/auth.ts``
- Components : ``frontend/src/components/``
- Lib Types : ``frontend/src/lib/types.ts``
- Svelte Stores : https://svelte.dev/docs/svelte-store
