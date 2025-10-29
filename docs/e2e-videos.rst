======================================
🎥 Vidéos Tests E2E (Documentation Vivante)
======================================

.. raw:: html

   <style>
   .video-stats {
       display: flex;
       gap: 1.5rem;
       margin: 2rem 0;
       flex-wrap: wrap;
   }
   .stat-card {
       background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
       color: white;
       padding: 1.5rem 2rem;
       border-radius: 12px;
       box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
       min-width: 150px;
       text-align: center;
   }
   .stat-number {
       font-size: 3rem;
       font-weight: bold;
       line-height: 1;
       margin-bottom: 0.5rem;
   }
   .stat-label {
       font-size: 0.9rem;
       opacity: 0.95;
       text-transform: uppercase;
       letter-spacing: 0.5px;
   }

   .video-section {
       margin: 3rem 0;
   }
   .video-section h2 {
       font-size: 1.8rem;
       color: #1a202c;
       margin-bottom: 1.5rem;
       padding-bottom: 0.75rem;
       border-bottom: 3px solid #667eea;
   }
   .video-grid {
       display: grid;
       grid-template-columns: repeat(auto-fit, minmax(420px, 1fr));
       gap: 2rem;
       margin: 2rem 0;
   }
   .video-card {
       background: #ffffff;
       border-radius: 12px;
       overflow: hidden;
       box-shadow: 0 4px 12px rgba(0,0,0,0.1);
       transition: all 0.3s ease;
       border: 1px solid #e2e8f0;
   }
   .video-card:hover {
       transform: translateY(-6px);
       box-shadow: 0 12px 28px rgba(0,0,0,0.15);
   }
   .video-card video {
       width: 100%;
       height: auto;
       display: block;
       background: #000;
   }
   .video-info {
       padding: 1.25rem;
       background: #f8f9fa;
   }
   .video-title {
       font-weight: 600;
       color: #2d3748;
       margin-bottom: 0.75rem;
       font-size: 1.1rem;
   }
   .video-badge {
       display: inline-block;
       padding: 0.35rem 0.85rem;
       border-radius: 8px;
       font-size: 0.8rem;
       font-weight: 600;
       text-transform: uppercase;
       letter-spacing: 0.5px;
   }
   .badge-auth { background: #d1fae5; color: #065f46; }
   .badge-dashboard { background: #dbeafe; color: #1e40af; }
   .badge-pwa { background: #fef3c7; color: #92400e; }

   .intro-box {
       background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
       color: white;
       padding: 2rem;
       border-radius: 12px;
       margin: 2rem 0;
       box-shadow: 0 8px 20px rgba(102, 126, 234, 0.3);
   }
   .intro-box h3 {
       color: white;
       margin-top: 0;
       font-size: 1.5rem;
   }
   .intro-box ul {
       margin: 1rem 0;
       padding-left: 1.5rem;
   }
   .intro-box li {
       margin: 0.5rem 0;
       line-height: 1.6;
   }
   </style>

Introduction
============

Bienvenue dans la **documentation vivante** de KoproGo ! Cette page présente les vidéos automatiquement générées par nos tests E2E Playwright. Chaque vidéo capture un parcours utilisateur réel, montrant l'application en action.

.. raw:: html

   <div class="intro-box">
       <h3>🎯 Pourquoi une documentation vivante ?</h3>
       <ul>
           <li><strong>Toujours à jour</strong> : Les vidéos sont régénérées à chaque CI/CD</li>
           <li><strong>Visuel et concret</strong> : Voir l'application fonctionner vaut mieux qu'un long texte</li>
           <li><strong>Tests + Docs en 1</strong> : Nos tests E2E servent aussi de documentation</li>
           <li><strong>Onboarding facilité</strong> : Les nouveaux contributeurs comprennent rapidement l'UX</li>
       </ul>
   </div>

Statistiques
============

.. raw:: html

   <div class="video-stats">
       <div class="stat-card">
           <div class="stat-number">30</div>
           <div class="stat-label">Tests E2E</div>
       </div>
       <div class="stat-card">
           <div class="stat-number">3</div>
           <div class="stat-label">Suites</div>
       </div>
       <div class="stat-card">
           <div class="stat-number">100%</div>
           <div class="stat-label">Couverture</div>
       </div>
       <div class="stat-card">
           <div class="stat-number">1280×720</div>
           <div class="stat-label">Résolution</div>
       </div>
   </div>

🔐 Tests d'Authentification
=============================

Les tests d'authentification couvrent tout le parcours utilisateur : de la landing page au login, jusqu'à la gestion de session et le logout.

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata" poster="">
               <source src="_static/videos/auth-landing-page.webm" type="video/webm">
               <source src="_static/videos/auth-landing-page.mp4" type="video/mp4">
               Votre navigateur ne supporte pas la balise vidéo.
           </video>
           <div class="video-info">
               <div class="video-title">Landing Page Visiteur</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-navigate-login.webm" type="video/webm">
               <source src="_static/videos/auth-navigate-login.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation vers Login</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-demo-credentials.webm" type="video/webm">
               <source src="_static/videos/auth-demo-credentials.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Comptes de Démonstration</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-login-success.webm" type="video/webm">
               <source src="_static/videos/auth-login-success.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Login Réussi + Redirection</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-error-invalid.webm" type="video/webm">
               <source src="_static/videos/auth-error-invalid.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Gestion des Erreurs</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-persist-reload.webm" type="video/webm">
               <source src="_static/videos/auth-persist-reload.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Persistance de Session</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-logout.webm" type="video/webm">
               <source src="_static/videos/auth-logout.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Déconnexion</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-redirect-syndic.webm" type="video/webm">
               <source src="_static/videos/auth-redirect-syndic.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Redirection Syndic</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/auth-redirect-accountant.webm" type="video/webm">
               <source src="_static/videos/auth-redirect-accountant.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Redirection Comptable</div>
               <span class="video-badge badge-auth">Auth</span>
           </div>
       </div>
   </div>

📊 Tests des Dashboards par Rôle
==================================

Chaque type d'utilisateur (Syndic, Comptable, Copropriétaire, Admin) a son propre dashboard avec des fonctionnalités adaptées à son rôle.

**Dashboard Syndic**
--------------------

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-syndic-sections.webm" type="video/webm">
               <source src="_static/videos/dashboard-syndic-sections.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Dashboard Syndic - Sections</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-syndic-navigation.webm" type="video/webm">
               <source src="_static/videos/dashboard-syndic-navigation.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation Syndic</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-navigate-buildings.webm" type="video/webm">
               <source src="_static/videos/dashboard-navigate-buildings.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation vers Immeubles</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-user-menu.webm" type="video/webm">
               <source src="_static/videos/dashboard-user-menu.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Menu Utilisateur</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>
   </div>

**Dashboard Comptable**
-----------------------

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-accountant-financial.webm" type="video/webm">
               <source src="_static/videos/dashboard-accountant-financial.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Dashboard Comptable - Finances</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-accountant-navigation.webm" type="video/webm">
               <source src="_static/videos/dashboard-accountant-navigation.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation Comptable</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>
   </div>

**Dashboard Copropriétaire**
----------------------------

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-owner-personal.webm" type="video/webm">
               <source src="_static/videos/dashboard-owner-personal.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Dashboard Copropriétaire</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-owner-limited.webm" type="video/webm">
               <source src="_static/videos/dashboard-owner-limited.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation Limitée (Owner)</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>
   </div>

**Dashboard Administrateur**
----------------------------

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-admin-overview.webm" type="video/webm">
               <source src="_static/videos/dashboard-admin-overview.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Dashboard Admin - Vue Globale</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-admin-full-access.webm" type="video/webm">
               <source src="_static/videos/dashboard-admin-full-access.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation Complète (Admin)</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>
   </div>

**Navigation Inter-Pages**
--------------------------

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-navigation-smooth.webm" type="video/webm">
               <source src="_static/videos/dashboard-navigation-smooth.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Navigation Fluide entre Pages</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/dashboard-auth-state-persist.webm" type="video/webm">
               <source src="_static/videos/dashboard-auth-state-persist.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Persistance État Auth</div>
               <span class="video-badge badge-dashboard">Dashboard</span>
           </div>
       </div>
   </div>

📱 Tests PWA et Mode Offline
==============================

KoproGo est une **Progressive Web App (PWA)** complète avec support offline, service worker, et synchronisation en tâche de fond.

.. raw:: html

   <div class="video-grid">
       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-manifest-valid.webm" type="video/webm">
               <source src="_static/videos/pwa-manifest-valid.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Manifest PWA</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-service-worker.webm" type="video/webm">
               <source src="_static/videos/pwa-service-worker.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Service Worker</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-online-status.webm" type="video/webm">
               <source src="_static/videos/pwa-online-status.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Indicateur En Ligne</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-offline-status.webm" type="video/webm">
               <source src="_static/videos/pwa-offline-status.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Mode Hors Ligne</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-indexeddb-storage.webm" type="video/webm">
               <source src="_static/videos/pwa-indexeddb-storage.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">IndexedDB Storage</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-cache-user-data.webm" type="video/webm">
               <source src="_static/videos/pwa-cache-user-data.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Cache Données Utilisateur</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-manual-sync.webm" type="video/webm">
               <source src="_static/videos/pwa-manual-sync.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Synchronisation Manuelle</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-offline-mode.webm" type="video/webm">
               <source src="_static/videos/pwa-offline-mode.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Fonctionnement Offline Complet</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>

       <div class="video-card">
           <video controls preload="metadata">
               <source src="_static/videos/pwa-sync-queue.webm" type="video/webm">
               <source src="_static/videos/pwa-sync-queue.mp4" type="video/mp4">
           </video>
           <div class="video-info">
               <div class="video-title">Queue de Synchronisation</div>
               <span class="video-badge badge-pwa">PWA</span>
           </div>
       </div>
   </div>

Comment Régénérer les Vidéos ?
================================

Pour régénérer les vidéos localement :

.. code-block:: bash

   # Méthode 1 : Via Make (recommandé)
   make docs-with-videos

   # Méthode 2 : Via npm scripts
   cd frontend
   npm run test:docs  # Lance les tests + copie les vidéos

   # Méthode 3 : Manuellement
   cd frontend && npm run test:e2e
   bash .claude/scripts/sync-playwright-videos.sh
   cd ../docs && make html

Configuration Playwright
=========================

Les vidéos sont configurées dans ``frontend/playwright.config.ts`` :

.. code-block:: typescript

   video: {
     mode: 'on',  // Enregistre toujours
     size: { width: 1280, height: 720 }
   }

Fichiers sources :

- ``frontend/tests/e2e/auth.spec.ts`` (9 tests)
- ``frontend/tests/e2e/dashboards.spec.ts`` (12 tests)
- ``frontend/tests/e2e/pwa-offline.spec.ts`` (9 tests)

Intégration CI/CD
=================

Les vidéos sont automatiquement régénérées et déployées via GitHub Actions :

1. ✅ Tests E2E exécutés sur chaque PR
2. 🎥 Vidéos capturées automatiquement
3. 📦 Artifacts uploadés
4. 📚 Documentation Sphinx générée
5. 🚀 Déploiement sur GitHub Pages (branche main)

Voir ``.github/workflows/docs-videos.yml`` pour la configuration complète.

Ressources
==========

- 📖 Guide E2E Testing : :doc:`E2E_TESTING_GUIDE`
- 🔗 Documentation Playwright : https://playwright.dev
- 🛠️ Makefile : :doc:`MAKEFILE_GUIDE`
- 🎯 Roadmap : :doc:`ROADMAP`

----

.. raw:: html

   <div style="text-align: center; margin: 3rem 0; color: #666;">
       <p><strong>🤖 Documentation vivante générée automatiquement avec Claude Code</strong></p>
       <p>KoproGo ASBL - Plateforme opensource de gestion de copropriété</p>
       <p style="font-size: 0.9rem;">Dernière mise à jour : automatique via CI/CD</p>
   </div>
