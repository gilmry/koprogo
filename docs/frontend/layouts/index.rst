Layouts - Templates Astro
===========================

Layouts réutilisables pour structure commune des pages.

**Localisation** : ``frontend/src/layouts/``

Layout.astro
------------

Layout principal de l'application.

**Localisation** : ``frontend/src/layouts/Layout.astro``

Structure
^^^^^^^^^

.. code-block:: astro

   ---
   import '../styles/global.css';
   import '../lib/i18n';
   import Navigation from '../components/Navigation.svelte';

   interface Props {
     title: string;
     showNav?: boolean;
   }

   const { title, showNav = true } = Astro.props;
   ---

   <!doctype html>
   <html lang="nl">
     <head>
       <meta charset="UTF-8" />
       <meta name="description" content="KoproGo - Plateforme SaaS de Gestion de Copropriété" />
       <meta name="viewport" content="width=device-width" />
       <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
       <meta name="generator" content={Astro.generator} />
       <title>{title}</title>
       <!-- Runtime configuration -->
       <script is:inline src="/config.js"></script>
     </head>
     <body class="bg-gray-50 min-h-screen">
       {showNav && <Navigation client:load />}
       <slot />
       <footer class="bg-white mt-12 border-t">
         <div class="container mx-auto px-4 py-6 text-center text-gray-600">
           <p>&copy; 2025 KoproGo - Gestion de Copropriété SaaS</p>
           <p class="text-sm mt-2">
             Architecture Hexagonale · Rust · Actix-web · PostgreSQL · GDPR · ISO 27001
           </p>
         </div>
       </footer>
     </body>
   </html>

Props
^^^^^

.. code-block:: typescript

   interface Props {
     title: string;      // Titre de la page (requis)
     showNav?: boolean;  // Afficher navigation (défaut: true)
   }

Sections
^^^^^^^^

**<head>** :

- Meta charset UTF-8
- Meta description (SEO)
- Meta viewport (responsive)
- Favicon
- Title dynamique
- **config.js** : Configuration runtime (``window.__ENV__``)

**<body>** :

- Navigation conditionnelle (``{showNav && <Navigation />}``)
- **<slot />** : Contenu de la page
- Footer avec mentions légales

Configuration Runtime
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: html

   <script is:inline src="/config.js"></script>

**Attribut is:inline** : Force chargement synchrone avant app Svelte.

**public/config.js** (généré par Ansible/Docker) :

.. code-block:: javascript

   window.__ENV__ = {
     API_URL: "https://api.koprogo.com/api/v1"
   };

Global CSS
^^^^^^^^^^

.. code-block:: astro

   import '../styles/global.css';

**frontend/src/styles/global.css** :

.. code-block:: css

   @tailwind base;
   @tailwind components;
   @tailwind utilities;

   @layer components {
     .btn-primary {
       @apply bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700 transition;
     }

     .card {
       @apply bg-white rounded-lg shadow p-6;
     }
   }

i18n Import
^^^^^^^^^^^

.. code-block:: astro

   import '../lib/i18n';

Initialise svelte-i18n avant chargement composants.

Utilisation dans Pages
-----------------------

**Page Standard** :

.. code-block:: astro

   ---
   import Layout from '../layouts/Layout.astro';
   import BuildingList from '../components/BuildingList.svelte';
   ---
   <Layout title="Immeubles">
     <main class="container mx-auto px-4 py-8">
       <h1 class="text-3xl font-bold mb-6">Gestion des Immeubles</h1>
       <BuildingList client:load />
     </main>
   </Layout>

**Page sans Navigation** (login) :

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

Layouts Spécialisés (Futurs)
-----------------------------

DashboardLayout.astro
^^^^^^^^^^^^^^^^^^^^^

Layout spécifique dashboards avec sidebar.

.. code-block:: astro

   ---
   import Layout from './Layout.astro';
   import Sidebar from '../components/Sidebar.svelte';

   interface Props {
     title: string;
     role: UserRole;
   }

   const { title, role } = Astro.props;
   ---
   <Layout {title}>
     <div class="flex">
       <Sidebar {role} client:load />
       <main class="flex-1 p-8">
         <slot />
       </main>
     </div>
   </Layout>

**Utilisation** :

.. code-block:: astro

   ---
   import DashboardLayout from '../layouts/DashboardLayout.astro';
   ---
   <DashboardLayout title="Dashboard" role="syndic">
     <p>Contenu du dashboard...</p>
   </DashboardLayout>

AdminLayout.astro
^^^^^^^^^^^^^^^^^

Layout admin avec menu latéral administrateur.

PrintLayout.astro
^^^^^^^^^^^^^^^^^

Layout pour impression (rapports PDF).

.. code-block:: astro

   ---
   interface Props {
     title: string;
   }
   const { title } = Astro.props;
   ---
   <!doctype html>
   <html lang="nl">
     <head>
       <meta charset="UTF-8" />
       <title>{title}</title>
       <style>
         @media print {
           body { font-size: 12pt; }
           .no-print { display: none; }
         }
       </style>
     </head>
     <body>
       <slot />
     </body>
   </html>

**Utilisation** :

.. code-block:: astro

   ---
   import PrintLayout from '../layouts/PrintLayout.astro';
   ---
   <PrintLayout title="Rapport PCN">
     <div class="no-print">
       <button onclick="window.print()">Imprimer</button>
     </div>

     <div class="report-content">
       <h1>Précompte de Charge Notariale</h1>
       <!-- Contenu rapport -->
     </div>
   </PrintLayout>

