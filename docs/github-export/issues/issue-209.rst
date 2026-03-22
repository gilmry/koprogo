=============================================================================
Issue #209: feat(tests): Playwright expansion - 7 new frontend E2E spec files
=============================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: priority:medium,release:v0.5.0 testing,playwright
:Assignees: Unassigned
:Created: 2026-02-26
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/209>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Seulement 3 specs Playwright existent (AdminDashboard, BoardOfDirectors, Gdpr), couvrant <5% des 80 pages frontend.
   
   ## Specs à créer (~15h)
   
   | Spec | Pages couvertes | Tests | Heures |
   |------|----------------|-------|--------|
   | `Login.spec.ts` | `/login` | 5 | 2h |
   | `Buildings.spec.ts` | `/buildings`, `/building-detail` | 5 | 2h |
   | `Expenses.spec.ts` | `/invoice-workflow` | 6 | 2h |
   | `Meetings.spec.ts` | `/meetings`, `/meeting-detail` | 5 | 2h |
   | `Tickets.spec.ts` | `/tickets`, `/ticket-detail` | 6 | 2h |
   | `Notifications.spec.ts` | `/notifications` | 5 | 2h |
   | `OwnerDashboard.spec.ts` | `/owner-dashboard` | 5 | 3h |
   
   ## Pattern
   
   Suivre `frontend/tests/e2e/Gdpr.spec.ts` :
   - Setup via API (pas navigation UI)
   - Sélecteurs `data-testid`
   - Assertions contenu visible
   
   ## Prérequis
   
   - [ ] Ajouter `data-testid` aux composants Svelte concernés
   - [ ] Ajouter job `playwright` dans `.github/workflows/ci.yml`
   
   Parent: #207

.. raw:: html

   </div>

