===================================================================
Issue #562: [Story 2.1] Entité Portfolio backend + CRUD /portfolios
===================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust maury,track-h-conformite slice-2
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/562>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 2.1 — Entité `Portfolio` backend + CRUD `/portfolios`
   
   > Maury Phase 6 Exécution · Slice 2 · Story `story/2.1-portfolio-backend-entity` · Refs: #556
   
   ## Goal
   
   Tables `portfolios` + `portfolio_buildings` + `portfolio_shares` + entité domain + use-cases + handlers + migration SQL. Portefeuille = **entité backend** (vs UI préférence localStorage) — cf. ADR-0011.
   
   ## Contexte Maury
   
   - **FR/INV** : FR36 ; mémoire [[koprogo-modular-toolbox]] (favoris star)
   - **Effort** : M
   - **Deps** : Story 1.1 (`acp_id` existe)
   - **ADR refs** : **ADR-0011** (Portefeuille entité backend)
   - **Cluster coord** : NEW → AppError natif
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic crée portfolio "Mes immeubles favoris" → ajoute 3 buildings (1 star, 2 normaux) → list buildings dans portfolio retourne 3, star d'abord
   - **@edge** : Portfolio vide (0 building) → autorisé, retourne `[]`
   - **@security** : Gestionnaire cabinet B tente accès portfolio cabinet A → 403 `AcpNotInScope` ; user non partagé tente GET portfolio partagé d'autre user → 403
   - **@negative** : POST portfolio sans `name` → 422 ; ajout building inexistant → 404 typé
   
   ## data-testid
   
   `portfolio-create-submit`, `portfolio-add-building`, `portfolio-share-submit`, `portfolio-toggle-favorite-{{id}}`
   
   ## Files
   
   - `backend/src/domain/entities/portfolio.rs`
   - `backend/src/application/ports/portfolio_repository.rs`
   - `backend/src/application/use_cases/portfolio_use_cases.rs`
   - `backend/src/infrastructure/database/repositories/portfolio_repository_impl.rs`
   - `backend/src/infrastructure/web/handlers/portfolio_handlers.rs`
   - `backend/migrations/20260601_050000_create_portfolios.sql` + DOWN
   - `backend/tests/features/portfolio.feature`
   
   ## Definition of Done
   
   - [ ] Entité `Portfolio` + `PortfolioBuilding` (M:N) + `PortfolioShare`
   - [ ] Port trait + use-cases CRUD + share + add_building (favorite flag)
   - [ ] Adapter PostgreSQL + handlers Actix
   - [ ] Migration SQL + DOWN testable
   - [ ] BDD 4-cat `portfolio.feature` VERT
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §4 Story 2.1
   - Architecture ADR-0011 : [`docs/maury/refonte-ux-multi-role-acp/architecture.md`](docs/maury/refonte-ux-multi-role-acp/architecture.md) §4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

