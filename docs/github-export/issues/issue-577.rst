================================================================================
Issue #577: [Story 4.2] Vote distant auth_method strong (itsme/eID) — closes #48
================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: javascript,track:software rust,security legal-compliance,governance maury,track-h-conformite slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/577>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.2 — Vote distant `auth_method` strong (#48 itsme/eID)
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.2-vote-auth-strong` · Refs: #556 · Closes #48
   
   ## Goal
   
   Extension `votes.auth_method` enum (presence|proxy|itsme|eid) + refus 403 si mode meeting=remote/hybrid sans auth strong.
   
   ## Contexte Maury
   
   - **FR/INV** : FR15 ; INV-18, #48
   - **Effort** : M
   - **Deps** : Story 4.1
   - **ADR refs** : ADR-0014 (signature électronique)
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Owner vote distant avec itsme → vote enregistré avec `auth_method=itsme`
   - **@edge** : Owner tente vote distant avec proxy (procuration distance) → autorisé sous conditions Art. 3.87 §4
   - **@security** : Owner tente vote distant avec auth_method=presence → 403 `VoteAuthInsufficient`
   - **@negative** : Vote sans auth_method → 422
   
   ## data-testid
   
   `vote-auth-method-select`, `vote-itsme-button`, `vote-eid-button`, `vote-cast-submit`
   
   ## Files
   
   - `backend/migrations/20260610_020000_extend_votes_auth_method.sql` + DOWN
   - `backend/src/domain/entities/vote.rs`
   - `backend/src/application/use_cases/vote_use_cases.rs`
   - `frontend/src/lib/components/governance/VoteCast.svelte` (refacto)
   - `backend/tests/features/vote_remote_auth.feature`
   
   ## Definition of Done
   
   - [ ] Migration votes.auth_method + DOWN
   - [ ] Use-case vote_cast valide auth_method vs Meeting.mode
   - [ ] VoteCast.svelte refacto avec boutons itsme/eID
   - [ ] BDD 4-cat VERT
   - [ ] Closes #48 (itsme/eID promu in-scope)
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.2
   - #48 itsme/eID · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

