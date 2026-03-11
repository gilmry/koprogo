Pages - Routing Astro
======================

Vue d'ensemble des pages Astro et structure de routing.

**Localisation** : ``frontend/src/pages/``

Routing File-Based
------------------

Astro utilise le routing basé sur fichiers :

.. code-block:: text

   pages/
   ├── index.astro              → /
   ├── login.astro              → /login
   ├── buildings/
   │   └── index.astro          → /buildings
   ├── owners.astro             → /owners
   ├── units.astro              → /units
   ├── expenses.astro           → /expenses
   ├── meetings.astro           → /meetings
   ├── documents.astro          → /documents
   ├── reports.astro            → /reports
   ├── settings.astro           → /settings
   ├── profile.astro            → /profile
   ├── admin/
   │   ├── index.astro          → /admin
   │   ├── users.astro          → /admin/users
   │   ├── organizations.astro  → /admin/organizations
   │   ├── subscriptions.astro  → /admin/subscriptions
   │   └── seed.astro           → /admin/seed
   ├── syndic/
   │   └── index.astro          → /syndic
   ├── accountant/
   │   └── index.astro          → /accountant
   └── owner/
       ├── index.astro          → /owner
       ├── units.astro          → /owner/units
       ├── expenses.astro       → /owner/expenses
       ├── documents.astro      → /owner/documents
       ├── profile.astro        → /owner/profile
       └── contact.astro        → /owner/contact

Structure Pages
---------------

Anatomie Page Astro
^^^^^^^^^^^^^^^^^^^

.. code-block:: astro

   ---
   // --- Section Frontmatter (TypeScript) ---
   import Layout from '../layouts/Layout.astro';
   import BuildingList from '../components/BuildingList.svelte';

   // Protection authentification
   const token = Astro.cookies.get('koprogo_token');
   if (!token) {
     return Astro.redirect('/login');
   }

   // Données SSG (optionnel)
   const title = "Immeubles";
   ---

   <!-- --- Section Template (HTML) --- -->
   <Layout title={title}>
     <main class="container mx-auto px-4 py-8">
       <h1 class="text-3xl font-bold mb-6">Gestion des Immeubles</h1>

       <!-- Île interactive Svelte -->
       <BuildingList client:load />
     </main>
   </Layout>

**Sections** :

1. **Frontmatter** (``---``) : Code TypeScript exécuté au build (SSG) ou requête (SSR)
2. **Template** : HTML statique + îles interactives Svelte

Pages Publiques
---------------

index.astro (/)
^^^^^^^^^^^^^^^

Page d'accueil, redirige vers dashboard si connecté.

.. code-block:: astro

   ---
   const token = Astro.cookies.get('koprogo_token');
   if (token) {
     return Astro.redirect('/dashboard');
   }
   ---
   <Layout title="KoproGo" showNav={false}>
     <div class="hero">
       <h1>KoproGo - Gestion de Copropriété SaaS</h1>
       <a href="/login" class="btn-primary">Se connecter</a>
     </div>
   </Layout>

login.astro (/login)
^^^^^^^^^^^^^^^^^^^^

Page authentification avec formulaire JWT.

.. code-block:: astro

   ---
   import Layout from '../layouts/Layout.astro';
   import LoginForm from '../components/LoginForm.svelte';
   ---
   <Layout title="Connexion" showNav={false}>
     <div class="login-container">
       <LoginForm client:load />
     </div>
   </Layout>

Pages Protégées (Authentification)
-----------------------------------

**Pattern Protection** :

.. code-block:: astro

   ---
   const token = Astro.cookies.get('koprogo_token');
   if (!token) {
     return Astro.redirect('/login');
   }
   ---

**⚠️ Cookies vs localStorage** :

- ``Astro.cookies`` : Uniquement si cookies utilisés (SSR)
- ``localStorage`` : Vérification côté client (Svelte component)

**Meilleure approche** : Middleware Astro

.. code-block:: typescript

   // src/middleware.ts
   export function onRequest({ cookies, redirect }, next) {
     const token = cookies.get('koprogo_token');
     const isPublicRoute = ['/login', '/'].includes(request.url.pathname);

     if (!token && !isPublicRoute) {
       return redirect('/login');
     }

     return next();
   }

Pages par Rôle
--------------

