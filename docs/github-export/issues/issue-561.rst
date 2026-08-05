===================================================================================================================
Issue #561: [Story 1.4] [cluster-coord] Building.is_conformant() + fiche immeuble correcte (closes #553 Bugs 1/3/4)
===================================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software priority:high,rust legal-compliance,governance maury,track-h-conformite slice-1,cluster-coord
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/561>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 1.4 — `Building.is_conformant()` + fiche immeuble correcte (`[cluster-coord]` #433 Decimal)
   
   > Maury Phase 6 Exécution · Slice 1 · Story `story/1.4-building-is-conformant` · Refs: #556 · Closes #553 (Bugs 1/3/4) · Coord cluster #433
   
   ## Goal
   
   Méthode domain `Building.is_conformant() -> bool` (`count_units == total_units && SUM(quotas) == Decimal(1000)`). Fiche immeuble admin/syndic affiche count réel + somme réelle des quotas + badge conformité + delta. **Résout #553 Bugs 1/3/4** (modal modifier admin + total tantièmes NaN/dérive).
   
   ## Contexte Maury
   
   - **FR/INV** : FR9, FR11, FR12 ; INV-1, mémoire [[admin-publishes-conform-buildings]]
   - **Effort** : M
   - **Deps** : Story 1.1, Story 1.2
   - **ADR refs** : ADR-0010 (ACP racine), ADR-0012 (data-testid)
   - **Cluster coord** : **`[cluster-coord]` #433 Decimal simultané** — les quotas migrent vers `rust_decimal::Decimal` dans la **même PR** que la refonte ; côté FE plus de `parseFloat`/`Number()`, somme vérifiée Decimal-equivalent
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Building 50/50 units + `SUM quotas == 1000` (Decimal exact) → `is_conformant() == true` + badge vert UI
   - **@edge** : Building 999/1000 millièmes (1 millième manquant) → non-conformant ; **pas de tolérance arrondi** (cluster #433 strict)
   - **@security** : Syndic ne peut publier (rendre visible) un building non-conformant ; admin garde la conformité (mémoire `admin-publishes-conform-buildings`)
   - **@negative** : `count_units == 0` → fiche affiche `—` et **non NaN** (#553 Bug 3) ; pas de `panic!` ni `unwrap()`
   
   ## data-testid
   
   `building-conformity-badge`, `building-units-count`, `building-quota-sum`, `building-quota-delta`, `building-edit-submit`
   
   ## Files
   
   - `backend/src/domain/entities/building.rs` (ajout `is_conformant`, `units_count`, `quota_sum` Decimal)
   - `backend/src/application/dto/building_dto.rs` (sérialise count réel + sum Decimal-as-string)
   - `backend/src/application/use_cases/*.rs` (use-cases touchés migrent `f64 → Decimal` simultané)
   - `frontend/src/lib/components/buildings/ConformityBadge.svelte` (NEW)
   - `frontend/src/lib/components/buildings/BuildingDetail.svelte` (refacto)
   - `frontend/src/lib/components/buildings/__tests__/ConformityBadge.test.ts` (Vitest RED-GREEN-BLUE)
   - `frontend/tests/e2e/refonte-ux/slice-1-acp-refacto/building-conformity.spec.ts`
   
   ## Definition of Done
   
   - [ ] `Building.is_conformant()` méthode domain testée
   - [ ] `quota_sum` calculé via `SUM(units.quota)` côté backend (Decimal)
   - [ ] DTO sérialise quotas Decimal-as-string (cf. ADR-0007/0008)
   - [ ] Aucun `parseFloat`/`Number()` sur quotas FE (grep VERT)
   - [ ] `ConformityBadge.svelte` créé avec Vitest 4-cat
   - [ ] `BuildingDetail.svelte` affiche count réel + somme réelle + delta + badge
   - [ ] data-testid présents et utilisés par spec Playwright
   - [ ] BDD 4-cat backend + Vitest 4-cat FE + Playwright E2E VERTS
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] Closes #553 Bugs 1, 3, 4 (modal admin + total tantièmes)
   - [ ] PR `[cluster-coord]` étiquetée — 2 migrations simultanées (refonte + Decimal #433)
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §3 Story 1.4
   - Bug d'origine : #553
   - Cluster Decimal umbrella : #433
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

