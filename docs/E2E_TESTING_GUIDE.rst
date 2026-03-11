=====================================
🎥 Guide Complet des Tests E2E
=====================================

Ce guide centralise **toutes les informations** sur les tests End-to-End avec Playwright et la génération de vidéos de documentation vivante.

.. contents:: Table des matières
   :local:
   :depth: 2

🎯 Introduction
===============

Les tests E2E de KoproGo testent **toute la stack** :

* ✅ Frontend (Astro + Svelte)  
* ✅ Backend (Rust + Actix-web)
* ✅ Base de données (PostgreSQL)
* ✅ API REST
* ✅ PWA + Mode Offline

Chaque test peut enregistrer une **vidéo** de démonstration : générez-les localement, puis ajoutez les fichiers ``.webm`` dans ``docs/_static/videos/`` pour enrichir la documentation.

🚀 Démarrage Rapide
===================

Installation (une seule fois)
------------------------------

.. code-block:: bash

   # Installer les dépendances frontend
   cd frontend
   npm install

   # Installer Playwright et Chromium
   npx playwright install chromium

Démarrer les services
---------------------

.. code-block:: bash

   # Depuis la racine du projet
   make up

   # Les services démarrent automatiquement via Docker Compose + Traefik
   # Frontend: http://localhost
   # API: http://localhost/api/v1

Lancer les tests
----------------

.. code-block:: bash

   # Tests normaux (rapides)
   cd frontend
   npm run test:e2e

   # OU depuis la racine
   make test-e2e

   # Tests ralentis (pour vidéos plus lisibles) ⭐
   make test-e2e-slow

📹 Enregistrer de Nouveaux Tests
=================================

Méthode 1 : Playwright Codegen (⭐ Recommandé)
-----------------------------------------------

**Enregistrement interactif** - Playwright génère le code automatiquement !

.. code-block:: bash

   # Assurer que l'app tourne
   make up

   # Lancer l'enregistrement
   cd frontend
   npm run codegen

   # OU pour mobile
   npm run codegen:mobile

   # Alternative depuis la racine
   make codegen
   make codegen DEVICE=mobile

**Ce qui se passe :**

1. Un navigateur s'ouvre sur ``http://localhost``
2. Une fenêtre **"Playwright Inspector"** s'ouvre à côté
3. Vous naviguez dans l'app (clic, remplissage de formulaires, etc.)
4. Le code du test apparaît en temps réel dans l'Inspector
5. Vous copiez le code et le collez dans un fichier ``.spec.ts``

**Sauvegarder le test :**

.. code-block:: typescript

   // frontend/tests/e2e/mon-test.spec.ts
   import { test, expect } from '@playwright/test';

   test('Mon scénario de test', async ({ page }) => {
     await page.goto('/login');
     await page.fill('input[type="email"]', 'test@test.com');
     await page.fill('input[type="password"]', 'test123');
     await page.click('button[type="submit"]');
     await expect(page.locator('text=Dashboard')).toBeVisible();
   });

**Lancer le test :**

.. code-block:: bash

   npm run test:e2e -- mon-test.spec.ts

La vidéo sera dans ``frontend/test-results/`` !

Méthode 2 : Écrire le Test Manuellement
----------------------------------------

Si vous préférez écrire le code directement :

.. code-block:: bash

   # Créer le fichier
   nano frontend/tests/e2e/mon-test.spec.ts

   # Écrire le test (voir exemple ci-dessus)

   # Lancer
   npm run test:e2e -- mon-test.spec.ts

🐌 Créer des Vidéos Plus Lisibles
==================================

Pour que les vidéos soient plus faciles à suivre, utilisez le **mode ralenti** :

.. code-block:: bash

   make test-e2e-slow

**Ce qui se passe automatiquement :**

1. ✅ Ajoute ``await page.waitForTimeout(1000)`` après chaque action (click, fill, etc.)
2. ✅ Lance les tests E2E
3. ✅ Génère les vidéos localement (1 seconde entre chaque action = plus lisible !)
4. ✅ Restaure automatiquement la vitesse normale après

