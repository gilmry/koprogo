=====================================================================================
Issue #332: fix(ci): Corrections CI pré-existantes (formatting, RUSTSEC, astro check)
=====================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: bug:mineur
:Assignees: Unassigned
:Created: 2026-03-24
:Updated: 2026-03-24
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/332>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Implémenté sur main (PR #325, 2026-03-24).
   
   ## Corrections
   - **cargo fmt**: 54 fichiers backend reformatés
   - **prettier**: 129 fichiers frontend reformatés
   - **RUSTSEC-2026-0066**: testcontainers 0.27.1 → 0.27.2 (supprime astral-tokio-tar 0.5.6 vulnérable)
   - **astro check**: 53 erreurs TypeScript corrigées (tickets.astro, contractor-report)
   - **SSG build**: contractor-report/[token].astro → index.astro (fix GetStaticPathsRequired)
   
   ## Statut
   ✅ **DONE** — PR #325 mergée.
   
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

