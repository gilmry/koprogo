===================================================
Issue #296: Tauri Mobile: Application iOS & Android
===================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: enhancement,track:software tauri,mobile
:Assignees: Unassigned
:Created: 2026-03-21
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/296>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Tauri v2 supporte iOS et Android avec la même codebase Rust + frontend web. Cela remplace/complète l'approche PWA (#87) avec une vraie app native distribuable sur les stores.
   
   ## Objectif
   
   Créer une application mobile native KoproGo pour iOS et Android via Tauri v2.
   
   ## Tâches
   
   - [ ] Configurer Tauri mobile (iOS + Android targets)
   - [ ] Adapter le frontend Svelte pour mobile (responsive, touch-first)
   - [ ] Plugins Tauri mobile : notifications push, biométrie, caméra (photos documents)
   - [ ] Deep links pour navigation depuis emails/notifications
   - [ ] Distribution App Store (iOS) et Google Play (Android)
   - [ ] Splash screen et icônes adaptatives
   - [ ] Tests sur devices réels (iOS 16+, Android 10+)
   
   ## Relation avec PWA (#87)
   
   - **PWA** : accès rapide via navigateur, pas d'installation store
   - **Tauri Mobile** : app native, accès capteurs, notifications push, mode offline
   - Les deux peuvent coexister (PWA comme fallback web)
   
   ## Critères d'acceptation
   
   - [ ] App publiable sur App Store et Google Play
   - [ ] Notifications push fonctionnelles
   - [ ] Mode offline avec sync (voir issue dédiée)
   - [ ] Performance : cold start < 2s
   - [ ] Biométrie pour login (Face ID / fingerprint)

.. raw:: html

   </div>

