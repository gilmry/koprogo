=================================================================================================
Issue #336: fix: Individual member handlers 100% stub — #280 fermée mais aucune implémentation DB
=================================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: bug,track:software priority:high
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/336>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   
   L'issue #280 a été fermée, mais `backend/src/infrastructure/web/handlers/individual_member_handlers.rs` contient **18 TODOs** et **aucune intégration DB**.
   
   ## Endpoints concernés (tous non fonctionnels)
   
   | Endpoint | Comportement actuel |
   |----------|-------------------|
   | `POST /energy-campaigns/{id}/join-as-individual` | Crée objet mais ne persiste pas |
   | `POST /energy-campaigns/{id}/members/{id}/consent` | Retourne JSON hardcodé |
   | `PUT /energy-campaigns/{id}/members/{id}/consumption` | Retourne JSON hardcodé |
   | `DELETE /energy-campaigns/{id}/members/{id}/withdraw` | Retourne JSON hardcodé |
   
   ## Exemples de TODOs critiques
   
   ```rust
   // TODO: Save to database using repository (line 32)
   // TODO: Check for duplicate email in campaign (line 33)
   // TODO: Send confirmation email with consent link (line 34)
   // TODO: Schedule data anonymization/deletion (GDPR Article 17) (line 95)
   email: "user@example.com".to_string(), // TODO: Get from member (line 100)
   ```
   
   ## Action requise
   
   - Créer `IndividualMemberRepository` + implémentation PostgreSQL
   - Migration pour table `energy_campaign_individual_members`
   - Implémenter persistance + GDPR consent trail
   - Tests E2E
   
   ## Contexte
   
   Détecté lors d'un audit croisé code vs GitHub issues (2026-03-25).
   Réf: #280

.. raw:: html

   </div>

