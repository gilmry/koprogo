==============================================================================
Issue #297: Adapters SQLite: Implémentation Repository traits pour mode local
==============================================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: enhancement,track:software tauri,offline
:Assignees: Unassigned
:Created: 2026-03-21
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/297>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'architecture hexagonale (Ports & Adapters) définit des traits Repository dans la couche Application. Actuellement seuls les adapters PostgreSQL existent. Pour le mode desktop/mobile offline, il faut des adapters SQLite.
   
   ## Objectif
   
   Implémenter les traits Repository existants avec SQLite comme backend de stockage local.
   
   ## Tâches
   
   - [ ] Créer un crate partagé `koprogo-core` extrayant Domain + Application (ou workspace Cargo)
   - [ ] Ajouter la dépendance `sqlx` avec feature `sqlite`
   - [ ] Implémenter les Repository traits prioritaires :
     - [ ] `SqliteBuildingRepository`
     - [ ] `SqliteUnitRepository`
     - [ ] `SqliteOwnerRepository`
     - [ ] `SqliteExpenseRepository`
     - [ ] `SqliteMeetingRepository`
     - [ ] `SqliteDocumentRepository`
     - [ ] `SqliteNotificationRepository`
   - [ ] Migrations SQLite (schéma simplifié adapté aux contraintes SQLite)
   - [ ] Tests unitaires pour chaque adapter SQLite
   - [ ] Benchmark comparatif PostgreSQL vs SQLite (latence locale)
   
   ## Architecture
   
   ```
   backend/src/application/ports/building_repository.rs  (trait existant)
       ↓ implémenté par
   backend/src/infrastructure/database/repositories/building_repository_impl.rs  (PostgreSQL - existant)
   desktop/src/adapters/sqlite/building_repository.rs  (SQLite - NOUVEAU)
   ```
   
   ## Notes techniques
   
   - SQLite ne supporte pas certains types PostgreSQL (ENUM, ARRAY) → mapping nécessaire
   - UUID stockés comme TEXT en SQLite
   - JSONB → JSON TEXT en SQLite
   - Pas de types TIMESTAMPTZ → TEXT ISO8601
   
   ## Critères d'acceptation
   
   - [ ] Tous les traits Repository core implémentés pour SQLite
   - [ ] Tests passants avec SQLite en mémoire
   - [ ] Même interface que PostgreSQL (interchangeable)

.. raw:: html

   </div>

