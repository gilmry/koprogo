============================================================================
Issue #30: feat: Améliorer l'affichage des comptes de test dans SeedManager
============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: documentation
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-10-27
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/30>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   Les comptes de test générés par le seed doivent être facilement accessibles aux développeurs et testeurs.
   
   ## Travail effectué ✅
   - [x] Affichage permanent des comptes (ne disparaît plus après quelques secondes)
   - [x] Comptes affichés dès qu'il y a des organisations seed dans la DB
   - [x] Affichage de TOUS les comptes (SuperAdmin, 3 Syndics, Comptable, 2 Propriétaires)
   - [x] UI améliorée avec rôles, organisations, et boutons copier
   - [x] Suppression des comptes hardcodés dans LoginForm.svelte
   - [x] Auto-reload des comptes après génération/suppression du seed
   
   ## Comptes affichés
   1. 👑 SuperAdmin - admin@koprogo.com / admin123
   2. 🏢 Syndic (Grand Place) - syndic@grandplace.be / syndic123
   3. 🏢 Syndic (Bruxelles) - syndic@copro-bruxelles.be / syndic123
   4. 🏢 Syndic (Liège) - syndic@syndic-liege.be / syndic123
   5. 📊 Comptable - comptable@grandplace.be / comptable123
   6. 👤 Propriétaire 1 - proprietaire1@grandplace.be / owner123
   7. 👤 Propriétaire 2 - proprietaire2@grandplace.be / owner123
   
   ## Changements techniques
   - `SeedManager.svelte`: Logique d'affichage basée sur `seedStats.seed_organizations > 0`
   - `LoginForm.svelte`: Suppression du bloc hardcodé (lignes 120-147)
   - Interface améliorée: badges de rôle, boutons copier, design cohérent
   
   ## Issue fermée par
   Cette issue documente le travail déjà effectué. Peut être fermée immédiatement.

.. raw:: html

   </div>

