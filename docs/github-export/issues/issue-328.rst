==================================================================================
Issue #328: feat(security): Gestion des clés API (API Keys CRUD + hashing SHA-256)
==================================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: enhancement
:Assignees: Unassigned
:Created: 2026-03-24
:Updated: 2026-03-24
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/328>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Implémenté dans le merge integration → main (2026-03-24).
   
   ## Implémentation existante
   - **Handler**: `api_key_handlers.rs`
   - **Migration**: `20260323000013_create_api_keys.sql`
   - **E2E tests**: `e2e_api_keys.rs`, `ApiKeys.spec.ts` (Playwright)
   - **Sécurité**: Hashing SHA-256 (crate sha2), clés jamais stockées en clair
   
   ## Statut
   ✅ **DONE** — Backend + Tests implémentés.
   
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

