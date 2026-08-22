===============================================================================
Issue #339: fix: API key rotation non implémenté (retourne 501 Not Implemented)
===============================================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:software priority:medium
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/339>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   
   `POST /api-keys/{id}/rotate` retourne `HttpResponse::NotImplemented()` dans `backend/src/infrastructure/web/handlers/api_key_handlers.rs:505`.
   
   ```rust
   // TODO: Implement key rotation (line 505)
   ```
   
   Les 6 autres endpoints API key fonctionnent (CRUD + revoke).
   
   ## Action requise
   
   - Implémenter la rotation : générer nouvelle clé, invalider l'ancienne, retourner la nouvelle
   - Audit log de la rotation
   - Test E2E
   
   ## Sévérité
   
   MEDIUM — les autres opérations CRUD fonctionnent, la rotation est un nice-to-have pour la production.

.. raw:: html

   </div>

