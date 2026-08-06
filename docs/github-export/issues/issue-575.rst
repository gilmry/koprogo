============================================================================
Issue #575: [Story 3.9] ContractorEvaluation (refuse 422 sans TechnicalSpec)
============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust maintenance,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/575>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.9 — `ContractorEvaluation` (refuse 422 sans TechnicalSpec)
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.9-contractor-evaluation` · Refs: #556
   
   ## Goal
   
   Entité `ContractorEvaluation` qui nécessite `TechnicalSpec` préalable (refus 422 `TechnicalSpecRequired` sinon). Lien `tickets_linked[]` vers plaintes ayant motivé l'éval.
   
   ## Contexte Maury
   
   - **FR/INV** : FR34, FR35 ; INV-21, INV-24, brief C18
   - **Effort** : M
   - **Deps** : Story 3.8
   - **ADR refs** : ADR-0014 (signature évaluation optionnelle)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Évaluation Contractor X référençant TechnicalSpec v1.0.0 + 2 tickets liés → scores 1-5 enregistrés + audit
   - **@edge** : Évaluation pile à expiration de TechnicalSpec (1 sec avant) → autorisée
   - **@security** : ContractorEvaluation append-only (INV-24) ; tentative édit → 403
   - **@negative** : Évaluation sans TechnicalSpec → 422 `TechnicalSpecRequired` ; Contractor inexistant → 404
   
   ## data-testid
   
   `contractor-eval-submit`, `contractor-eval-spec-select`, `contractor-eval-tickets-link`, `contractor-eval-scores-{{criterion}}`
   
   ## Files
   
   - `backend/migrations/20260605_070000_create_contractor_evaluations.sql` + DOWN
   - `backend/src/domain/entities/contractor_evaluation.rs`
   - `backend/src/application/use_cases/contractor_evaluation_use_cases.rs`
   - `backend/tests/features/contractor_evaluation.feature`
   
   ## Definition of Done
   
   - [ ] Entité ContractorEvaluation append-only avec scores + tickets_linked + spec_id NOT NULL
   - [ ] Use-case create_evaluation avec pré-check TechnicalSpec existante
   - [ ] BDD 4-cat VERT incl. @negative TechnicalSpecRequired
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.9
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

