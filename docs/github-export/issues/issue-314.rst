===================================================================
Issue #314: [Conformité] Syndic : Mandat max 3 ans avec validation
===================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: conformité
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/314>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Exigence légale
   Art. 3.89 CC : Le mandat du syndic ne peut excéder 3 ans, renouvelable.
   
   ## Implémentation requise
   - Ajouter validation domain : mandate_end - mandate_start <= 3 ans
   - Alerte avant expiration du mandat (3 mois, 1 mois, 15 jours)
   - Point AGO obligatoire pour renouvellement
   
   ## Statut matrice
   MANQUANT (matrice_conformite.rst)

.. raw:: html

   </div>

