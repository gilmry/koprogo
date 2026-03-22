============================================================================
Issue #301: [Bug] Permissions rôles : boutons admin visibles pour le syndic
============================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug:majeur,test:e2e
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/301>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Le syndic voit les boutons **Créer Organisation** et **Créer Utilisateur** dans le sidebar.
   Ces actions sont réservées au rôle SuperAdmin.
   
   ## Impact
   Risque de confusion UX. Les endpoints backend retournent 403, mais l'UI ne devrait pas exposer ces actions.
   
   ## Correction proposée
   Filtrer les éléments de navigation par `active_role` dans le frontend (Navigation.svelte).
   
   ## Origine
   Tests E2E manuels — 22/03/2026

.. raw:: html

   </div>

