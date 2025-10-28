
🎥 Guide des Tests E2E avec Documentation Vidéo
===============================================

KoproGo utilise Playwright pour les tests End-to-End qui génèrent automatiquement des **vidéos de documentation vivante** !

🎯 Qu'est-ce que c'est ?
------------------------

Les tests E2E testent **toute la stack** :


* ✅ Frontend (Astro + Svelte)
* ✅ Backend (Rust + Actix-web)
* ✅ Base de données (PostgreSQL)
* ✅ API REST
* ✅ PWA + Mode Offline

Chaque test génère une **vidéo** qui montre exactement comment l'application fonctionne !

🚀 Démarrage Rapide
-------------------

1. Installation (une seule fois)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd frontend
   npm install
   npm run test:install  # Installe Playwright + Chromium

2. Démarrer les Services
^^^^^^^^^^^^^^^^^^^^^^^^

**Terminal 1 - Backend:**

.. code-block:: bash

   cd backend
   docker-compose up -d postgres  # Si pas déjà démarré
   cargo run

**Terminal 2 - Frontend (optionnel si test:e2e démarre déjà le serveur):**

.. code-block:: bash

   cd frontend
   npm run dev

3. Lancer les Tests
^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd frontend
   npm run test:e2e  # Exécute tous les tests + génère les vidéos

4. Voir les Vidéos ! 🎬
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   npm run test:e2e:report  # Ouvre le rapport HTML avec vidéos intégrées

📹 Vidéos Générées
------------------

Après chaque exécution, vous trouverez les vidéos dans :

.. code-block::

   frontend/test-results/
   ├── auth-Authentication-Flow-should-login-successfully-chromium/
   │   └── video.webm  ← Vidéo du parcours de login
   ├── pwa-offline-PWA-Capabilities-should-work-offline-chromium/
   │   └── video.webm  ← Vidéo du mode offline
   └── dashboards-Syndic-Dashboard-chromium/
       └── video.webm  ← Vidéo du dashboard

🎬 Commandes Disponibles
------------------------

.. code-block:: bash

   # Mode Headless (CI/CD) - Génère les vidéos
   npm run test:e2e

   # Mode UI - Interface graphique interactive
   npm run test:e2e:ui

   # Mode Headed - Voir le navigateur en action
   npm run test:e2e:headed

   # Mode Debug - Debug pas à pas
   npm run test:e2e:debug

   # Voir le rapport avec vidéos
   npm run test:e2e:report

📝 Tests Disponibles
--------------------

1. Tests d'Authentification (\ ``auth.spec.ts``\ )
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   npx playwright test auth.spec.ts

**Ce qui est testé :**


* ✅ Page de login accessible
* ✅ Login avec appel API backend réel
* ✅ Redirection vers dashboard selon le rôle
* ✅ Gestion d'erreurs (mauvais password)
* ✅ Persistance de session (localStorage + IndexedDB)
* ✅ Logout complet
* ✅ Création de comptes pour chaque rôle

**Vidéo générée :** Parcours complet d'un utilisateur qui se connecte.

2. Tests des Dashboards (\ ``dashboards.spec.ts``\ )
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   npx playwright test dashboards.spec.ts

**Ce qui est testé :**


* ✅ Dashboard Syndic (gestion immeubles)
* ✅ Dashboard Comptable (finances)
* ✅ Dashboard Copropriétaire (infos personnelles)
* ✅ Dashboard SuperAdmin (vue plateforme)
* ✅ Navigation entre sections
* ✅ Permissions par rôle

**Vidéos générées :** Un parcours pour chaque type d'utilisateur.

3. Tests PWA et Offline (\ ``pwa-offline.spec.ts``\ )
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   npx playwright test pwa-offline.spec.ts

**Ce qui est testé :**


* ✅ Manifest.json valide
* ✅ Service Worker enregistré
* ✅ Indicateur online/offline
* ✅ IndexedDB utilisé
* ✅ Mode offline fonctionnel
* ✅ Queue de synchronisation

**Vidéos générées :** Démonstration du mode offline.

🎓 Cas d'Usage des Vidéos
-------------------------

1. Documentation d'Équipe
^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Générer les vidéos
   npm run test:e2e

   # Partager le rapport
   npm run test:e2e:report
   # Envoyer le lien dans Slack/Teams

2. Onboarding Développeurs
^^^^^^^^^^^^^^^^^^^^^^^^^^

Les vidéos montrent **exactement** comment l'application fonctionne :


* Parcours utilisateur complet
* Interactions frontend-backend
* Mode offline en action

3. Présentation Client/Stakeholders
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Exécuter les tests en mode headed pour montrer en direct
   npm run test:e2e:headed

   # Ou partager les vidéos du dernier run
   npm run test:e2e:report

4. Debugging
^^^^^^^^^^^^

Si un test échoue, la vidéo montre **exactement** ce qui s'est passé :

.. code-block:: bash

   npm run test:e2e:report
   # Cliquer sur le test qui a échoué
   # Voir la vidéo + screenshots + traces

🔧 Configuration
----------------

Modifier la qualité vidéo
^^^^^^^^^^^^^^^^^^^^^^^^^

Dans ``frontend/playwright.config.ts`` :

.. code-block:: typescript

   video: {
     mode: 'on',  // Toujours enregistrer
     size: { width: 1920, height: 1080 }  // HD
   }

Garder les vidéos même en cas de succès
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Par défaut, **toutes les vidéos sont gardées** (\ ``mode: 'on'``\ ) pour la documentation.

Pour économiser l'espace :

