===========================================================================================
Issue #331: test(playwright): 48 fichiers E2E Playwright frontend couvrant tous les modules
===========================================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: test:e2e
:Assignees: Unassigned
:Created: 2026-03-24
:Updated: 2026-04-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/331>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Implémenté sur main (commits `1c1834e`, `76e8857`, mars 2026).
   
   ## Implémentation existante
   - **48 spec files** dans `frontend/tests/e2e/`
   - **Couverture**: Login, Buildings, Meetings, Expenses, Payments, Tickets, Notifications, GDPR, Polls, Gamification, ApiKeys, Consent, SecurityIncidents, Marketplace, LegalHelper, etc.
   - **Global setup**: `global-setup.ts` avec seed data + auth helpers
   - **Helpers**: `api-client.ts`, `auth.ts`, `test-world.ts`
   - **CI**: Job Playwright E2E Tests dans GitHub Actions (219 passed, 21 failures pré-existants ApiKeys/SecurityIncidents)
   
   ## Reste à faire
   - [ ] Corriger 21 tests en échec (ApiKeys + SecurityIncidents — NULL constraint building_id)
   - [ ] Valider les 32 spec files non encore exécutés en CI
   
   ## Statut
   ⚠️ **PARTIEL** — 219/240 tests passent, 21 à corriger.
   
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

