==========================================================================
Issue #307: [Bug] Sondages/Annonces/Réservations : immeubles non chargés
==========================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug:majeur,test:e2e
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/307>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Les pages Sondages (/polls), Annonces (/announcements) et Réservations (/bookings) ne chargent pas la liste des immeubles, rendant impossible la création de contenu.
   
   ## Impact
   Pages communautaires inutilisables.
   
   ## Correction proposée
   Corriger l'appel API pour charger les immeubles de l'organisation du syndic (filtré par organization_id).
   
   ## Origine
   Tests E2E manuels — 22/03/2026

.. raw:: html

   </div>

