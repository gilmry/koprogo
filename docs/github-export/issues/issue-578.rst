====================================================================
Issue #578: [Story 4.3] Minutes (PV) + 2 signatures eIDAS qualifiées
====================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: javascript,track:software rust,legal-compliance governance,maury track-h-conformite,slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/578>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.3 — `Minutes` (PV) + 2 signatures eIDAS qualifiées
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.3-minutes-eidas-signatures` · Refs: #556
   
   ## Goal
   
   Aggregate `Minutes` + 2 signatures (président + secrétaire) eIDAS qualifié. Refus `Meeting.complete()` sans 2 signatures (cf. 4.5).
   
   ## Contexte Maury
   
   - **FR/INV** : FR16 ; INV-20
   - **Effort** : M
   - **Deps** : Story 4.4 (adapter signature)
   - **ADR refs** : ADR-0014
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Président signe PV via eID belge → secrétaire signe via itsme → Meeting.complete() OK
   - **@edge** : 1 seul signataire → Meeting reste InProgress, attente 2ème
   - **@security** : Tentative signature par owner non-président/secrétaire → 403
   - **@negative** : Signature invalide (eIDAS rejet) → 422 + détail erreur ; PV vide → 422
   
   ## data-testid
   
   `minutes-pdf-preview`, `minutes-sign-president`, `minutes-sign-secretary`, `minutes-status-badge`
   
   ## Files
   
   - `backend/migrations/20260610_030000_create_minutes.sql` + DOWN
   - `backend/src/domain/entities/minutes.rs`
   - `backend/src/application/use_cases/minutes_use_cases.rs`
   - `frontend/src/lib/components/governance/MinutesSigning.svelte`
   - `backend/tests/features/minutes_signatures.feature`
   
   ## Definition of Done
   
   - [ ] Entité Minutes + 2 signatures eIDAS qualifiées
   - [ ] Use-cases create + sign (président/secrétaire)
   - [ ] MinutesSigning.svelte avec boutons signature
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.3
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