Nested Layouts
--------------

Composer plusieurs layouts.

.. code-block:: astro

   ---
   // layouts/AdminLayout.astro
   import Layout from './Layout.astro';
   ---
   <Layout title="Admin">
     <div class="admin-wrapper">
       <aside class="admin-sidebar">
         <!-- Menu admin -->
       </aside>
       <main class="admin-content">
         <slot />
       </main>
     </div>
   </Layout>

.. code-block:: astro

   ---
   // pages/admin/users.astro
   import AdminLayout from '../../layouts/AdminLayout.astro';
   ---
   <AdminLayout>
     <h1>Gestion Utilisateurs</h1>
     <!-- Contenu -->
   </AdminLayout>

**Résultat** :

.. code-block:: text

   Layout (navigation + footer)
     └─ AdminLayout (sidebar admin)
        └─ Page Content (users)

Slots Nommés
------------

Pour zones multiples personnalisables.

.. code-block:: astro

   ---
   // layouts/DashboardLayout.astro
   interface Props {
     title: string;
   }
   const { title } = Astro.props;
   ---
   <Layout {title}>
     <div class="dashboard-grid">
       <aside class="sidebar">
         <slot name="sidebar" />
       </aside>
       <main class="main-content">
         <slot />
       </main>
       <aside class="widgets">
         <slot name="widgets" />
       </aside>
     </div>
   </Layout>

**Utilisation** :

.. code-block:: astro

   ---
   import DashboardLayout from '../layouts/DashboardLayout.astro';
   ---
   <DashboardLayout title="Dashboard">
     <!-- Slot par défaut (main-content) -->
     <h1>Bienvenue</h1>

     <!-- Slot nommé "sidebar" -->
     <div slot="sidebar">
       <ul>
         <li><a href="/buildings">Immeubles</a></li>
       </ul>
     </div>

     <!-- Slot nommé "widgets" -->
     <div slot="widgets">
       <div class="widget">Stats</div>
     </div>
   </DashboardLayout>

Head Injection
--------------

Injecter meta tags depuis pages.

.. code-block:: astro

   ---
   // layouts/Layout.astro
   interface Props {
     title: string;
     description?: string;
   }
   const { title, description } = Astro.props;
   ---
   <html>
     <head>
       <title>{title}</title>
       {description && <meta name="description" content={description} />}
       <slot name="head" />
     </head>
     <body>
       <slot />
     </body>
   </html>

**Utilisation** :

.. code-block:: astro

   ---
   import Layout from '../layouts/Layout.astro';
   ---
   <Layout title="Immeubles" description="Gérez vos immeubles">
     <head slot="head">
       <meta property="og:image" content="/og-buildings.png" />
       <link rel="canonical" href="https://koprogo.com/buildings" />
     </head>

     <main>...</main>
   </Layout>

Responsive Design
-----------------

Layout adapte automatiquement via Tailwind :

.. code-block:: astro

   <body class="bg-gray-50 min-h-screen">
     <div class="container mx-auto px-4 sm:px-6 lg:px-8">
       {showNav && <Navigation client:load />}
       <slot />
     </div>
   </body>

**Breakpoints Tailwind** :

- ``sm:`` : ≥ 640px (mobile)
- ``md:`` : ≥ 768px (tablet)
- ``lg:`` : ≥ 1024px (desktop)
- ``xl:`` : ≥ 1280px (large desktop)

Dark Mode (Futur)
-----------------

Support thème sombre.

.. code-block:: astro

   ---
   // layouts/Layout.astro
   ---
   <html lang="nl" class="dark">
     <body class="bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
       <slot />
     </body>
   </html>

.. code-block:: css

   /* global.css */
   @layer base {
     :root {
       --color-primary: 59 130 246;  /* blue-500 */
     }

     .dark {
       --color-primary: 96 165 250;  /* blue-400 */
     }
   }

Performance
-----------

**Inlining Critical CSS** :

.. code-block:: astro

   <head>
     <style is:inline>
       /* Critical CSS inline pour First Paint rapide */
       body { margin: 0; font-family: sans-serif; }
     </style>
   </head>

**Preload Fonts** :

.. code-block:: astro

   <head>
     <link rel="preload" href="/fonts/inter.woff2" as="font" type="font/woff2" crossorigin />
   </head>

**Resource Hints** :

.. code-block:: astro

   <head>
     <link rel="dns-prefetch" href="https://api.koprogo.com" />
     <link rel="preconnect" href="https://api.koprogo.com" />
   </head>

Tests Layouts
-------------

.. code-block:: typescript

   // tests/e2e/layout.spec.ts
   import { test, expect } from '@playwright/test';

   test('layout should render navigation', async ({ page }) => {
     await page.goto('/buildings');

     // Vérifier navigation
     await expect(page.locator('nav')).toBeVisible();

     // Vérifier footer
     await expect(page.locator('footer')).toContainText('2025 KoproGo');
   });

   test('layout should hide navigation on login page', async ({ page }) => {
     await page.goto('/login');

     // Navigation cachée
     await expect(page.locator('nav')).not.toBeVisible();
   });

Références
----------

- Pages : ``frontend/src/pages/``
- Components : ``frontend/src/components/``
- Global CSS : ``frontend/src/styles/global.css``
- Astro Layouts : https://docs.astro.build/en/core-concepts/layouts/
