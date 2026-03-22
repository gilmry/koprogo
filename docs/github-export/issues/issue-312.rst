=============================================================================
Issue #312: [Conformité] AG : Procurations — max 3 mandats + exception 10%
=============================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: conformité
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/312>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Exigence légale
   Art. 3.87 §7 CC : Un mandataire ne peut détenir plus de 3 procurations, sauf s'il ne dépasse pas 10% du total des voix.
   
   ## Implémentation requise
   - Compter les procurations par proxy_owner_id lors du vote
   - Bloquer si > 3 procurations ET > 10% des voix totales
   - Afficher le nombre de procurations restantes dans l'UI
   - BDD scenarios
   
   ## Statut matrice
   MANQUANT (matrice_conformite.rst)

.. raw:: html

   </div>