/admin/* (SUPERADMIN)
^^^^^^^^^^^^^^^^^^^^^

**admin/index.astro** : Dashboard admin

.. code-block:: astro

   ---
   import AdminDashboard from '../../components/dashboards/AdminDashboard.svelte';
   // TODO: Vérifier role === SUPERADMIN
   ---
   <Layout title="Admin Dashboard">
     <AdminDashboard client:load />
   </Layout>

**admin/users.astro** : Gestion utilisateurs

.. code-block:: astro

   ---
   import UserListAdmin from '../../components/UserListAdmin.svelte';
   ---
   <Layout title="Gestion Utilisateurs">
     <UserListAdmin client:load />
   </Layout>

**admin/organizations.astro** : Gestion organisations (multi-tenant)

**admin/subscriptions.astro** : Abonnements (cloud vs self-hosted)

**admin/seed.astro** : Génération données de test

/syndic (SYNDIC)
^^^^^^^^^^^^^^^^

**syndic/index.astro** : Dashboard syndic

.. code-block:: astro

   ---
   import SyndicDashboard from '../../components/dashboards/SyndicDashboard.svelte';
   ---
   <Layout title="Dashboard Syndic">
     <SyndicDashboard client:load />
   </Layout>

/accountant (ACCOUNTANT)
^^^^^^^^^^^^^^^^^^^^^^^^

**accountant/index.astro** : Dashboard comptable

.. code-block:: astro

   ---
   import AccountantDashboard from '../../components/dashboards/AccountantDashboard.svelte';
   ---
   <Layout title="Dashboard Comptable">
     <AccountantDashboard client:load />
   </Layout>

/owner/* (OWNER)
^^^^^^^^^^^^^^^^

**owner/index.astro** : Dashboard copropriétaire

.. code-block:: astro

   ---
   import OwnerDashboard from '../../components/dashboards/OwnerDashboard.svelte';
   ---
   <Layout title="Mon Espace Copropriétaire">
     <OwnerDashboard client:load />
   </Layout>

**owner/units.astro** : Mes lots

**owner/expenses.astro** : Mes charges

**owner/documents.astro** : Mes documents

**owner/contact.astro** : Contacter le syndic

Pages Entités (CRUD)
---------------------

buildings/index.astro
^^^^^^^^^^^^^^^^^^^^^

Liste et création immeubles.

.. code-block:: astro

   ---
   import Layout from '../../layouts/Layout.astro';
   import BuildingList from '../../components/BuildingList.svelte';
   ---
   <Layout title="Immeubles">
     <main class="container mx-auto px-4 py-8">
       <h1 class="text-3xl font-bold mb-6">Gestion des Immeubles</h1>
       <BuildingList client:load />
     </main>
   </Layout>

owners.astro
^^^^^^^^^^^^

Liste copropriétaires.

.. code-block:: astro

   ---
   import OwnerList from '../components/OwnerList.svelte';
   ---
   <Layout title="Copropriétaires">
     <OwnerList client:load />
   </Layout>

units.astro
^^^^^^^^^^^

Liste lots.

expenses.astro
^^^^^^^^^^^^^^

Liste charges avec filtres et marquage "Payé".

meetings.astro
^^^^^^^^^^^^^^

Liste assemblées générales avec upload PV.

documents.astro
^^^^^^^^^^^^^^^

Liste documents partagés avec upload.

Pages Utilitaires
-----------------

reports.astro
^^^^^^^^^^^^^

Génération rapports (PCN, financiers).

.. code-block:: astro

   ---
   import Layout from '../layouts/Layout.astro';
   ---
   <Layout title="Rapports">
     <main class="container mx-auto px-4 py-8">
       <h1>Rapports et Exports</h1>

       <section class="card">
         <h2>Précompte de Charge Notariale (PCN)</h2>
         <button class="btn-primary" onclick="downloadPCN()">
           Télécharger PDF
         </button>
       </section>

       <section class="card">
         <h2>Rapports Financiers</h2>
         <button class="btn-primary" onclick="downloadFinancialReport()">
           Télécharger Excel
         </button>
       </section>
     </main>
   </Layout>

   <script>
     import { api } from '../lib/api';

     async function downloadPCN() {
       const buildingId = '...';  // Depuis sélection
       await api.download(`/pcn/export/pdf/${buildingId}`, 'rapport-pcn.pdf');
     }

     async function downloadFinancialReport() {
       await api.download('/reports/financial', 'rapport-financier.xlsx');
     }
   </script>

settings.astro
^^^^^^^^^^^^^^

Paramètres utilisateur (langue, notifications, préférences).

profile.astro
^^^^^^^^^^^^^

Profil utilisateur (nom, email, mot de passe).

Routing Dynamique
-----------------

Pour pages dynamiques (ex: ``/buildings/[id]``), créer :

.. code-block:: text

   pages/
   └── buildings/
       └── [id].astro

.. code-block:: astro

   ---
   // pages/buildings/[id].astro
   import Layout from '../../layouts/Layout.astro';

   const { id } = Astro.params;

   // Fetch building data (SSG)
   const response = await fetch(`${API_URL}/buildings/${id}`);
   const building = await response.json();
   ---
   <Layout title={building.name}>
     <main>
       <h1>{building.name}</h1>
       <p>{building.address}</p>
       <p>{building.total_units} lots</p>
     </main>
   </Layout>

**Static Site Generation (SSG)** : Générer toutes les pages au build

.. code-block:: typescript

   export async function getStaticPaths() {
     const buildings = await fetchAllBuildings();

     return buildings.map(building => ({
       params: { id: building.id },
       props: { building }
     }));
   }

Redirections
------------

**Redirect Simple** :

.. code-block:: astro

   ---
   return Astro.redirect('/login');
   ---

**Redirect Conditionnel** :

.. code-block:: astro

   ---
   const token = Astro.cookies.get('koprogo_token');
   if (!token) {
     return Astro.redirect('/login');
   }

   // Continuer si authentifié
   ---

**Redirect après Action** :

.. code-block:: astro

   ---
   if (Astro.request.method === 'POST') {
     // Traiter formulaire
     await api.post('/buildings', formData);
     return Astro.redirect('/buildings');
   }
   ---

SEO et Meta Tags
----------------

**Page avec SEO** :

.. code-block:: astro

   ---
   import Layout from '../layouts/Layout.astro';

   const title = "Gestion Immeubles - KoproGo";
   const description = "Gérez vos copropriétés avec KoproGo";
   ---
   <Layout {title}>
     <head>
       <meta name="description" content={description} />
       <meta property="og:title" content={title} />
       <meta property="og:description" content={description} />
       <meta property="og:type" content="website" />
     </head>

     <main>...</main>
   </Layout>

**Sitemap** :

.. code-block:: xml

   <!-- public/sitemap.xml -->
   <?xml version="1.0" encoding="UTF-8"?>
   <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
     <url>
       <loc>https://koprogo.com/</loc>
       <lastmod>2025-01-01</lastmod>
       <priority>1.0</priority>
     </url>
     <url>
       <loc>https://koprogo.com/login</loc>
       <priority>0.8</priority>
     </url>
   </urlset>

Gestion Erreurs
---------------

**404.astro** :

.. code-block:: astro

   ---
   import Layout from '../layouts/Layout.astro';
   ---
   <Layout title="Page non trouvée">
     <main class="text-center py-20">
       <h1 class="text-6xl font-bold">404</h1>
       <p class="text-xl text-gray-600 mt-4">Page non trouvée</p>
       <a href="/" class="btn-primary mt-8">Retour à l'accueil</a>
     </main>
   </Layout>

**500.astro** (erreur serveur) :

Créer ``src/pages/500.astro`` pour erreurs serveur.

Tests Pages
-----------

.. code-block:: typescript

   // tests/e2e/login.spec.ts
   import { test, expect } from '@playwright/test';

   test('should login successfully', async ({ page }) => {
     await page.goto('/login');

     await page.fill('input[type="email"]', 'test@example.com');
     await page.fill('input[type="password"]', 'password123');

     await page.click('button[type="submit"]');

     await expect(page).toHaveURL('/dashboard');
   });

   test('should redirect to login if not authenticated', async ({ page }) => {
     await page.goto('/buildings');

     await expect(page).toHaveURL('/login');
   });

Build Output
------------

**Mode SSG (par défaut)** :

.. code-block:: bash

   npm run build

   # Génère:
   dist/
   ├── index.html
   ├── login.html
   ├── buildings/
   │   └── index.html
   └── _astro/
       ├── client.*.js
       └── *.css

**Mode SSR (Server-Side Rendering)** :

.. code-block:: javascript

   // astro.config.mjs
   export default defineConfig({
     output: 'server',  // SSR
     adapter: node()    // Adapter Node.js
   });

Références
----------

- Layouts : ``frontend/src/layouts/``
- Components : ``frontend/src/components/``
- Middleware : ``frontend/src/middleware.ts`` (à créer)
- Astro Routing : https://docs.astro.build/en/core-concepts/routing/
