=====================================================================================
Issue #327: feat(security): Gestion des incidents de sécurité (GDPR Art. 33 registre)
=====================================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: gdpr
:Assignees: Unassigned
:Created: 2026-03-24
:Updated: 2026-03-24
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/327>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Implémenté dans le merge integration → main (2026-03-24).
   
   ## Implémentation existante
   - **Handler**: `security_incident_handlers.rs`
   - **Migration**: `20260323000006_create_security_incidents.sql`
   - **E2E tests**: `e2e_security_incidents.rs`, `SecurityIncidents.spec.ts` (Playwright)
   - **Domain entity**: severity, incident_type, status workflow
   
   ## Statut
   ✅ **DONE** — Backend + Tests implémentés.
   
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

