==============================================================================================
Issue #338: bug: energy_bill_upload — Uuid::nil() hardcodé pour unit_id casse get_my_uploads()
==============================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: bug,track:software priority:high
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/338>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Bug
   
   Dans `backend/src/infrastructure/web/handlers/energy_bill_upload_handlers.rs:88`, le `unit_id` est hardcodé à `Uuid::nil()` au lieu d'être résolu depuis la table `unit_owners`.
   
   ```rust
   // TODO: Get unit_id from unit_owners table based on user_id (line 81)
   let unit_id = Uuid::nil(); // Placeholder (line 88)
   ```
   
   ## Impact
   
   - `GET /energy-bills/my-uploads` retourne toujours une liste vide (aucun unit n'a l'ID nil)
   - L'upload fonctionne mais associe les factures à un unit inexistant
   
   ## Fix requis
   
   Remplacer `Uuid::nil()` par un lookup `unit_owners WHERE owner_id = user.owner_id AND end_date IS NULL`.
   
   ## Contexte
   
   Détecté lors d'un audit croisé code vs GitHub issues (2026-03-25).

.. raw:: html

   </div>

