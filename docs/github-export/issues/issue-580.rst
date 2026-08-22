===================================================================================
Issue #580: [Story 4.5] [cluster-coord] Meeting.assert_can_complete() — closes #554
===================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust legal-compliance,governance maury,track-h-conformite cluster-coord,slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/580>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.5 — `Meeting.assert_can_complete()` (reprise #554) `[cluster-coord]`
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.5-meeting-assert-can-complete` · Refs: #556 · Closes #554 · Coord cluster #555
   
   ## Goal
   
   Méthode domain `assert_can_complete()` qui vérifie : convocations envoyées + quorum atteint + résolutions clôturées + PV signé 2×. Refus 422 `MeetingNotCompletable{missing:[...]}` sinon.
   
   ## Contexte Maury
   
   - **FR/INV** : FR17 ; #554, brief C-brief
   - **Effort** : M
   - **Deps** : Story 4.1, 4.2, 4.3
   - **ADR refs** : —
   - **Cluster coord** : **`[cluster-coord]` #555 simultané** (Result<_, String> legacy à migrer dans meeting_use_cases)
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Meeting avec toutes pré-conditions → complete() → status Completed + audit
   - **@edge** : Meeting avec PV signé 2× mais 1 résolution encore ouverte → 422 avec missing=["resolution_R-42_not_closed"]
   - **@security** : Tentative complete() par owner non-syndic → 403
   - **@negative** : Tentative complete() sur Meeting déjà Completed → 422 `MeetingAlreadyCompleted`
   
   ## data-testid
   
   `meeting-complete-submit`, `meeting-missing-checklist-{{key}}`
   
   ## Files
   
   - `backend/src/domain/entities/meeting.rs` (ajout `assert_can_complete`)
   - `backend/src/application/use_cases/meeting_use_cases.rs` (refacto + AppError)
   - `backend/tests/features/meeting_complete.feature`
   
   ## Definition of Done
   
   - [ ] Méthode `assert_can_complete()` testée
   - [ ] Use-case complete_meeting refacto → AppError (migration #555 simultanée)
   - [ ] BDD 4-cat VERT incl. @edge avec missing typés
   - [ ] FE bouton désactivé + checklist visuelle
   - [ ] PR `[cluster-coord]` étiquetée
   - [ ] Closes #554 Bug 1
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.5
   - #554 · Cluster Result : #555 · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

