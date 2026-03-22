==================================================================================
Issue #310: [Conformité] AG : Lien agenda-résolutions — bloquer votes hors ODJ
==================================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: conformité
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/310>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Exigence légale
   Art. 3.87 Code Civil belge : seuls les points inscrits à l'ordre du jour peuvent faire l'objet d'un vote.
   Un vote hors ODJ est frappé de nullité.
   
   ## Implémentation requise
   - Ajouter FK resolution.agenda_item_id → meeting_agenda_items
   - Bloquer la création de résolutions sans point d'agenda correspondant
   - BDD scenario dans features/meetings.feature
   
   ## Statut matrice
   MANQUANT (matrice_conformite.rst)

.. raw:: html

   </div>

