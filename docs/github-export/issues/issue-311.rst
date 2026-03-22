==============================================================================
Issue #311: [Conformité] AG : Quorum 50%+50% et 2ème convocation automatique
==============================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: conformité
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/311>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Exigence légale
   Art. 3.87 §5 CC : Si le quorum (>50% des quotes-parts) n'est pas atteint à la 1ère convocation, une 2ème convocation est obligatoire (15 jours minimum).
   À la 2ème convocation, aucun quorum n'est requis.
   
   ## Implémentation requise
   - Ajouter champ meeting.is_second_convocation (boolean)
   - Valider quorum seulement si is_second_convocation = false
   - Auto-créer Meeting + Convocation de 2ème appel si quorum KO
   - BDD scenarios
   
   ## Statut matrice
   MANQUANT (matrice_conformite.rst)

.. raw:: html

   </div>

