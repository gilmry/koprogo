---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: validation
phase_togaf: F (Migration Planning)
agent_bmad: Product Owner (@gilmry)
authors: [Gilles Maury]
date: 2026-06-15
version: 0.1
status: Draft 0.1 — signature pending @gilmry
brief_source: brief.md (Draft v0.1)
prd_source: prd.md (Draft v0.1)
architecture_source: architecture.md (Draft v0.1)
stories_source: stories.md (Draft v0.1)
parent_feature: docs/maury/refonte-ux-multi-role-acp/validation.md (v1.0 signée 2026-05-20)
---

# Validation Product Owner — Track H Conformité légale copropriété

> Phase F TOGAF. À signer par @gilmry pour débloquer la **passe d'agent CL0** (ADR) puis CL1. Tant que non signé : aucune passe.

## 1. Périmètre soumis

- **Brief** (`brief.md`) — 3 personas, 13 capacités CB-L, 14 invariants INV-L, 10 SCB-L, sources légales sourcées, hors-scope, risques.
- **PRD** (`prd.md`) — FR-CL1..7 par work package, user journeys (groupe d'immeubles, quorum double, suspension vote, fonds réserve, association partielle), AC 4-cat, NFR, matrice.
- **Architecture** (`architecture.md`) — modèle hybride, conformité ACP-level, quorum double, représentant de vote, fonds, critère utilité, associations partielles (quotités 2 niveaux), 9 migrations réversibles, patterns BDD/TDD.
- **Stories** (`stories.md`) — H0-ADR + H4-H17 self-contained, Gantt par passe d'agent (≈ 9 passes chemin critique, ≈ 22 total).

## 2. Décisions PO à confirmer

| Décision | Choix proposé |
|---|---|
| **D1 placement acte de base** | Hybride : `acps.total_tantiemes` + sous-totaux blocs ✅ |
| **D2 périmètre** | Full conformité maintenant ✅ |
| **D3 associations partielles** | In-scope (H16) ✅ |
| **D4 migration units.acp_id** | In-scope (H15) ✅ |
| **D5 multi-titulaires/suspension** | In-scope (H17) ✅ |
| **D6 schéma quotités 2 niveaux** | À trancher en H0-ADR (`units.particular_quota` nullable vs table `unit_quotas`) — **décision PO attendue** |
| **D7 backfill multi-building** | SUM des sous-totaux + WARNING + validation admin manuelle |

## 3. Risques résiduels acceptés (cf. brief §7)

Migration multi-building (mitigée WARNING+admin) ; rétro-compat seeds (stash repris H7/H9) ; quotités 2 niveaux MVP ; `present_quotas` DOUBLE PRECISION (dette ADR-0008 non aggravée) ; cascade `units.acp_id`.

## 4. Autorisation Phase 6 (exécution)

À la signature, PO autorise :
- Démarrage des passes d'agent selon le Gantt (`stories.md` + WBS Track H WP-CL).
- 1 PR par story (sauf H7 wave qui peut être BE+FE) ; gate CI par PR ; sign-off @gilmry par PR.
- ADR(s) H0 en statut `Proposed`, acceptation @gilmry au merge.
- Branches `story/CL<n>-<...>`.
- **Pas de** `git push --force` / `--no-verify` / `gh issue close` autonomes (CRITICAL.md).

## 5. Gate de validation — critères

- [ ] D6 (schéma quotités 2 niveaux) tranchée par @gilmry.
- [ ] Sources légales vérifiables (liens README).
- [ ] Cohérence WBS (WP-CL0-7) ↔ stories (H4-H17) ↔ #618.
- [ ] Track H bloqueurs déjà mergé pris en compte (H1 conservé, H2/H3 retravaillés).

## 6. Signature

```
Mary (Brief)        : Draft v0.1 — pending @gilmry
John (PRD)          : Draft v0.1 — pending @gilmry
Winston (Arch)      : Draft v0.1 — pending @gilmry
Bob (Stories)       : Draft v0.1 — pending @gilmry
PO (Validation)     : Draft v0.1 — pending @gilmry
```

**→ À la signature des 5 docs + arbitrage D6 : passe d'agent CL0 (ADR) autorisée.**

## 7. Refs
- Issue [#618](https://github.com/gilmry/koprogo/issues/618) · WBS `docs/WBS_GO_LIVE_v0.1.0.md` Track H WP-CL (commit `34537ad`).
- Kit sœur signé `track-h-bloqueurs/` (`50f3c43`).
- Mémoires : voir `README.md`.
