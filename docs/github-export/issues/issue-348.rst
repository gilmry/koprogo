===========================================================================
Issue #348: BDD multi-roles: aligner les features sur les specs (etape 3/5)
===========================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: priority:high,legal-compliance testing
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/348>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Etape 3 : Aligner les BDD sur les specs multi-roles
   
   Parent : #345
   Depend de : #346 (specs), #347 (seeds)
   
   ### Objectif
   
   Enrichir les feature files BDD existantes avec des scenarios de **workflow complet multi-roles** qui reprennent exactement le narratif des specs formalisees en etape 1.
   
   ### Principe d'alignement BDD <-> E2E
   
   Le meme scenario metier est la **source de verite unique** :
   ```
   Spec (docs/specs/vote-ag.rst)
     | traduit en Gherkin
   BDD (backend/tests/features/vote_ag_workflow.feature)
     | meme narratif
   E2E (frontend/tests/e2e/scenarios/meeting-vote.scenario.ts)
   ```
   
   Si le BDD passe mais le E2E echoue -> bug frontend.
   Si les deux echouent -> probleme de spec/backend.
   
   ### Scenarios BDD a ajouter/enrichir
   
   1. **vote_ag_workflow.feature** (NOUVEAU) :
      ```gherkin
      Feature: Workflow complet de vote en AG
        As a syndic and co-owners
        We want to create, vote, and close resolutions
        So that AG decisions comply with Belgian law
   
        Scenario: AG complete avec quorum et vote multi-owner
          Given a building with 3 owners (Alice 300, Bob 200, Charlie 500 tantiemes)
          And a second convocation meeting (Art. 3.87 par.5 — no quorum needed)
          And the syndic has created a resolution "Travaux facade" with Simple majority
          When Alice votes "Pour" with 300 tantiemes
          And Bob votes "Contre" with 200 tantiemes
          And Charlie votes "Pour" with 500 tantiemes
          And the syndic closes the voting
          Then the resolution should be "Adopted" (800 Pour vs 200 Contre)
      ```
   
   2. **ticket_workflow.feature** (NOUVEAU) — Owner cree -> Syndic assigne -> resolution
   3. **sel_workflow.feature** (NOUVEAU) — Alice offre -> Bob demande -> completion -> rating
   4. **age_request_workflow.feature** (ENRICHIR age_requests.feature) — seuil 1/5 -> syndic
   5. **poll_workflow.feature** (NOUVEAU) — Syndic publie -> Owner vote -> cloture
   6. **expense_approval_workflow.feature** (ENRICHIR expenses.feature)
   7. **convocation_attendance.feature** (ENRICHIR convocations.feature)
   8. **notice_board_workflow.feature** (NOUVEAU) — publication + lecture
   
   ### Regles
   
   - Chaque feature file de workflow reference l'article du CC dans le commentaire
   - Les Background utilisent les seeds de l'etape 2
   - Les personas (Alice, Bob, Charlie) sont coherentes entre BDD et E2E
   - Les steps multi-role utilisent `When "Alice" votes` / `When the syndic closes`
   
   ### Definition of Done
   
   - [ ] 5 nouveaux feature files + 3 enrichis
   - [ ] Chaque scenario reference l'article CC
   - [ ] Les step definitions supportent le multi-acteur
   - [ ] Tous les scenarios passent avec `cargo test --test bdd`
   - [ ] Le narratif est identique aux specs docs/specs/

.. raw:: html

   </div>

