---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: validation
phase_togaf: F (Migration Planning)
agent_bmad: Product Owner (@gilmry)
authors: [Gilles Maury]
date: 2026-06-15
version: 1.0
status: SIGNED v1.0 par @gilmry 2026-06-15 — Phase 6 exécution débloquée
signed_at: 2026-06-15
signed_by: "@gilmry"
brief_source: brief.md (SIGNED v1.0)
prd_source: prd.md (SIGNED v1.0)
architecture_source: architecture.md (SIGNED v1.0)
stories_source: stories.md (SIGNED v1.0)
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
| **D3 associations partielles (H16/CL5)** | **DIFFÉRÉ v0.2.0** (décision D6 ci-dessous) |
| **D4 migration units.acp_id** | In-scope (H15) ✅ |
| **D5 multi-titulaires/suspension** | In-scope (H17) ✅ |
| **D6 schéma quotités 2 niveaux** | **Tranché @gilmry 2026-06-15 : reporter H16 (associations partielles + quotités 2 niveaux) hors v0.1.0.** Le modèle hybride v0.1.0 = ACP (acte de base) + building (sous-total bloc), sans `partial_associations` ni `units.particular_quota`. |
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

- [x] D6 (schéma quotités 2 niveaux) tranchée par @gilmry → **H16/CL5 reporté v0.2.0**.
- [ ] @gilmry relit les 5 docs (Draft v0.1) avant signature.
- [ ] Sources légales vérifiables (liens README).
- [ ] Cohérence WBS (WP-CL0-4/6-7, CL5 différé) ↔ stories (H4-H15, H17) ↔ #618.
- [ ] Track H bloqueurs déjà mergé pris en compte (H1 conservé, H2/H3 retravaillés).

**Périmètre v0.1.0 confirmé** : CL0, CL1, CL2, CL3, CL4, CL6, CL7. **CL5 (associations partielles) → v0.2.0.**

## 6. Signature

```
Mary (Brief)        : SIGNED v1.0 par @gilmry 2026-06-15
John (PRD)          : SIGNED v1.0 par @gilmry 2026-06-15
Winston (Arch)      : SIGNED v1.0 par @gilmry 2026-06-15
Bob (Stories)       : SIGNED v1.0 par @gilmry 2026-06-15
PO (Validation)     : SIGNED v1.0 par @gilmry 2026-06-15
```

**→ 5 docs signés + D6 tranché (H16 différé v0.2.0). Phase 6 débloquée : passe d'agent CL0 (ADR) autorisée, puis CL1.**

## 7. Refs
- Issue [#618](https://github.com/gilmry/koprogo/issues/618) · WBS `docs/WBS_GO_LIVE_v0.1.0.md` Track H WP-CL (commit `34537ad`).
- Kit sœur signé `track-h-bloqueurs/` (`50f3c43`).
- Mémoires : voir `README.md`.
