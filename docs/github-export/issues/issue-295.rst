==================================================================
Issue #295: Tauri Shell: Application desktop (Windows/macOS/Linux)
==================================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: enhancement,track:software tauri,desktop
:Assignees: Unassigned
:Created: 2026-03-21
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/295>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'architecture hexagonale de KoproGo permet de brancher de nouveaux adapters Infrastructure sans toucher au Domain ni aux Use Cases. Tauri v2 permet de créer des applications desktop natives avec un backend Rust et un frontend web.
   
   ## Objectif
   
   Créer un shell Tauri pour distribuer KoproGo comme application desktop installable sur Windows, macOS et Linux.
   
   ## Tâches
   
   - [ ] Initialiser le projet Tauri v2 dans `desktop/` (ou `apps/desktop/`)
   - [ ] Configurer le frontend Svelte existant comme webview Tauri
   - [ ] Créer les commandes Tauri (`#[tauri::command]`) wrappant les Use Cases existants
   - [ ] Implémenter les adapters SQLite pour les Repository traits (voir issue dédiée)
   - [ ] Packaging et distribution (MSI Windows, DMG macOS, AppImage/deb Linux)
   - [ ] Auto-update via Tauri Updater
   - [ ] Icônes et branding KoproGo
   - [ ] Tests E2E desktop (Tauri WebDriver)
   
   ## Architecture
   
   ```
   frontend/src/ (Svelte components) ──→ Tauri Webview
   backend/src/domain/         ──→ Réutilisé tel quel (crate partagé)
   backend/src/application/    ──→ Réutilisé tel quel (crate partagé)
   desktop/src-tauri/          ──→ Nouveaux adapters (SQLite repos, Tauri commands)
   ```
   
   ## Dépendances
   
   - Issue SQLite adapters (voir issue dédiée)
   - Issue sync offline/online (voir issue dédiée)
   
   ## Critères d'acceptation
   
   - [ ] Application installable sur les 3 OS
   - [ ] Toutes les fonctionnalités core accessibles (Buildings, Units, Owners, Expenses)
   - [ ] Taille binaire < 15 MB
   - [ ] Auto-update fonctionnel

.. raw:: html

   </div>

