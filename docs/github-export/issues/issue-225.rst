===========================================================================================
Issue #225: R&D: Application mobile native - Évaluation framework (RN vs Flutter vs Tauri)
===========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:medium R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/225>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #98 prévoit une application mobile native. Le choix du framework
   a un impact majeur sur le coût de développement et de maintenance.
   
   **Issue liée**: #98
   
   ## Objectifs de la R&D
   
   1. **Frameworks à évaluer** :
      - **React Native** : large écosystème, réutilisation partielle du JS
      - **Flutter** : performances natives, Dart (nouvelle compétence)
      - **Tauri Mobile** : Rust backend natif (aligné avec le stack)
      - **PWA avancée** : alternative sans app store (déjà en cours #87)
   
   2. **Critères d'évaluation** :
      - Performance (startup time, animations, liste scrolling)
      - Offline-first (sync avec IndexedDB/SQLite)
      - Taille de l'app (APK/IPA)
      - Biométrie (Touch ID, Face ID, empreinte)
      - Push notifications (FCM/APNs)
      - Coût de maintenance double codebase
      - Compétences équipe (Rust → Tauri avantage)
      - App Store policies (Apple review process)
   
   3. **Features mobile spécifiques** :
      - Scan QR code (accès AG, identification lot)
      - Notifications push (votes ouverts, tickets urgents)
      - Caméra (upload factures, photos interventions)
      - GPS (localisation bâtiments, prestataires proches)
   
   ## Points de décision
   
   - [ ] Framework retenu
   - [ ] PWA suffisante vs. app native nécessaire
   - [ ] Timeline de développement (post-Jalon 4 ou avant?)
   - [ ] Budget App Store (99$/an Apple, 25$ one-time Google)
   
   ## Estimation
   
   8-10h d'étude comparative

.. raw:: html

   </div>

