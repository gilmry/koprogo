=============================================================================
Issue #574: [Story 3.8] TechnicalSpec versionnable (cahier des charges signé)
=============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust maintenance,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/574>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.8 — `TechnicalSpec` versionnable (cahier des charges signé)
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.8-technical-spec-versionable` · Refs: #556
   
   ## Goal
   
   Entité `TechnicalSpec` avec versionning semver + signatures multi-parties (ACP/Syndic/AMO) + attachements documents.
   
   ## Contexte Maury
   
   - **FR/INV** : FR33 ; brief C16
   - **Effort** : M
   - **Deps** : Story 3.4 (Mandate AMO)
   - **ADR refs** : ADR-0014 (signatures)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic crée TechnicalSpec v1.0.0 → AMO signe → status Approved → versionnable v1.1.0 si modif
   - **@edge** : Spec v2.0.0 majeure → signatures précédentes invalidées + re-signature requise
   - **@security** : Owner non-mandaté ne peut pas signer ; signature par tiers → 403
   - **@negative** : Tentative création spec avec scope mal défini (deliverables vides) → 422
   
   ## data-testid
   
   `tech-spec-create-submit`, `tech-spec-version-input`, `tech-spec-sign-submit`, `tech-spec-attach-upload`
   
   ## Files
   
   - `backend/migrations/20260605_060000_create_technical_specs.sql` + DOWN
   - `backend/src/domain/entities/technical_spec.rs`
   - `backend/src/application/use_cases/technical_spec_use_cases.rs`
   - `backend/tests/features/technical_spec.feature`
   
   ## Definition of Done
   
   - [ ] Entité TechnicalSpec avec semver + signatures multi-parties
   - [ ] Use-case create + sign + version_bump (major invalide signatures)
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.8
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

