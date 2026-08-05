=========================================================================
Issue #330: refactor(i18n): Centralisation svelte-i18n + 4 langues belges
=========================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: enhancement
:Assignees: Unassigned
:Created: 2026-03-24
:Updated: 2026-03-24
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/330>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Implémenté sur main (commits `866e878` → `a1765f5`, mars 2026).
   
   ## Implémentation existante
   - **Module centralisé**: `frontend/src/lib/i18n.ts` — setupI18n() synchrone, re-exports $_/locale/isLoading
   - **Locales**: fr.json, nl.json, de.json, en.json (~800 clés chacune)
   - **Migration**: 60+ composants migrés depuis import direct svelte-i18n vers lib/i18n
   - **Fix hydration**: Chargement synchrone des 4 locales pour éviter race condition Astro SSG + Svelte islands
   - **Playwright**: `I18n.spec.ts`
   
   ## Statut
   ✅ **DONE**
   
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