**Délai personnalisé :**

.. code-block:: bash

   # 2 secondes entre chaque action
   bash .claude/scripts/slow-down-tests.sh 2000
   cd frontend && npm run test:e2e
   bash .claude/scripts/restore-test-speed.sh

**Restaurer manuellement :**

.. code-block:: bash

   make test-e2e-restore-speed

📚 Ajouter les Vidéos dans la Documentation
===========================================

Une fois les tests exécutés **en local**, synchronisez et versionnez les vidéos :

.. code-block:: bash

   # Copie les vidéos + génère la page RST automatiquement
   make docs-sync-videos

   # Générer la documentation Sphinx
   make docs-sphinx

   # Vérifier le rendu localement
   open docs/_build/html/e2e-videos.html

Les fichiers copiés dans ``docs/_static/videos/`` doivent être **commités** dans Git. Ils seront ensuite publiés automatiquement par le workflow documentation (:file:`.github/workflows/docs.yml`).

Le dossier ``docs/_build/`` reste local (ignoré par Git) : ne le commitez pas.

Les vidéos validées sont listées dans la page :doc:`e2e-videos`.

🎬 Commandes Disponibles
=========================

Commandes npm (depuis ``frontend/``)
-------------------------------------

.. code-block:: bash

   # Enregistrement interactif
   npm run codegen              # Desktop
   npm run codegen:mobile       # iPhone 13

   # Tests
   npm run test:e2e             # Tous les tests (headless)
   PLAYWRIGHT_BASE_URL=http://localhost npm run test:e2e -- AdminDashBoard.improved.spec.ts  # Suite admin
   npm run test:e2e -- mon-test.spec.ts  # Un test spécifique
   npm run test:e2e:ui          # Mode UI (interface graphique)
   npm run test:e2e:headed      # Voir le navigateur
   npm run test:e2e:debug       # Mode debug pas à pas

   # Rapports
   npm run test:e2e:report      # Ouvre le rapport HTML avec vidéos

Commandes make (depuis la racine)
----------------------------------

.. code-block:: bash

   # Tests E2E
   make test-e2e                # Tests normaux (rapides)
   make test-e2e-slow           # Tests ralentis (vidéos lisibles)
   make test-e2e-restore-speed  # Restaurer vitesse normale

   # Documentation
   make docs-sync-videos        # Copier vidéos + générer RST (local)
   make docs-with-videos        # Helper local pour générer vidéos + doc
   make docs-sphinx             # Générer doc Sphinx seule
   make codegen                 # Playwright codegen (DEVICE=mobile pour iPhone 13)

Les cibles ``make test-e2e`` et ``make docs-with-videos`` exportent automatiquement ``PLAYWRIGHT_BASE_URL=http://localhost`` pour cibler l'environnement Traefik local.

📂 Structure des Fichiers
==========================

Tests E2E
---------

.. code-block::

   frontend/tests/e2e/
   ├── config.ts                        # Helper pour construire les endpoints
   └── AdminDashBoard.improved.spec.ts  # Suite complète admin (org/users/buildings)

Vidéos Générées
---------------

.. code-block::

   frontend/test-results/
   ├── admin-dashboard-tour-test-chromium/
   │   ├── video.webm              # ← Vidéo du test
   │   ├── trace.zip               # Trace Playwright
   │   └── test-failed-1.png       # (si échec)
   └── autre-test-chromium/
       └── video.webm

Documentation Vidéos
--------------------

.. code-block::

   docs/_static/videos/
   ├── admin-dashboard-tour.webm
   ├── login-success.webm
   └── *.webm                      # Toutes vos vidéos

   docs/e2e-videos.rst             # Page auto-générée

⚙️ Configuration Playwright
============================

Le fichier ``frontend/playwright.config.ts`` configure :

* **Enregistrement vidéo** : ``video: { mode: 'on', size: { width: 1280, height: 720 } }``
* **Base URL** : ``baseURL: 'http://localhost:3000'``
* **WebServer** : Démarre automatiquement ``npm run dev``
* **Timeouts** : 10s par action, 30s par page
* **Screenshots** : Uniquement en cas d'échec