.. code-block:: typescript

   video: {
     mode: 'retain-on-failure'  // Seulement en cas d'échec
   }

🤖 CI/CD avec GitHub Actions
----------------------------

Le workflow ``.github/workflows/e2e-tests.yml`` :


#. ✅ Lance le backend + PostgreSQL
#. ✅ Exécute tous les tests E2E
#. ✅ Génère les vidéos
#. 📦 Sauvegarde les vidéos comme **artifacts GitHub**
#. 💬 Commente la PR avec lien vers les vidéos

Voir les vidéos dans GitHub Actions
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


#. Aller dans **Actions** tab
#. Sélectionner le workflow run
#. Descendre vers **Artifacts**
#. Télécharger ``test-videos-XXX.zip``

Les vidéos sont gardées **30 jours** !

📊 Rapport HTML Interactif
--------------------------

Le rapport HTML contient :

.. code-block::

   playwright-report/
   ├── index.html          ← Page principale
   ├── data/               ← Données des tests
   └── trace/              ← Traces Playwright

**Contenu du rapport :**


* 🎥 Vidéos de chaque test (embedded)
* 📸 Screenshots à chaque étape
* 📝 Logs de console
* ⏱️ Timeline d'exécution
* 🔍 Traces interactives

.. code-block:: bash

   npm run test:e2e:report  # Ouvre dans le navigateur

🎨 Écrire de Nouveaux Tests
---------------------------

Template de Base
^^^^^^^^^^^^^^^^

.. code-block:: typescript

   import { test, expect } from '@playwright/test';

   test('Mon nouveau test', async ({ page }) => {
     // Se connecter (si besoin)
     await page.goto('/login');
     await page.fill('input[type="email"]', 'test@test.com');
     await page.fill('input[type="password"]', 'test123');
     await page.click('button[type="submit"]');

     // Tester ma fonctionnalité
     await page.click('text=Ma Fonctionnalité');
     await expect(page.locator('text=Succès')).toBeVisible();
   });

Test avec Création d'Utilisateur
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: typescript

   test('Mon test avec nouvel utilisateur', async ({ page }) => {
     // Créer un utilisateur via l'API
     const response = await page.request.post('http://127.0.0.1:8080/api/v1/auth/register', {
       data: {
         email: `user-${Date.now()}@test.com`,
         password: 'test123',
         first_name: 'Test',
         last_name: 'User',
         role: 'syndic'
       }
     });

     const { user } = await response.json();

     // Login avec ce compte
     await page.goto('/login');
     await page.fill('input[type="email"]', user.email);
     await page.fill('input[type="password"]', 'test123');
     await page.click('button[type="submit"]');

     // Faire quelque chose...
   });

La **vidéo sera automatiquement générée** ! 🎥

🐛 Problèmes Courants
---------------------

Backend pas démarré
^^^^^^^^^^^^^^^^^^^

**Erreur :**

.. code-block::

   Error: connect ECONNREFUSED 127.0.0.1:8080

**Solution :**

.. code-block:: bash

   cd backend
   cargo run

Base de données pas migrée
^^^^^^^^^^^^^^^^^^^^^^^^^^

**Erreur :**

.. code-block::

   relation "users" does not exist

**Solution :**

.. code-block:: bash

   cd backend
   sqlx migrate run

Timeout des tests
^^^^^^^^^^^^^^^^^

**Erreur :**

.. code-block::

   Timeout 30000ms exceeded

**Solution :**
Augmenter le timeout dans ``playwright.config.ts`` :

.. code-block:: typescript

   use: {
     navigationTimeout: 60000,  // 60 secondes
   }

Service Worker pas enregistré
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Solution :**
Vérifier que le serveur de dev est démarré et que la PWA est bien configurée.

📚 Ressources
-------------


* `Documentation Playwright <https://playwright.dev>`_
* `Playwright Best Practices <https://playwright.dev/docs/best-practices>`_
* `Test Generator <https://playwright.dev/docs/codegen>`_

Générer des Tests Automatiquement
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   npx playwright codegen http://localhost:3000

Cela ouvre un navigateur et enregistre vos actions en code Playwright !

✨ Workflow Recommandé
----------------------

Développement d'une Nouvelle Fonctionnalité
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


#. 
   **Développer** la fonctionnalité (frontend + backend)

#. 
   **Écrire un test E2E** qui la valide :

   .. code-block:: bash

      # Créer le fichier de test
      touch tests/e2e/ma-feature.spec.ts

#. 
   **Exécuter le test en mode UI** pour le développer :

   .. code-block:: bash

      npm run test:e2e:ui

#. 
   **Générer la vidéo finale** :

   .. code-block:: bash

      npm run test:e2e

#. 
   **Partager la vidéo** avec l'équipe/client :

   .. code-block:: bash

      npm run test:e2e:report

Pull Request
^^^^^^^^^^^^


#. Les tests s'exécutent automatiquement via GitHub Actions
#. Les vidéos sont uploadées comme artifacts
#. Le bot commente la PR avec le lien vers les vidéos
#. Reviewer peut voir exactement comment ça fonctionne ! 🎬

🎉 C'est Tout !
---------------

.. code-block:: bash

   # Quick Start
   cd frontend
   npm run test:install    # Installation (une fois)
   cd ../backend && cargo run &  # Démarrer le backend
   cd ../frontend
   npm run test:e2e        # Lancer les tests
   npm run test:e2e:report # Voir les vidéos !

**Les vidéos sont votre documentation vivante !** 🎥✨

Elles montrent exactement comment l'application fonctionne, remplaçant des heures de documentation écrite par des vidéos claires et actualisées automatiquement.
