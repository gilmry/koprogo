---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: index
status: SIGNED v1.0 par @gilmry 2026-06-15 — Phase 6 exécution débloquée
date: 2026-06-15
authors: [Claude Opus 4.8 (drafting), @gilmry (signature 2026-06-15)]
related_issues: [553, 554, 561, 580, 584, 618]
parent_maury: refonte-ux-multi-role-acp (validation v1.0 signée 2026-05-20)
wbs: docs/WBS_GO_LIVE_v0.1.0.md Track H — WP-CL0 à WP-CL7 (commit 34537ad)
---

# Track H — Conformité légale copropriété — BMAD index

Cible : mettre le modèle de copropriété KoproGo en **pleine conformité avec le droit belge** (Code civil Livre 3, Art. 3.84-3.88 + loi 18/06/2018), suite à la revue domain complète du 2026-06-15. Modèle **hybride** : acte de base / dénominateur des quotités porté par l'**ACP** + sous-totaux par bloc (associations partielles), conformité à **2 niveaux**.

## Trigger

- Revue domain vs droit belge (3 audits + recherche légale sourcée) — issue [#618](https://github.com/gilmry/koprogo/issues/618).
- 10+ divergences/absences code↔loi au-delà du bug `total_tantiemes` déjà corrigé en Story H1 (Track H bloqueurs, commit `6a053a1`).
- Décisions PO @gilmry 2026-06-15 : **(D1) modèle hybride** (acte de base sur ACP), **(D2) full conformité maintenant**, **in-scope** associations partielles (H16) + migration `units.acp_id` (H15) + multi-titulaires/représentant de vote (H17).

## Position vs Track H bloqueurs (kit sœur signé `track-h-bloqueurs/`)

| Track H bloqueurs (signé `50f3c43`) | Track H conformité légale (ce dossier) |
|---|---|
| H1 `BuildingNotConformantError` + bug `quota_basis` (FAIT `6a053a1`) | **conservé** = sous-total par bloc |
| H2 validate-before-compute building-level (FAIT `3ede509`) | **retravaillé** → gate ACP-level (CL1·H7) |
| H3 `Meeting::assert_can_complete` quorum simple (FAIT `3ede509`) | **étendu** → quorum double + têtes (CL3·H9) |

## Index des documents (ordre de lecture Maury TOGAF)

1. **`brief.md`** (Phase A Vision) — personas, capacités légales CB-L1..n, invariants INV-L1..n, critères succès SCB-L, hors-scope, risques, **sources légales sourcées**, budget.
2. **`prd.md`** (Phase B Business) — FR-CL1..7 (par work package WBS), user journeys narratifs, AC 4-cat, NFR, matrice traçabilité.
3. **`architecture.md`** (Phase C App+Data) — modèle hybride (acps.total_tantiemes + sous-totaux blocs), conformité 2 niveaux, associations partielles (quotités 2 niveaux), quorum double, représentant de vote/suspension, fonds réserve/roulement, migrations réversibles, mermaid, patterns BDD/TDD.
4. **`stories.md`** (Phase D Stories) — H0-ADR + H4-H17 self-contained briefables + **Gantt par passe d'agent**.
5. **`validation.md`** (Phase F PO) — acceptation @gilmry.

## Bases légales (sourcées)

- **Art. 3.84** — copropriété = immeuble OU groupe d'immeubles ; acte de base = statuts ; quote-part des communs par lot (valeur respective).
- **Art. 3.86** — ACP = personne morale ; groupe = personnalité au niveau groupe ; **fonds réserve (≥5% charges ord. N-1) + roulement obligatoires** (loi 2019), comptes distincts, réserve renonçable 4/5 ; associations partielles.
- **Art. 3.87 §1** — lot indivis/démembré (usufruit/nue-propriété/emphytéose/superficie) → **droit de vote SUSPENDU** jusqu'à désignation d'un représentant unique.
- **Art. 3.87 §3** — convocation **15 j toutes AG**, sauf urgence (pas de 8 j).
- **Art. 3.87 §5** — **DOUBLE quorum** : > moitié des têtes ET ≥ moitié des quotités.
- **Art. 3.87 §7** — procurations max 3, ou ≤ 10% des voix.
- **Art. 3.88** — majorités absolue / 2-3 / 4-5 / unanimité.

Sources : [Code civil ejustice](https://www.ejustice.just.fgov.be/img_l/pdf/2020/02/04/2020020347_F.pdf) · [quotités](https://www.choisirunsyndic.be/dossiers/quotites-en-copropriete/) · [personnalité juridique](https://copropriete-ejuris.be/personnalite-juridique/) · [convocation/AG](https://www.droitbelge.be/fiches_detail.asp?idcat=9&id=623) · [fonds réserve](https://vjn-legal.be/copropriete-fonds-de-reserve-et-fonds-de-roulement-desormais-obligatoires/) · [procurations](https://www.choisirunsyndic.be/question/procurations-ag-limite-a-3-mandats-ou-10/) · [vote indivision/usufruit](https://copropriete-ejuris.be/assemblee-generale-des-coproprietaires/)

## Mapping WBS ↔ stories

| WP WBS | Stories | Bloqueur légal |
|---|---|---|
| WP-CL0 | H0-ADR | gate signature |
| WP-CL1 | H4+H5+H6+H7 | Art. 3.84 (BLOQUEUR) |
| WP-CL2 | H8 | — (bug 10000) |
| WP-CL3 | H9+H10+H17 | Art. 3.87 §1/§5/§7 (BLOQUEUR) |
| WP-CL4 | H11+H12+H13 | Art. 3.86 / loi 2019 |
| ~~WP-CL5~~ | ~~H16~~ | **DIFFÉRÉ v0.2.0** (D6 @gilmry) |
| WP-CL6 | H15 | — (cohérence #602) |
| WP-CL7 | H14 | Art. 3.87 §3 (doc) |

## Gates de signature (workflow Maury)

```
brief.md (Mary) → @gilmry sign
   ↓
prd.md (John) → @gilmry sign
   ↓
architecture.md (Winston) → @gilmry sign
   ↓
stories.md (Bob) → @gilmry sign
   ↓
validation.md (PO @gilmry) → débloque passe CL0/CL1
   ↓
agents Track H conformité briefés selon Gantt par passe d'agent (WBS + stories.md)
```

## Mémoires Maury appliquées

`maury-fullstack-first` · `maury-token-economy` · `tdd-bdd-four-categories` · `quota-basis-acte-de-base` · `admin-publishes-conform-buildings` · `validate-before-compute` · `no-f64-in-money` · `world-model-seed` · `docker-parallelism-bottleneck` · `multirole-narrative-scenarios` · `data-testid-systematic` · `a11y-wcag-aa-baseline`.

## Statut de signature

```
brief.md          : SIGNED v1.0 par @gilmry 2026-06-15
prd.md            : SIGNED v1.0 par @gilmry 2026-06-15
architecture.md   : SIGNED v1.0 par @gilmry 2026-06-15
stories.md        : SIGNED v1.0 par @gilmry 2026-06-15
validation.md     : SIGNED v1.0 par @gilmry 2026-06-15
WBS Track H WP-CL : intégré (commit 34537ad)
D6 (assoc. partielles) : différé v0.2.0
```

**→ Phase 6 débloquée. Passe d'agent CL0 (ADR) autorisée, puis CL1 (socle conformité ACP). Périmètre v0.1.0 : CL0/CL1/CL2/CL3/CL4/CL6/CL7.**
