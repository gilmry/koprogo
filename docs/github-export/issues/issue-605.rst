================================================================================
Issue #605: [FLAKY] 3 Gdpr Playwright E2E tests fail with timing/race conditions
================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug
:Assignees: Unassigned
:Created: 2026-05-27
:Updated: 2026-05-27
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/605>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   3 tests Gdpr Playwright E2E échouent consistemment depuis longtemps (pré-existant, hors scope slice 2/#602) :
   
   | # | Test | Symptôme |
   |---|---|---|
   | 1 | `Gdpr.spec.ts:300` GDPR - Audit Logs Verification → should record all GDPR operations in audit logs | `getByTestId('admin-gdpr-audit-log-row').first()` not visible after 5s |
   | 2 | `Gdpr.spec.ts:358` GDPR - Cross-Organization Access → should allow SuperAdmin to access any user regardless of organization | `getByTestId('admin-gdpr-erasure-result')` not visible after 10s |
   | 3 | `Gdpr.spec.ts:226` GDPR - Mixed Scenario: User Creates Data, Admin Exports | Flaky : passe sur retry #2 (`gdpr-export-button` element detached from DOM) |
   
   ## Hypothèses
   
   - **Audit log async** : le POST /gdpr/export crée un log async via spawn task ; le test ne wait pas que l'écriture DB soit committée avant le toggle admin
   - **DOM detach race** : modal/page re-render entre 2 actions (`gdpr-export-button` est detached pendant click)
   - **Erasure async** : `admin-gdpr-erase-user` → POST /gdpr/erasure → la réponse arrive avant que le toast/result soit rendu
   
   ## Recette proposée
   
   Investigation app-running (npm run dev + manual repro) :
   1. Lancer le flow Gdpr export+audit log manuellement
   2. Vérifier que `admin-gdpr-audit-log-row` apparaît bien avec un timeout > 5s
   3. Si le log est créé en async (spawn task), wrapper avec un `waitFor` sur l'audit endpoint
   4. Pour le DOM detach : ajouter `await page.waitForLoadState('networkidle')` avant les clicks fragiles
   
   ## Critères de validation
   
   - [ ] 3/3 Gdpr tests GREEN sur 3 runs CI consécutifs (pas de flake)
   - [ ] Pas de retry needed
   - [ ] Test isolés runnable localement via `npx playwright test Gdpr` → pass first try
   
   ## Liens
   
   - Découvert pendant batch CI sweep #602 (commits 18bf529..082f6b8)
   - Non bloquant pour slice 2 (Story 2.5 E2E all GREEN)
   
   @gilmry à fermer manuellement quand patché.

.. raw:: html

   </div>

