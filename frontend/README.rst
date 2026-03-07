==================
KoproGo Frontend
==================

:Stack: Astro 4.x + Svelte 4.x + Tailwind CSS 3.x
:Architecture: Islands Architecture (SSG + Hydration selective)

Structure du projet
===================

.. code-block:: text

   frontend/
   |- src/
   |  |- components/       # Composants Svelte interactifs (Islands)
   |  |  |- ui/            # Composants UI reutilisables (Modal, etc.)
   |  |  |- bookings/      # Reservations de ressources
   |  |  |- sharing/       # Partage d'objets
   |  |  |- inspections/   # Inspections techniques
   |  |  |- work-reports/  # Rapports de travaux
   |  |  |- polls/         # Sondages
   |  |  |- energy-campaigns/  # Achats groupes energie
   |  |  '- local-exchanges/   # SEL (echanges locaux)
   |  |- layouts/          # Layouts Astro
   |  |- pages/            # Pages Astro (routes)
   |  |  |- admin/         # Pages administration
   |  |  |- owner/         # Espace coproprietaire
   |  |  '- settings/      # Parametres utilisateur
   |  |- lib/
   |  |  |- api/           # Modules API (bookings, tickets, etc.)
   |  |  |- api.ts         # Client HTTP de base (apiFetch)
   |  |  |- i18n.ts        # Internationalisation (FR/NL/DE/EN)
   |  |  |- types.ts       # Types TypeScript partages
   |  |  '- config.ts      # Configuration runtime
   |  |- stores/           # Stores Svelte
   |  |  |- auth.ts        # Authentification + multi-role
   |  |  |- toast.ts       # Notifications toast
   |  |  '- notifications.ts  # Notifications temps reel
   |  |- locales/          # Fichiers de traduction JSON
   |  '- styles/           # CSS global + Tailwind
   |- tests/
   |  '- e2e/              # Tests Playwright
   '- public/              # Assets statiques + PWA

Pattern Islands
===============

Les pages Astro sont generees statiquement. Les composants Svelte sont hydrates cote client uniquement la ou l'interactivite est requise :

.. code-block:: javascript

   // Dans une page .astro
   import { mount } from "svelte";
   import MonComposant from "../components/MonComposant.svelte";

   const container = document.getElementById("target");
   mount(MonComposant, { target: container, props: { ... } });

Pattern Modal
=============

.. code-block:: svelte

   <script>
     import Modal from '../ui/Modal.svelte';
     import { toast } from '../../stores/toast';
     import { someApi } from '../../lib/api/feature';

     export let isOpen = false;
     let loading = false;

     async function handleSubmit() {
       loading = true;
       try {
         await someApi.create(formData);
         toast.success("Creation reussie !");
         dispatch("created");
       } catch (e) {
         toast.error(e.message);
       } finally {
         loading = false;
       }
     }
   </script>

Demarrage rapide
================

.. code-block:: bash

   # Prerequis : Node.js 18+
   cp .env.example .env
   npm install

   # Dev server (localhost:3000)
   npm run dev

   # Build production
   npm run build

   # Preview build
   npm run preview

Tests E2E (Playwright)
======================

.. code-block:: bash

   # Installer les navigateurs
   npx playwright install

   # Lancer les tests
   npx playwright test

   # Mode interactif
   npx playwright test --ui

10 fichiers spec couvrant : Login, Buildings, Expenses, Meetings, Tickets, Notifications, GDPR, AdminDashboard, BoardOfDirectors, OwnerDashboard.

Variables d'environnement
=========================

- ``PUBLIC_API_URL`` : URL de l'API backend (ex: ``http://localhost:8080/api/v1``)
