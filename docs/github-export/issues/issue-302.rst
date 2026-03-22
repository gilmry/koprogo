==================================================================================================
Issue #302: [Bug] CRITIQUE : Isolation multi-tenant — données non filtrées par organization_id
==================================================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug:critique,test:e2e
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/302>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Plusieurs endpoints retournent les données de TOUTES les organisations au lieu de filtrer par `organization_id` de l'utilisateur authentifié.
   
   ## Impact
   **CRITIQUE** — Fuite de données inter-tenants. Violation RGPD potentielle.
   
   ## Endpoints concernés
   - GET /buildings (retourne les immeubles de toutes les organisations)
   - GET /owners (idem)
   - Potentiellement d'autres endpoints à auditer
   
   ## Correction proposée
   Ajouter un filtre `WHERE organization_id = $user.organization_id` à TOUS les handlers de listing.
   Audit systématique des 511 endpoints.
   
   ## Origine
   Tests E2E manuels — 22/03/2026

.. raw:: html

   </div>

