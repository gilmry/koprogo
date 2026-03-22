========================================================================================
Issue #299: Workspace Cargo: Extraire crate partagé koprogo-core (Domain + Application)
========================================================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: enhancement,track:software tauri
:Assignees: Unassigned
:Created: 2026-03-21
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/299>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Pour partager le code Domain et Application entre le backend Actix-web (PostgreSQL) et les clients Tauri (SQLite), il faut restructurer le projet en workspace Cargo.
   
   ## Objectif
   
   Créer un workspace Cargo avec un crate `koprogo-core` contenant Domain + Application, réutilisable par le backend serveur ET les applications Tauri.
   
   ## Structure cible
   
   ```
   Cargo.toml (workspace)
   ├── crates/
   │   ├── koprogo-core/        # Domain + Application (entities, use_cases, ports, DTOs)
   │   │   ├── src/domain/
   │   │   └── src/application/
   │   ├── koprogo-server/      # Infrastructure Actix-web + PostgreSQL (actuel backend/)
   │   │   └── src/infrastructure/
   │   └── koprogo-desktop/     # Infrastructure Tauri + SQLite
   │       └── src-tauri/
   ```
   
   ## Tâches
   
   - [ ] Créer le workspace Cargo root
   - [ ] Extraire `backend/src/domain/` → `crates/koprogo-core/src/domain/`
   - [ ] Extraire `backend/src/application/` → `crates/koprogo-core/src/application/`
   - [ ] Adapter les imports dans `backend/` (dépendance sur `koprogo-core`)
   - [ ] Vérifier que tous les tests passent après restructuration
   - [ ] Documenter la structure workspace
   
   ## Prérequis
   
   - Aucun (peut être fait avant Tauri)
   
   ## Critères d'acceptation
   
   - [ ] `cargo build --workspace` compile sans erreur
   - [ ] `cargo test --workspace` passe à 100%
   - [ ] `koprogo-core` n'a aucune dépendance sur Actix-web, SQLx-postgres, ou Tauri
   - [ ] Le backend serveur fonctionne identiquement après refactoring

.. raw:: html

   </div>

