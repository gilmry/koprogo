===============================================================================
Issue #572: [Story 3.6] Ticket.kind=complaint + severity + evidence + witnesses
===============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software rust,maintenance accessibility,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/572>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.6 — `Ticket.kind=complaint` + severity + evidence + witnesses
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.6-ticket-complaint-extended` · Refs: #556
   
   ## Goal
   
   Extension entité `Ticket` avec `kind` enum (Request|Complaint), `severity` (Low|Normal|High|Critical), `incident_date`, `evidence_attachments[]`, `witnesses[]`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR31 ; brief C17
   - **Effort** : M
   - **Deps** : Story 1.1
   - **ADR refs** : ADR-0012 (data-testid)
   - **Cluster coord** : si use-case ticket touche legacy `Result<_, String>` → migrer #555 simultané
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Owner crée plainte severity=critical avec 3 photos + 2 témoins owners → Ticket persisté + notifications syndic + CdC
   - **@edge** : Plainte sans evidence (text-only) → autorisée mais badge "preuves manquantes"
   - **@security** : Audit immuable INV-24 : 5 min après création, tentative edit → 403 `TicketImmutable`
   - **@negative** : kind=complaint sans severity → 422 ; evidence_attachments > 10 fichiers → 422 ; fichier > 10MB → 422
   
   ## data-testid
   
   `ticket-create-kind-select`, `ticket-severity-select`, `ticket-evidence-upload`, `ticket-witness-add`, `ticket-submit`
   
   ## Files
   
   - `backend/migrations/20260605_040000_extend_tickets_complaint.sql` + DOWN
   - `backend/src/domain/entities/ticket.rs` (refacto)
   - `backend/src/application/dto/ticket_dto.rs`
   - `backend/src/application/use_cases/ticket_use_cases.rs` (refacto)
   - `frontend/src/lib/components/tickets/TicketCreate.svelte` (refacto)
   - `backend/tests/features/ticket_complaint.feature`
   
   ## Definition of Done
   
   - [ ] Migration tickets étendus (kind/severity/incident_date/evidence/witnesses) + DOWN
   - [ ] Entité Ticket refacto avec invariants (5min mutable window puis immutable)
   - [ ] TicketCreate.svelte refacto avec upload + témoins
   - [ ] BDD 4-cat VERT
   - [ ] a11y axe-core VERT sur formulaire
   - [ ] data-testid présents
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.6
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

