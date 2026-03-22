==============================================================================
Issue #305: [Bug] Création ticket : bouton silencieux si building_id manquant
==============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug:majeur,test:e2e
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/305>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Le bouton 'Create New Ticket' sur /tickets ne fait rien de visible quand `building_id` n'est pas dans les query params.
   Le handler appelle `showPageToast('Veuillez d'abord sélectionner un immeuble.', 'warning')` mais le toast ne s'affiche pas.
   
   ## Impact
   UX bloquante — l'utilisateur ne comprend pas pourquoi rien ne se passe.
   
   ## Correction proposée
   1. Ajouter un sélecteur d'immeuble dans la page tickets (dropdown)
   2. Corriger `showPageToast` pour qu'il affiche réellement le toast
   3. Désactiver le bouton si aucun immeuble n'est sélectionné
   
   ## Origine
   Tests E2E manuels — 22/03/2026 (tickets.astro lignes 93-121)

.. raw:: html

   </div>

