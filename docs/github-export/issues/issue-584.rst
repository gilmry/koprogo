=================================================================================================
Issue #584: [Story 4.9] [cluster-coord] MÉGA validate-before-compute (4 use-cases) — couvre WP-H2
=================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: javascript,track:software priority:high,rust finance,legal-compliance maury,track-h-conformite cluster-coord,slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/584>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.9 — `validate-before-compute` sur calculs accounting `[cluster-coord]` MÉGA
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.9-validate-before-compute` · Refs: #556 · Coord cluster #433 + #555 méga (4 use-cases × 2 migrations atomiques)
   
   ## Goal
   
   Tout use-case calcul (charges/répartition/quorum/appels de fonds/PV) commence par `building.assert_conformant()?`. Refus 422 `BuildingNotConformant{reason}` sinon. Pattern aligné mémoire [[validate-before-compute]]. **Couvre WP-H2 WBS Track H.**
   
   ## Contexte Maury
   
   - **FR/INV** : FR22 ; mémoire [[validate-before-compute]], brief C-brief
   - **Effort** : L
   - **Deps** : Story 1.4 (is_conformant)
   - **ADR refs** : ADR-0010
   - **Cluster coord** : **`[cluster-coord]` MÉGA — #433 + #555 simultané sur 4 use-cases** (PR regroupée pour préserver l'invariant validate-before-compute atomique)
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Building conforme → calculs charges/répartition OK + résultats Decimal exacts
   - **@edge** : Building avec 999/1000 millièmes (delta 0.0001) → 422 sans tolérance (mémoire #433)
   - **@security** : Audit immuable INV-24 : tentative calcul sur building non-conforme → log + alerte
   - **@negative** : Calcul SUM tantièmes sur 0 unit → renvoie Decimal(0), pas NaN ; FE affiche `—`
   
   ## data-testid
   
   `charge-distribution-error-banner`, `call-for-funds-error-banner`
   
   ## Files
   
   - `backend/src/application/use_cases/expense_use_cases.rs` (pre-check + Decimal + AppError)
   - `backend/src/application/use_cases/call_for_funds_use_cases.rs` (idem)
   - `backend/src/application/use_cases/charge_distribution_use_case.rs` (idem)
   - `backend/src/application/use_cases/etat_date_use_cases.rs` (idem)
   - `backend/tests/features/validate_before_compute.feature`
   - `frontend/src/lib/components/expenses/ExpenseList.svelte` (banner)
   
   ## Definition of Done
   
   - [ ] 4 use-cases : pré-check `building.assert_conformant()?` ajouté
   - [ ] 4 use-cases : migration Decimal simultanée (cluster #433)
   - [ ] 4 use-cases : migration AppError simultanée (cluster #555)
   - [ ] BDD 4-cat VERT pour chaque use-case
   - [ ] FE banner non-conformant + désactivation boutons calcul
   - [ ] PR `[cluster-coord]` MÉGA : 4 use-cases × 2 migrations = 1 PR atomique pour préserver invariant
   - [ ] Couvre WP-H2 Track H WBS
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.9
   - WBS WP-H2 : [`docs/WBS_GO_LIVE_v0.1.0.md`](docs/WBS_GO_LIVE_v0.1.0.md) Track H
   - Cluster Decimal : #433 · Cluster Result : #555 · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