🐛 Debugging
============

Mode UI (Recommandé)
--------------------

.. code-block:: bash

   cd frontend
   npm run test:e2e:ui

Cela ouvre une interface graphique où vous pouvez :

* ✅ Voir tous vos tests
* ✅ Les lancer un par un
* ✅ Voir les vidéos/screenshots
* ✅ Inspecter chaque étape
* ✅ Voir les timings

Mode Debug
----------

.. code-block:: bash

   npm run test:e2e:debug

Le test s'arrête à chaque étape, vous pouvez :

* Inspecter le DOM
* Exécuter du code dans la console
* Avancer pas à pas

Mode Headed (Voir le navigateur)
---------------------------------

.. code-block:: bash

   npm run test:e2e:headed

Le navigateur s'affiche pendant l'exécution des tests.

🆘 Problèmes Courants
=====================

❌ Les navigateurs ne s'installent pas
---------------------------------------

.. code-block:: bash

   # Sans dépendances système (si pas de sudo)
   npx playwright install chromium

   # Avec dépendances (si sudo disponible)
   npx playwright install chromium --with-deps

❌ L'app n'est pas accessible
------------------------------

.. code-block:: bash

   # Vérifier que les services tournent
   curl http://localhost
   curl http://localhost/api/v1/health

   # Si pas de réponse, démarrer :
   make up

❌ Timeout lors des tests
--------------------------

Augmentez les timeouts dans ``playwright.config.ts`` :

.. code-block:: typescript

   use: {
     actionTimeout: 20000,        // 20s au lieu de 10s
     navigationTimeout: 60000,    // 60s au lieu de 30s
   }

❌ Les vidéos ne sont pas générées
-----------------------------------

Vérifiez dans ``playwright.config.ts`` :

.. code-block:: typescript

   video: {
     mode: 'on',  // Doit être 'on', pas 'retain-on-failure'
   }

❌ "Target page has been closed"
---------------------------------

Votre app redirige trop vite. Ajoutez des attentes :

.. code-block:: typescript

   await page.click('button');
   await page.waitForURL('/dashboard');

📊 Best Practices
=================

1. **Noms de tests explicites**

   .. code-block:: typescript

      // ✅ Bon
      test('Login admin et navigation vers dashboard organisations', ...)

      // ❌ Mauvais
      test('test', ...)

2. **Utiliser les rôles ARIA**

   .. code-block:: typescript

      // ✅ Bon (plus robuste)
      await page.getByRole('button', { name: 'Se connecter' }).click();

      // ❌ Éviter (fragile)
      await page.click('.btn-login');

3. **Attentes explicites**

   .. code-block:: typescript

      // ✅ Bon
      await expect(page.getByText('Dashboard')).toBeVisible();

      // ❌ Éviter
      await page.waitForTimeout(5000);

4. **One test, one scenario**

   Chaque test doit tester UN scénario utilisateur complet.

5. **Vidéos lisibles**

   Utilisez ``make test-e2e-slow`` pour créer des vidéos de documentation.

🔗 Intégration CI/CD
====================

Les vidéos ne sont plus générées dans la CI : elles doivent provenir d'un run local fiable, puis être ajoutées au dépôt.  
Le workflow ``.github/workflows/docs.yml`` se charge ensuite de publier la documentation Sphinx (et toutes les vidéos déjà présentes dans ``docs/_static/videos/``) vers GitHub Pages.

📚 Ressources
=============

* **Documentation Playwright** : https://playwright.dev
* **Page vidéos** : :doc:`e2e-videos`
* **Scripts** : ``.claude/scripts/README.md``
* **Configuration** : ``frontend/playwright.config.ts``
* **Makefile** : :doc:`MAKEFILE_GUIDE`

----

.. raw:: html

   <div style="text-align: center; margin: 2rem 0; color: #666;">
       <p><strong>🤖 Guide maintenu avec Claude Code</strong></p>
       <p>KoproGo ASBL - Tests E2E et Documentation Vivante</p>
   </div>
