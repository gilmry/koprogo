=======================================================================
Issue #406: a11y: integrate @axe-core/playwright into CI (STORY-P7-801)
=======================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: accessibility,audit-2026-04
:Assignees: Unassigned
:Created: 2026-04-18
:Updated: 2026-04-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/406>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Sprint 8 — STORY-P7-801. From the WCAG 2.1 AA audit produced at the close of Sprint 7.
   
   `@axe-core/playwright@^4.11.1` is already installed in `frontend/package.json` but **no accessibility check runs in CI**. Top-impact fix (10 min setup).
   
   ## Scope
   
   1. Add an `npm run test:a11y` script that runs Playwright with axe-core against the smoke-test pages (/, /login, /dashboard, /buildings, /meetings, /tickets, /owners).
   2. Add a new `playwright-a11y` job in `.github/workflows/ci.yml` that:
      - Reuses the backend+frontend setup from the existing `playwright` job
      - Runs `npm run test:a11y`
      - Uploads the axe JSON report as an artifact
      - Fails on any WCAG 2.1 AA violation (severity = serious, critical)
   3. Baseline: accept the current violation count; any regression fails CI.
   
   ## Acceptance
   
   - PR that introduces an `<img>` without `alt` attribute on a smoke-tested page → CI fails
   - PR that fixes an existing violation → CI passes + the baseline is updated in the same commit
   
   ## Files
   
   - `frontend/playwright-a11y.config.ts` (new, extends playwright.config.ts)
   - `frontend/tests/a11y/*.spec.ts` (new, one file per smoke page)
   - `.github/workflows/ci.yml` (new job)
   
   Part of Sprint 8 roadmap toward v0.1.0 release.

.. raw:: html

   </div>

