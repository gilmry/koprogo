===============================================================================================
Issue #349: E2E multi-roles: aligner les scenarios sur les specs + seeds + teardown (etape 4/5)
===============================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: priority:high,testing
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/349>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Etape 4 : Aligner les E2E Documentation Vivante sur les specs
   
   Parent : #345
   Depend de : #346 (specs), #347 (seeds), #348 (BDD alignes)
   
   ### Objectif
   
   Reecrire les 6 scenarios E2E en echec + corriger les 6 qui passent pour qu'ils :
   - Utilisent les seeds de l'etape 2 (beforeAll appelle `POST /seed/scenario/{name}`)
   - Respectent le narratif multi-roles des specs de l'etape 1
   - Incluent un afterAll qui appelle `DELETE /seed/scenario/{name}` (teardown)
   - Produisent des videos conformes au narratif metier
   
   ### Pattern standard pour chaque scenario
   
   ```typescript
   test.describe("Scenario: Vote AG multi-role", () => {
     test.setTimeout(180_000);
     let seedData: any;
   
     test.beforeAll(async ({ request }) => {
       // Appeler le seed pre-configure (pas de setup manuel!)
       const resp = await request.post(`${API_BASE}/seed/scenario/ag_with_quorum`, {
         headers: adminHeaders,
       });
       seedData = await resp.json();
     });
   
     test.afterAll(async ({ request }) => {
       // Cleanup : supprimer toutes les donnees seedees
       await request.delete(`${API_BASE}/seed/scenario/ag_with_quorum`, {
         headers: adminHeaders,
       });
     });
   
     test("Syndic prepare, coproprietaire vote, syndic cloture", async ({ page }) => {
       // Etape 1 : Syndic
       await humanLogin(page, seedData.syndic_email, seedData.syndic_password);
       // ... actions syndic ...
   
       // Etape 2 : Coproprietaire
       await humanLogin(page, seedData.owner_email, seedData.owner_password);
       // ... actions owner ...
   
       // Etape 3 : Retour syndic
       await humanLogin(page, seedData.syndic_email, seedData.syndic_password);
       // ... cloture ...
     });
   });
   ```
   
   ### Scenarios a reecrire
   
   | Scenario | Seed utilise | Roles | Status actuel |
   |----------|-------------|-------|--------------|
   | meeting-vote | `ag_with_quorum` | Syndic -> Owner -> Syndic | Echoue (quorum) |
   | sel-exchange | `sel_marketplace` | Alice -> Bob | Echoue (page) |
   | poll-vote | `poll_active` | Syndic -> Owner -> Syndic | Echoue (page) |
   | payment-method | `ticket_workflow` (avec owner) | Owner | Echoue (link) |
   | notice-board | `notice_board` | Syndic -> Owner | Echoue (page) |
   | quote-comparison | `quote_comparison` | Syndic | Echoue (page) |
   
   ### Scenarios a corriger (passent mais roles incorrects)
   
   | Scenario | Correction |
   |----------|-----------|
   | ticket-lifecycle | Owner devrait creer le ticket (pas syndic) |
   | expense-approval | OK (syndic correct) |
   | budget-workflow | OK (syndic correct) |
   
   ### Definition of Done
   
   - [ ] 12/12 scenarios passent
   - [ ] Chaque scenario utilise un seed + teardown
   - [ ] Les roles sont metier-corrects (Owner vote, Syndic gere)
   - [ ] Les videos sont exploitables comme documentation YouTube
   - [ ] Le narratif BDD et E2E est identique pour chaque workflow

.. raw:: html

   </div>

