===================================================================================
Issue #576: [Story 4.1] [cluster-coord] Meeting.mode hybrid + quorum agrégé Decimal
===================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust legal-compliance,governance maury,track-h-conformite cluster-coord,slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/576>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.1 — `Meeting.mode` enum (in_person/remote/hybrid) + quorum agrégé `[cluster-coord]`
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.1-meeting-mode-hybrid` · Refs: #556 · Coord cluster #433 + #555
   
   ## Goal
   
   Extension `meetings.mode` + use-case `compute_quorum` qui agrège `attendees_in_person + remote + proxy` en Decimal.
   
   ## Contexte Maury
   
   - **FR/INV** : FR13, FR14 ; INV-19, brief C15
   - **Effort** : M
   - **Deps** : Story 1.1
   - **ADR refs** : —
   - **Cluster coord** : **`[cluster-coord]` #433 simultané** (quorum Decimal, pas f64) ; **#555 simultané**
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : AG hybride mode=hybrid → 10 présentiels + 5 distants + 3 procurations → quorum agrégé OK selon Decimal somme
   - **@edge** : Quorum à exactement 50.0% (seuil Art. 3.87 §3) → respecté ; 49.99% → refusé
   - **@security** : Mode=remote impose `auth_method` strong (cf. 4.2)
   - **@negative** : Meeting mode=hybrid sans configuration distance (videoconf_url manquant) → 422
   
   ## data-testid
   
   `meeting-mode-select`, `meeting-quorum-current`, `meeting-quorum-required`
   
   ## Files
   
   - `backend/migrations/20260610_010000_extend_meetings_hybrid.sql` + DOWN
   - `backend/src/domain/entities/meeting.rs` (refacto)
   - `backend/src/application/use_cases/compute_quorum_use_case.rs` (refacto + Decimal + AppError)
   - `backend/tests/features/meeting_hybrid_quorum.feature`
   
   ## Definition of Done
   
   - [ ] Migration meetings.mode + videoconf_url + DOWN
   - [ ] Use-case compute_quorum refacto Decimal-strict
   - [ ] PR `[cluster-coord]` : #433 + #555 migrations simultanées
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.1
   - Cluster Decimal : #433 · Cluster Result : #555 · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

