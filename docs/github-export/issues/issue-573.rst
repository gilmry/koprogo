===========================================================
Issue #573: [Story 3.7] SyndicResponse + SLA + escalade CdC
===========================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust maintenance,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/573>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.7 — `SyndicResponse` + SLA + escalade CdC
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.7-syndic-response-sla` · Refs: #556
   
   ## Goal
   
   Entité `SyndicResponse` (append-only) + champ `Ticket.sla_due_at` calculé par severity policy + escalade CdC si dépassé.
   
   ## Contexte Maury
   
   - **FR/INV** : FR32 ; INV-23, brief C17
   - **Effort** : M
   - **Deps** : Story 3.6
   - **ADR refs** : —
   - **Cluster coord** : NEW → AppError natif
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic répond < SLA (24h pour critical, 5j pour low) → escalade évitée + notification owner
   - **@edge** : SLA juste à expiration (1 seconde avant) → autorisée ; juste après → escalade créée
   - **@security** : SyndicResponse non éditable (audit immuable) ; CdC reçoit notification escalade mais ne peut pas répondre à la place du syndic
   - **@negative** : Tentative édit response 1h après → 403 `ResponseImmutable`
   
   ## data-testid
   
   `syndic-response-submit`, `syndic-response-action-proposed`, `ticket-sla-badge`
   
   ## Files
   
   - `backend/migrations/20260605_050000_create_syndic_responses.sql` + DOWN
   - `backend/src/domain/entities/syndic_response.rs`
   - `backend/src/application/use_cases/syndic_response_use_cases.rs`
   - `backend/src/infrastructure/jobs/sla_escalation_job.rs` (NEW, cron)
   - `backend/tests/features/syndic_response_sla.feature`
   
   ## Definition of Done
   
   - [ ] Entité SyndicResponse append-only
   - [ ] Use-case respond + calcul sla_due_at par severity policy
   - [ ] Job cron escalade CdC si SLA dépassé
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.7
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

