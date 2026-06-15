---
feature: refonte-ux-multi-role-acp/track-h-bloqueurs
phase: index
status: SIGNED v1.0 par @gilmry 2026-06-15 — Phase 6 exécution débloquée
date: 2026-06-15
authors: [Claude Opus 4.7 (drafting), @gilmry (signature 2026-06-15)]
---

# Track H — Bloqueurs légaux v0.1.0 — BMAD index

Cible : livrer les 3 invariants légaux **bloqueurs go-live v0.1.0** identifiés dans `docs/WBS_GO_LIVE_v0.1.0.md` Track H (WP-H1 / WP-H2 / WP-H3) en respectant la méthode Maury fullstack-first (cf. mémoire `feedback_maury-fullstack-first.md`).

## Trigger

- Issues GH ouvertes : [#553](https://github.com/gilmry/koprogo/issues/553) (admin garant conformité + drift total_units/quotas) et [#554](https://github.com/gilmry/koprogo/issues/554) (AG transition Completed invariants).
- WBS Track H bloqueurs légaux non-fait à 100% (cartographie 2026-06-15) :
  - WP-H1 = 90% (domain `is_conformant()` + filtrage + ConformityBadge FE OK ; **`BuildingNotConformantError` typée + mapping 422 ABSENT**).
  - WP-H2 = 0% (pre-checks `assert_conformant()?` dans 4 use-cases absents : call_for_funds, expense, charge_distribution, etat_date).
  - WP-H3 = ~20% (`Meeting.complete()` état machine OK ; **`assert_can_complete()` invariants Art. 3.87 §3-5 ABSENTS** — convocations, votes clôturés, présents, quorum, PV).
- WP-B4 (issue #553 Bug 1) = **DÉJÀ COMPLET** — handler `PUT /buildings/{id}` + Svelte 5 binding `on:click` correct (cf. cartographie 2026-06-15). Une seule action requise : ajouter regression spec Playwright.
- Stories Maury 1.4 / 4.5 / 4.9 signées 2026-05-20 mais écrites BE-d'abord (avant leçon `feedback_maury-fullstack-first.md`). Ce Track H complète FE en simultané.

## Évolution méthode Maury appliquée

Voir mémoire **`feedback_maury-fullstack-first.md`** : les 3 stories H1/H2/H3 incluent **FE+BE dès le brief**, pas seulement BE-d'abord avec dette FE différée.

## Index des documents (ordre de lecture Maury TOGAF)

1. **`brief.md`** — Phase A (Vision TOGAF). Personas concernés (Admin garant conformité, Syndic exploitant, Owner constatant). Capacités CB-H1 à CB-H3. Invariants INV-H1-9 (admin-conform, validate-before-compute, AG can_complete, error typée, FE banner, audit, 4-cat, append-only, rollback). Critères succès SCB1-SCB6. Hors-scope. Risques. Signé @gilmry → débloque PRD.
2. **`prd.md`** — Phase B (Business architecture). FR-H1 (admin-conform error typée + 422) / FR-H2 (validate-before-compute 4 use-cases) / FR-H3 (Meeting.assert_can_complete invariants Art. 3.87 §3-5) / FR-B4 (regression spec). Goals métier + User Journey narratif + AC business + NFR. Signé → débloque Architecture.
3. **`architecture.md`** — Phase C (Application + Data). Pattern Error typée → AppError → 422 ; pattern `assert_conformant()?` en début de use-case ; pattern `Meeting::assert_can_complete()` avec `MissingInvariant` vec ; FE banner pattern + bouton disabled + toast 422 ; tests 4-cat. Signé → débloque Stories.
4. **`stories.md`** — Phase D (Stories). 4 stories (H1, H2, H3, B4) Maury-grade self-contained briefables agent. Chaque story = Goal + parent CC/Maury + user journey + AC 4-cat détaillées + Files exhaustifs + a11y checklist + wireframe + notes anti-pattern + cluster coord BE+FE. Signé → agents Track H peuvent être lancés.

## Mapping WBS

Track H du **`docs/WBS_GO_LIVE_v0.1.0.md`** (lignes 131-148, 167-170) :

| Story Track H | WP WBS | Wave | Cartographie |
|---|---|---|---|
| H1 — `BuildingNotConformantError` + 422 mapping | WP-H1 | V1 BE | 90% fait, gap erreur typée |
| H2 — `validate-before-compute` 4 use-cases | WP-H2 | V2 BE (deps H1) | 0% fait |
| H3 — `Meeting.assert_can_complete()` Art. 3.87 | WP-H3 | V2 BE | 20% fait |
| B4 — regression spec Modifier immeuble | WP-B4 | V1 FE | déjà OK, juste spec |

**Wall-clock estimé** : 3-4 jours critical path (H1 → H2 deps → FE wave), 2-5j range selon parallélisme docker (cf. mémoire `docker-parallelism-bottleneck` : 1 BE + 3 FE OK).

## Gates de signature (workflow Maury)

```
brief.md (Mary)
   ↓ @gilmry sign
prd.md (John)
   ↓ @gilmry sign
architecture.md (Winston)
   ↓ @gilmry sign
stories.md (Bob)
   ↓ @gilmry sign
agents Track H briefés depuis stories.md
   ↓ DoD-H1..H3 atteints (cf. stories.md)
CI verte feature/dev sans testIgnore Track H
   ↓
convergence Gate G1 (revue humaine fraîche)
   ↓
v0.1.0 bêta-ready (Tier 1 humain promotion feature/dev → dev)
```

## Mémoires Maury appliquées

- `feedback_maury-fullstack-first` — leçon principale (FE+BE même brief).
- `admin-publishes-conform-buildings` — admin garant conformité (#553).
- `validate-before-compute` — aucun calcul opérationnel sur entités non conformes (#553).
- `world-model-seed` — seeds via use-cases (cf. tests BDD H2 nécessitent buildings conformes + non-conformes).
- `no-f64-in-money` — `Decimal` strict pour quotas (cf. WP-H1 domain).
- `validate-before-compute` — pattern d'application (#553).
- `tdd-bdd-four-categories` — RED-first @happy + @edge + @security + @negative.
- `data-testid-systematic` — banner conformance + toast 422 testables stables.
- `a11y-wcag-aa-baseline` — banner contrast + ARIA live region.
- `multirole-narrative-scenarios` — H3 implique syndic + owner.

## Statut de signature

```
brief.md          : SIGNED v1.0 par @gilmry 2026-06-15
prd.md            : SIGNED v1.0 par @gilmry 2026-06-15
architecture.md   : SIGNED v1.0 par @gilmry 2026-06-15
stories.md        : SIGNED v1.0 par @gilmry 2026-06-15
validation.md     : SIGNED v1.0 par @gilmry 2026-06-15 (Phase E TOGAF F)
WBS Track H       : déjà mappé (lignes 131-148 + 167-170 WBS)
```

**→ Agents Track H autorisés à être briefés et lancés selon le Gantt waves V1 (H1) → V2 // (H2 BE + H3 BE puis FE) → V3 (B4). Cf. `stories.md §Gantt`.**
