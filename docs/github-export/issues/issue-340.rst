====================================================================================
Issue #340: fix: RBAC manquant sur 9 endpoints gamification (TODO: Check admin role)
====================================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug,track:software priority:high
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/340>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   
   Dans `backend/src/infrastructure/web/handlers/gamification_handlers.rs`, **9 endpoints** acceptent `_auth: AuthenticatedUser` sans vérifier le rôle admin :
   
   ```rust
   _auth: AuthenticatedUser, // TODO: Check admin role (lines 38, 163, 190, 320, 465, 493, 520, 547, 573)
   ```
   
   ## Endpoints concernés
   
   - `POST /achievements` — Create achievement (devrait être admin/superadmin)
   - `PUT /achievements/{id}` — Update achievement
   - `DELETE /achievements/{id}` — Delete achievement
   - `POST /challenges` — Create challenge
   - `PUT /challenges/{id}/activate` — Activate challenge
   - `PUT /challenges/{id}/complete` — Complete challenge
   - `PUT /challenges/{id}/cancel` — Cancel challenge
   - `DELETE /challenges/{id}` — Delete challenge
   - `POST /challenges/{id}/progress/increment` — Increment progress
   
   ## Impact sécurité
   
   N'importe quel utilisateur authentifié peut créer/modifier/supprimer des achievements et challenges. Pas de séparation des privilèges.
   
   ## Fix requis
   
   Ajouter `require_role(&auth, &["admin", "superadmin"])` ou équivalent sur chaque handler.

.. raw:: html

   </div>

