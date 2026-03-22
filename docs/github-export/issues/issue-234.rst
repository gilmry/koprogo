==============================================================================================
Issue #234: R&D: PWA offline sync - Architecture de synchronisation et résolution de conflits
==============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:medium R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/234>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #87 a mis en place la base PWA (Service Worker, IndexedDB, install prompt).
   Cette R&D couvre l'architecture de sync offline complète.
   
   **Issue liée**: #87
   **Documentation existante**: ``docs/PWA_OFFLINE.rst``
   
   ## Objectifs de la R&D
   
   1. **Stratégie de synchronisation** :
      - Background Sync API (queue d'actions offline)
      - Sync périodique vs. événementielle
      - Priorité des syncs (paiements > votes > consultations)
      - Bandwidth management (sync Wi-Fi only option)
   
   2. **Résolution de conflits** :
      - Last-write-wins (simple, perte potentielle)
      - CRDT (Conflict-free Replicated Data Types)
      - Server-side merge (complexe, flexible)
      - User-choice (présenter les 2 versions)
      - Cas critiques : double vote, double paiement
   
   3. **Cache strategy par module** :
      - Documents : cache-first (PDF rarement modifiés)
      - Charges : stale-while-revalidate
      - Notifications : network-first
      - Tickets : offline create + sync
   
   4. **IndexedDB schema** :
      - Tables miroir des API responses
      - Queue d'actions pendantes (``sync_queue``)
      - Versioning du schema (migrations IndexedDB)
      - Quota management (50MB default, demander extension)
   
   ## Points de décision
   
   - [ ] Conflict resolution strategy par type de donnée
   - [ ] CRDT nécessaire ou last-write-wins suffisant
   - [ ] Modules prioritaires pour offline (charges, documents, tickets)
   - [ ] Indicateur UI de mode offline (banner, badge)
   
   ## Estimation
   
   8-10h

.. raw:: html

   </div>

