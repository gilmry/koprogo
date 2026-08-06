===============================================================================
Issue #581: [Story 4.6] Résolution EvaluationContractors AGO auto non retirable
===============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust governance,maury track-h-conformite,slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/581>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.6 — Résolution `EvaluationContractors` AGO auto non retirable
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.6-resolution-auto-evaluation` · Refs: #556
   
   ## Goal
   
   Use-case `generate_ago_resolutions(meeting_id)` ajoute auto une Resolution `kind=EvaluationContractors_AUTO` + `is_auto_generated=true`. Refus 403 `ResolutionAutoNotRemovable` si tentative delete.
   
   ## Contexte Maury
   
   - **FR/INV** : FR18 ; INV-22
   - **Effort** : S
   - **Deps** : Story 4.1
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : AGO créée → use-case lance auto → Resolution `EvaluationContractors_AUTO` présente non-éditable
   - **@edge** : AGE (extraordinaire) → pas de génération auto
   - **@security** : Syndic tente DELETE resolution `EvaluationContractors_AUTO` → 403
   - **@negative** : Modification text resolution AUTO → 403 (immutable)
   
   ## data-testid
   
   `resolution-auto-badge`, `resolution-delete-{{id}}` (caché si AUTO)
   
   ## Files
   
   - `backend/src/application/use_cases/generate_ago_resolutions_use_case.rs` (NEW)
   - `backend/src/domain/entities/resolution.rs` (refacto avec is_auto_generated)
   - `backend/tests/features/resolution_auto_evaluation.feature`
   
   ## Definition of Done
   
   - [ ] Use-case generate_ago_resolutions ajoute auto Resolution EvaluationContractors_AUTO
   - [ ] Resolution.is_auto_generated bool, refus delete/edit
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.6
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

