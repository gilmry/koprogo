====================================================================================
Issue #337: fix: Consent handlers 100% stub — #326 fermée mais aucune persistance DB
====================================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug,track:software priority:high
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/337>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   
   L'issue #326 (GDPR Art. 7 consent management) a été fermée, mais `backend/src/infrastructure/web/handlers/consent_handlers.rs` ne persiste **rien en base de données**.
   
   ## Endpoints concernés
   
   | Endpoint | Comportement actuel |
   |----------|-------------------|
   | `POST /consent` | Valide l'input mais **ne persiste pas** (TODO line 131) |
   | `GET /consent/status` | Retourne des **false hardcodés** (TODO line 168) |
   
   ## Code problématique
   
   ```rust
   // TODO: Implement database persistence in consent repository (line 131)
   // TODO: Query database for consent records (line 168)
   ```
   
   ## Impact GDPR
   
   - **Art. 7**: Le consentement doit être enregistré et prouvable
   - **Art. 30**: Pas d'audit trail pour les consentements
   - Bloquant pour la conformité GDPR complète
   
   ## Action requise
   
   - Créer `ConsentRepository` trait + implémentation PostgreSQL
   - Migration pour table `user_consents` (consent_type, ip_address, user_agent, timestamp)
   - Implémenter persistance + audit trail
   - Tests
   
   ## Contexte
   
   Détecté lors d'un audit croisé code vs GitHub issues (2026-03-25).
   Réf: #326

.. raw:: html

   </div>

