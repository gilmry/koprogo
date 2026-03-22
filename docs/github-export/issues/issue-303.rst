=========================================================================================
Issue #303: [Bug] Calcul tantièmes : total des lots ≠ total immeuble (1000 millièmes)
=========================================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: bug:majeur,conformité test:e2e
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/303>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Aucune validation frontend/backend ne vérifie que la somme des tantièmes des lots = 1000 millièmes.
   Le trigger PostgreSQL `validate_unit_ownership_total` valide les quotes-parts des propriétaires mais pas la somme des tantièmes structurels.
   
   ## Impact
   Non-conformité Art. 577-2 §4 Code Civil belge. Les calculs de quorum et charges seront erronés.
   
   ## Correction proposée
   1. Ajouter un champ `total_shares` au Building (défaut: 1000)
   2. Valider à la création/modification d'un lot que sum(unit.shares) <= building.total_shares
   3. Afficher un warning si sum < total (lots manquants)
   
   ## Origine
   Tests E2E manuels — 22/03/2026

.. raw:: html

   </div>

