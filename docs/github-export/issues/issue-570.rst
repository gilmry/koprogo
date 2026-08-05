=============================================================================================
Issue #570: [Story 3.4] Entité Mandate (avocat/notaire/AMO/architect/BET) + workflow émission
=============================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust legal-compliance,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/570>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.4 — Entité `Mandate` (avocat/notaire/AMO/architect/BET) + workflow émission
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.4-mandate-entity` · Refs: #556
   
   ## Goal
   
   Table `mandates` + entité + use-cases issue + workflow émission (syndic OU décision AG selon kind) + audit immuable. Refus 403 si `valid_until < now()`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR7 ; INV-14
   - **Effort** : M
   - **Deps** : Story 1.1
   - **ADR refs** : —
   - **Cluster coord** : NEW → AppError natif
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic émet Mandate notaire pour Building X (état daté) avec valid_until=2026-12-31 → Mandate persistée + audit_event
   - **@edge** : Mandate juste à `valid_until` (1 seconde avant) → autorisée pour action ; après → 403 expired
   - **@security** : Notaire mandaté sur Unit Y tente accès Unit Z → 403 `AcpNotInScope` ; expiré → 403 `MandateExpired`
   - **@negative** : POST `/mandates` sans `valid_until` → 422 (champ obligatoire)
   
   ## data-testid
   
   `mandate-issue-submit`, `mandate-kind-select`, `mandate-valid-until-input`
   
   ## Files
   
   - `backend/src/domain/entities/mandate.rs`
   - `backend/src/application/ports/mandate_repository.rs`
   - `backend/src/application/use_cases/mandate_use_cases.rs`
   - `backend/src/infrastructure/database/repositories/mandate_repository_impl.rs`
   - `backend/migrations/20260605_020000_create_mandates.sql` + DOWN
   - `backend/tests/features/mandate.feature`
   
   ## Definition of Done
   
   - [ ] Entité Mandate avec kind enum (lawyer/notary/amo/architect/bet) + valid_until NOT NULL
   - [ ] Use-case issue + check valid_until (refus expired)
   - [ ] Endpoint POST /mandates + GET filtré scope
   - [ ] Migration SQL + DOWN
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

