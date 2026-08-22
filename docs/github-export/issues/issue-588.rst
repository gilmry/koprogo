=======================================================================
Issue #588: [Story 5.4] Reservation.on_behalf_of_acp (exception syndic)
=======================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: javascript,track:software rust,community maury,track-h-conformite slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/588>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.4 — `Reservation.on_behalf_of_acp` (exception syndic)
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.4-reservation-on-behalf` · Refs: #556
   
   ## Goal
   
   Champ `Reservation.on_behalf_of_acp: bool` + motif obligatoire si true. Syndic autorisé à réserver pour AG/prestataires sous cette flag.
   
   ## Contexte Maury
   
   - **FR/INV** : FR27 ; INV-5
   - **Effort** : S
   - **Deps** : Story 5.3
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic réserve salle commune `on_behalf_of_acp=true motif="AG annuelle"` → OK + log spécifique
   - **@edge** : Syndic réserve `on_behalf_of_acp=false` → 403 (participation perso interdite)
   - **@security** : Owner ne peut pas mettre `on_behalf_of_acp=true` (réservé syndic)
   - **@negative** : `on_behalf_of_acp=true` sans motif → 422
   
   ## data-testid
   
   `reservation-on-behalf-toggle`, `reservation-motif-input`, `reservation-submit`
   
   ## Files
   
   - `backend/src/domain/entities/reservation.rs` (refacto)
   - `backend/src/application/use_cases/reservation_use_cases.rs` (refacto)
   - `frontend/src/lib/components/community/ReservationCreate.svelte` (refacto)
   - `backend/tests/features/reservation_on_behalf.feature`
   
   ## Definition of Done
   
   - [ ] Reservation.on_behalf_of_acp + motif (obligatoire si true)
   - [ ] Use-case + UI toggle syndic-only
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

