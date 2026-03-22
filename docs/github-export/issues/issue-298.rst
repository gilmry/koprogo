=============================================================================
Issue #298: Sync bidirectionnelle SQLite ↔ PostgreSQL (mode offline/online)
=============================================================================

:State: **OPEN**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: enhancement,track:software tauri,offline
:Assignees: Unassigned
:Created: 2026-03-21
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/298>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Les applications desktop et mobile Tauri utilisent SQLite localement. Il faut synchroniser les données avec le serveur PostgreSQL quand l'utilisateur est connecté.
   
   ## Objectif
   
   Implémenter une synchronisation bidirectionnelle fiable entre SQLite (local) et PostgreSQL (serveur).
   
   ## Stratégie de sync
   
   ### Option A : Sync basée sur timestamps (recommandé pour MVP)
   - Chaque entité a `updated_at` → sync les modifications depuis le dernier sync
   - Résolution de conflits : last-write-wins ou user-choice
   
   ### Option B : Event sourcing / CRDT (long terme)
   - Chaque modification = événement immutable
   - Merge automatique sans conflit
   - Plus complexe mais plus robuste
   
   ## Tâches
   
   - [ ] Définir la stratégie de sync (timestamps vs events)
   - [ ] Implémenter le sync engine (pull server → local, push local → server)
   - [ ] Gestion des conflits (détection + résolution UI)
   - [ ] Queue de modifications offline (opérations pendantes)
   - [ ] Indicateur de statut sync dans l'UI (synced/pending/conflict)
   - [ ] Sync sélective (ne pas tout synchroniser, filtrer par building)
   - [ ] Retry avec backoff exponentiel en cas d'erreur réseau
   - [ ] Tests : scénarios offline → online, conflits, interruption réseau
   
   ## Dépendances
   
   - Adapters SQLite (issue dédiée)
   - API publique REST v1 (issue #87 ou dédiée)
   
   ## Critères d'acceptation
   
   - [ ] Modifications offline sauvegardées et synchronisées au retour online
   - [ ] Conflits détectés et présentés à l'utilisateur
   - [ ] Pas de perte de données en cas de coupure réseau
   - [ ] Sync incrémentale (pas de full refresh)

.. raw:: html

   </div>

