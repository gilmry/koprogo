==============================================================================
Issue #568: [Story 3.2] Entité MagicLink + endpoint + page publique /c/<token>
==============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust security,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/568>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.2 — Entité `MagicLink` + endpoint + page publique `/c/<token>`
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.2-magic-link-entity` · Refs: #556
   
   ## Goal
   
   Table `magic_links` + entité + use-cases issue/validate_and_consume + endpoint `POST /magic-links` (syndic) + page Astro publique `/c/[token]` qui résout le scope (ticket/quote/invoice/evaluation).
   
   ## Contexte Maury
   
   - **FR/INV** : FR6 ; INV-13, INV-17
   - **Effort** : M
   - **Deps** : Story 1.1
   - **ADR refs** : —
   - **Cluster coord** : NEW → AppError natif
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic POST `/magic-links {subject_user_id, scope_kind=ticket, scope_id, expires_in=7d}` → token signé → contractor ouvre `/c/<token>` → voit le ticket
   - **@edge** : Token à exactement `expires_at` (1 seconde près) → autorisé ; consommation single_use seconde fois → 403
   - **@security** : Token forgé/altéré (HMAC invalide) → 403 `MagicLinkInvalid` ; tentative `/c/<token-other-scope>` → 403
   - **@negative** : Token expiré → 403 typé avec message "lien expiré, demandez-en un nouveau au syndic"
   
   ## data-testid
   
   `magic-link-issue-submit`, `magic-link-target-input`, `c-page-ticket-content`, `c-page-respond-submit`
   
   ## Files
   
   - `backend/src/domain/entities/magic_link.rs`
   - `backend/src/application/ports/magic_link_repository.rs`
   - `backend/src/application/use_cases/magic_link_use_cases.rs`
   - `backend/src/infrastructure/database/repositories/magic_link_repository_impl.rs`
   - `backend/src/infrastructure/web/handlers/magic_link_handlers.rs`
   - `backend/migrations/20260605_010000_create_magic_links.sql` + DOWN
   - `frontend/src/pages/c/[token].astro` (NEW)
   - `backend/tests/features/magic_link.feature`
   
   ## Definition of Done
   
   - [ ] Entité MagicLink avec token_hash scrypt + HMAC signing
   - [ ] Use-cases issue + validate_and_consume (1-shot single_use)
   - [ ] Endpoint POST /magic-links (syndic) + page publique /c/[token]
   - [ ] Migration SQL + DOWN
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.2
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

