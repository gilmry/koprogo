=========================================================================================================
Issue #306: [Bug] CRITIQUE : Validation tantièmes — possible de dépasser 100% par ajouts séquentiels
=========================================================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: bug:critique,conformité test:e2e
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/306>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Le trigger PostgreSQL `validate_unit_ownership_total` avertit si total < 100% mais bloque seulement > 100%.
   En ajoutant des propriétaires séquentiellement, on peut arriver à un état incohérent.
   
   ## Impact
   **CRITIQUE** — Les calculs de charges et votes seront erronés. Non-conformité Art. 577-2 §4 CC.
   
   ## Correction proposée
   1. Bloquer au niveau frontend avec calcul en temps réel du total restant
   2. Renforcer le trigger PostgreSQL avec validation transactionnelle
   3. Ajouter un endpoint de validation : GET /units/:id/owners/total-percentage
   
   ## Origine
   Tests E2E manuels — 22/03/2026

.. raw:: html

   </div>

