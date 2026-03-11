Documentation Frontend
======================

Vue d'ensemble du frontend KoproGo, développé avec Astro (SSG) et Svelte (Islands Architecture).

**Stack Technique** :

- **Framework SSG** : Astro 4.x (Static Site Generation)
- **Composants Interactifs** : Svelte 4.x (Islands Architecture)
- **Styling** : Tailwind CSS 3.x
- **Internationalisation** : svelte-i18n (nl, fr, de, en)
- **Tests E2E** : Playwright
- **Build** : Vite

**Architecture** :

.. code-block:: text

   frontend/src/
   ├── components/      # Composants Svelte réutilisables
   │   ├── dashboards/  # Dashboards spécifiques par rôle
   │   └── admin/       # Composants admin
   ├── layouts/         # Layouts Astro
   ├── lib/             # Bibliothèques et utilitaires
   │   ├── api.ts       # Client API REST
   │   ├── types.ts     # Types TypeScript
   │   ├── i18n.ts      # Configuration i18n
   │   ├── sync.ts      # Service synchronisation offline
   │   └── config.ts    # Configuration runtime
   ├── locales/         # Traductions (nl, fr, de, en)
   ├── pages/           # Pages Astro (routing)
   │   ├── admin/       # Pages admin
   │   ├── owner/       # Pages copropriétaire
   │   ├── accountant/  # Pages comptable
   │   ├── syndic/      # Pages syndic
   │   └── buildings/   # Pages immeubles
   └── stores/          # Stores Svelte (auth, etc.)

**Principe Islands Architecture** :

Les pages Astro génèrent du HTML statique avec des "îles" Svelte hydratées côté client pour l'interactivité (formulaires, pagination, dashboards).

.. code-block:: astro

   ---
   // Layout Astro (statique)
   import BuildingList from '../components/BuildingList.svelte';
   ---
   <Layout>
     <!-- Île interactive Svelte -->
     <BuildingList client:load />
   </Layout>

**Modules** :

.. toctree::
   :maxdepth: 2

   lib/index
   components/index
   pages/index
   layouts/index
   stores/index
   locales/index

**Commandes Développement** :

.. code-block:: bash

   # Dev server (localhost:3000)
   npm run dev

   # Production build
   npm run build

   # Preview production
   npm run preview

   # Format (Prettier)
   npm run format

   # Tests E2E (Playwright)
   npm test
   npm run test:ui      # Interface graphique
   npm run test:debug   # Mode debug

**Configuration Runtime** :

Le frontend utilise un système de configuration runtime via ``window.__ENV__`` pour permettre la configuration au déploiement (Docker, GitOps) :

.. code-block:: javascript

   // public/config.js (chargé dans Layout.astro)
   window.__ENV__ = {
     API_URL: "https://api.koprogo.com/api/v1"
   };

**Priorité des variables** :

1. ``window.__ENV__.API_URL`` (runtime, injecté par Docker/Ansible)
2. ``import.meta.env.PUBLIC_API_URL`` (build-time, .env)
3. ``http://127.0.0.1:8080/api/v1`` (fallback local)

**Internationalisation** :

Le frontend supporte 4 langues avec fallback sur le néerlandais :

- **nl** (Nederlands) - Langue par défaut (60% Belgique)
- **fr** (Français) - 40% Belgique
- **de** (Deutsch) - Communauté germanophone
- **en** (English) - International

Les traductions sont dans ``frontend/src/locales/`` et chargées via ``svelte-i18n``.

**Authentification** :

Le frontend utilise JWT Bearer tokens stockés dans ``localStorage`` :

.. code-block:: typescript

   // Stockage du token
   localStorage.setItem("koprogo_token", token);

   // Headers automatiques dans api.ts
   headers["Authorization"] = `Bearer ${token}`;

**Rôles Utilisateurs** :

- **superadmin** : Administrateur plateforme (gestion organisations)
- **syndic** : Gestionnaire de copropriété (gestion immeubles)
- **accountant** : Comptable (consultation, rapports)
- **owner** : Copropriétaire (consultation uniquement)

**Synchronisation Offline (Progressive Web App)** :

Le service ``sync.ts`` implémente une stratégie offline-first avec IndexedDB :

- Détection automatique online/offline
- Queue de synchronisation pour les modifications
- Fallback automatique sur données locales
- Synchronisation automatique au retour online

**Performance** :

- **SSG** : HTML statique généré au build (0ms de génération côté serveur)
- **Code Splitting** : Chargement lazy des composants Svelte
- **Tailwind CSS** : Purge automatique du CSS non utilisé
- **Vite** : Build ultra-rapide avec HMR

**GDPR & Sécurité** :

- Pas de cookies tiers
- Token JWT httpOnly recommandé (actuellement localStorage)
- Pas de tracking analytics par défaut
- Headers Accept-Language pour i18n côté serveur
