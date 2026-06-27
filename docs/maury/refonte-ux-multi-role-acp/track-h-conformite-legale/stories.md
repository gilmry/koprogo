---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: D (Stories TOGAF)
status: SIGNED v1.0 par @gilmry 2026-06-15 — Phase 6 exécution débloquée
date: 2026-06-15
authors: [Claude Opus 4.8 (drafting), @gilmry (signature 2026-06-15)]
depends_on: brief.md (SIGNED v1.0), prd.md (SIGNED v1.0), architecture.md (SIGNED v1.0)
---

# Stories Track H — Conformité légale copropriété

> Phase D. Stories self-contained briefables par agent fresh. 1 story = 1 PR = ≥1 passe d'agent (cf. anatomie WBS). Tests 4-cat RED-first (`@happy`+`@edge`+`@security`+`@negative`). Détails types/SQL : `architecture.md`.

## Plan par passe d'agent

Unité = **1 passe** : PLAN → Story → BDD 4-cat → TDD 4-cat → exéc (ROUGE) → code (VERT) → BLEUE/refactor (tests continus anti-régression) → e2e → qualité → sécurité → commit.

| Story | WP | Taille | Passes | Déps | Stash impact |
|---|---|---|---|---|---|
| H0-ADR | CL0 | M (doc) | 1 | — | — |
| H4 migration acps.total_tantiemes | CL1 | M | 1 | H0 | — |
| H5 Acp.assert_conformant + erreur | CL1 | M | 1 | H4 | — |
| H6 AcpRepository metrics | CL1 | S | 1 | H5 | — |
| H7 bascule 4 gates building→ACP | CL1 | L | 2 | H6 | reprend stash (seeds ACP-level) |
| H8 Unit MAX_QUOTA base 10000 | CL2 | S | 1 | H5 | — |
| H9 quorum double | CL3 | L | 2 | H0 | adapte match MissingInvariant |
| H10 gates votes quorum+proxy | CL3 | M | 1 | H9 | — |
| H17 représentant vote/suspension | CL3 | L | 2 | H10 | — |
| H11 budget f64→Decimal | CL4 | M | 1 | H0 | — |
| H12 DistributionCriteria | CL4 | M | 1 | H7 | — |
| H13 fonds réserve/roulement | CL4 | L | 2 | H4 | — |
| H14 doc CONVOCATIONS | CL7 | S (doc) | 1 | — | — |
| H15 units acp_id | CL6 | L | 2 | H4 | — |
| ~~H16 associations partielles~~ | ~~CL5~~ | — | — | — | **DIFFÉRÉ v0.2.0 (D6 @gilmry)** |

**Chemin critique** ≈ **7 passes** (H0→H4→H5→H6→H7→H12) après report de H16. Branche gouvernance H0→H9→H10→H17 ≈ 6 passes en parallèle. Total ≈ **20 passes**. Cf. Gantt WBS.

> **H16 / CL5 (associations partielles à personnalité juridique propre + quotités à 2 niveaux) : DIFFÉRÉ v0.2.0** par décision PO @gilmry 2026-06-15 (D6). Le modèle hybride v0.1.0 conserve ACP (dénominateur acte de base) + building (sous-total bloc) ; pas de `units.particular_quota`, pas de table `partial_associations`. La conception reste en §H16 ci-dessous pour mémoire v0.2.0.

---

## H0-ADR — ADR conformité copropriété (CL0)

**Goal** : consigner les bases légales + décisions de modèle dans `docs/adr/` AVANT code.
**Livrables** :
- `00XX-acte-de-base-conformite-copropriete.md` : acte de base sur ACP, conformité 2 niveaux, associations partielles, **trancher le schéma quotités 2 niveaux** (`units.particular_quota` nullable vs table `unit_quotas`).
- `00XX-quorum-double-et-suspension-vote.md` : double quorum Art. 3.87 §5 + suspension Art. 3.87 §1.
- `00XX-fonds-reserve-roulement.md` : fonds obligatoires loi 2019.
**DoD** : 3 ADR statut `Proposed` (acceptation @gilmry au merge) ; chaque décision sourcée (liens README).

---

## H4 — Migration acps.total_tantiemes (CL1)

**Goal/parent** : Art. 3.84 / INV-L2 / FR-CL1. Porter le dénominateur sur l'ACP.
**AC 4-cat** : `@happy` migration applique + backfill mono=building ; `@edge` multi-building → SUM + WARNING ; `@security` CHECK>0 ; `@negative` `.down` réversible (DROP COLUMN).
**Files** : `backend/migrations/..._add_acps_total_tantiemes.sql` + `.down` ; `..._backfill_acps_total_tantiemes.sql` + `.down` ; `acp.rs` (champ `total_tantiemes: i32`) ; `acp_dto` + repo SELECT.
**DoD** : `sqlx migrate run` + down OK ; warnings backfill inspectés ; `cargo test --lib acp` vert.

---

## H5 — Acp::assert_conformant + AcpNotConformantError (CL1)

**Goal/parent** : Art. 3.84 / INV-L1/L3 / FR-CL1.
**AC 4-cat** : `@happy` ACP 1000 ET 10000 conforme → Ok ; `@edge` dérive 0,1 → `quota_delta=0.1, quota_basis` ; `@security` metrics forgé (vérité = SQL) ; `@negative` empty metrics → Err.
**Files** : `acp.rs` (`AcpMetrics`, `is_conformant`, `assert_conformant`, `AcpNotConformantError`) ; `application/error.rs` (`AppError::AcpNotConformant` + `From<>` AppError 422 + String) ; tests inline 4-cat + `cargo test --lib application::error`.
**DoD** : tests verts ; pattern identique à `BuildingNotConformantError` ; pas d'`unwrap`/`Result<_,String>` nouveau.

---

## H6 — AcpRepository::find_by_id_with_metrics (CL1)

**Goal/parent** : INV-L4 / FR-CL1.
**AC 4-cat** : `@happy` JOIN multi-building somme correcte ; `@edge` ACP sans units → metrics 0 ; `@security` scope org respecté ; `@negative` ACP inexistant → None.
**Files** : `application/ports/acp_repository.rs` (méthode) ; `infrastructure/database/repositories/acp_repository_impl.rs` (SQL agrégé, `sqlx::query` non-macro offline-safe).
**DoD** : test integration testcontainers vert ; `cargo check --lib --tests` propre.

---

## H7 — Bascule des 4 gates building→ACP (CL1, retravaille WP-H2) · 2 passes

**Goal/parent** : INV-L4 / FR-CL1.
**AC 4-cat** : `@happy` ACP conforme → 4 use-cases 200 ; `@security` ACP non conforme → 422 `ACP_NOT_CONFORMANT` (4 use-cases) ; `@edge` building d'une ACP multi-blocs ; `@negative` building/acp inexistant 404.
**Files** : `expense_use_cases.rs`, `call_for_funds_use_cases.rs`, `charge_distribution_use_cases.rs`, `etat_date_use_cases.rs` (helper `assert_acp_conformant`) ; `main.rs` wiring `acp_repo` ; `conformity_response.rs` (préfixe `ACP_NOT_CONFORMANT:`) ; BDD `validate_before_compute_acp.feature` (4 use-cases × mono/multi-blocs).
**Stash** : reprendre `git stash@{0}` (filler units → conformité ACP-level) + résoudre le **400-vs-422** du spec H2 (`validate-before-compute.spec.ts`).
**DoD** : BDD vert ; e2e track-h vert ; `make ci` vert.

---

## H8 — Unit MAX_QUOTA base 10000 (CL2)

**Goal/parent** : Art. 3.84 / INV-L5 / FR-CL2.
**AC 4-cat** : `@happy` quota 5000 sur ACP 10000 OK ; `@edge` quota == total_tantiemes OK / > rejeté ; `@security` borne agrégée non contournable ; `@negative` quota ≤ 0 rejeté.
**Files** : `unit.rs` (retrait `MAX_QUOTA=dec!(1000)` lignes 7/54/81 ; borne via acte de base) ; tests inline 4-cat ; `validate_unit_shares_distribution(units, total_tantiemes)`.
**DoD** : `git grep "MAX_QUOTA"` → 0 hard-code 1000 ; tests verts.

---

## H9 — Quorum double (CL3, étend WP-H3) · 2 passes

> **Amendement légal 2026-06-25 (@gilmry — décision « corriger les specs d'abord »)** :
> l'AC initiale contredisait l'Art. 3.87 §5 (quotités `>50%` strict + pas d'alternative).
> Règle légale exacte ci-dessous ; à re-valider à la reprise de CL3.

**Goal/parent** : Art. 3.87 §5 / INV-L6 / FR-CL3.

**Règle légale exacte (Art. 3.87 §5)** — quorum atteint à l'ouverture de l'AG si :
- **(A) primaire** : têtes **> 50 %** (strict, « plus de la moitié des copropriétaires ») **ET** quotités **≥ 50 %** (**inclusif**, « au moins la moitié des quotités ») ;
- **OU (B) alternative** : quotités **> 3/4 (75 %)** (strict), quel que soit le nombre de têtes ;
- sinon → 2e convocation (15 j) délibère sans quorum (déjà géré : `is_second_convocation`).

⚠️ **Code H3 actuel** (`meeting.rs:185`) = quorum **simple**, quotités **strict > 50 %** (commentaire cite à tort « §4 »). H9 doit : (1) quotités → **≥ 50 %** inclusif ; (2) ajouter critère **têtes > 50 %** ; (3) ajouter **alternative > 3/4**.

**AC 4-cat (corrigée)** :
- `@happy` têtes>50% ET quotités≥50% → Ok ; OU quotités>75% (têtes quelconque) → Ok.
- `@edge` exactement 50% têtes (quotités≥50%) → KO (têtes strict) ; **exactement 50% quotités (têtes>50%) → Ok** (quotités inclusif) ; exactement 75% quotités sans têtes>50% → KO (alternative strict >3/4) ; >75% quotités seul → Ok.
- `@security` `present_owners_count` / `attendees_count` falsifié ignoré (source = COUNT DISTINCT DB-side).
- `@negative` checklist tous faux → missing complet (têtes + quotités).
**Files** : `meeting.rs` (`validate_quorum` signature + têtes + alternative 3/4 ; `MeetingCompletionChecklist`+têtes ; `MissingInvariant::HeadCountQuorumNotReached`) ; `error.rs` ; migration `meetings.present_owners_count/total_owners_count` ; `meeting_completion_checker_impl` (COUNT DISTINCT owners) ; BDD `quorum_double.feature`.
**Stash** : adapter le match `MissingInvariant` (`bdd_meeting_complete.rs`) au nouveau variant.
**DoD** : domaine 4-cat + BDD verts ; rétrocompat tests meeting existants — ⚠️ seuil quotités `>50%` → `≥50%` : adapter seeds/tests asseyant un rejet à exactement 50% quotités.

---

## H10 — Gates votes quorum + procuration (CL3)

> **⚠️ À vérifier vs loi avant impl (note 2026-06-25)** — Art. 3.87 §7 a **deux** règles
> distinctes : (a) **procurations** : max **3** mandats, **OU** plus si le total des voix du
> mandataire (siennes + mandats) **≤ 10 %** du total des voix de l'AG (10 % **inclusif**) ;
> (b) **réduction de vote** : nul ne peut prendre part au vote pour un nombre de voix
> supérieur à la somme des voix des **autres** copropriétaires présents/représentés (cap
> ~50 %). L'AC ci-dessous ne couvre que (a) ; confirmer si (b) est in-scope H10.

**Goal/parent** : Art. 3.87 §5/§7 / INV-L7 / FR-CL3.
**AC 4-cat** : `@happy` vote enregistré si quorum + proxy OK ; `@edge` proxy exactement 3/10% OK ; `@security` proxy >3 et >10% rejeté `PROXY_LIMIT_EXCEEDED` / vote sans quorum rejeté ; `@negative` meeting inexistant.
**Files** : `resolution_use_cases.rs` (appelle `check_quorum_for_voting` + `validate_proxy_mandate`) ; port comptage mandats ; tests.
**DoD** : tests verts ; les méthodes domaine existantes (non branchées) sont câblées.

---

## H17 — Représentant de vote / suspension (CL3) · 2 passes

> **⚠️ À confirmer vs loi avant impl (note 2026-06-25)** — Art. 3.87 §1 : lot en
> indivision OU démembré (usufruit/nue-propriété, emphytéose, superficie) → les
> titulaires désignent **un mandataire unique** ; à défaut, **droit de vote suspendu**.
> AC alignée. **Point quorum à trancher** : un lot suspendu doit être exclu **et des
> têtes ET des quotités présentes** (cohérence avec H9) — préciser le recalcul.
> Lien avec H9 (quorum) et H10 (gate vote) : ordonner **après** H9+H10.

**Goal/parent** : Art. 3.87 §1 / INV-L8 / FR-CL3.
**AC 4-cat** : `@happy` lot mono-plein OU représentant désigné → vote OK ; `@edge` lot usufruit avec représentant désigné → actif ; `@security` lot indivis/démembré sans représentant → vote rejeté `VOTING_RIGHT_SUSPENDED` + non compté au quorum ; `@negative` désignation de 2 représentants → rejet (un seul).
**Files** : migration `unit_owners.ownership_type` + `is_voting_representative` ; `unit.rs`/`unit_owner.rs` (`OwnershipType`, `voting_right_status`, `VotingRightSuspended`) ; `error.rs` (422) ; gate dans use-case vote ; checker quorum exclut lots suspendus ; BDD `voting_right_suspension.feature` ; FE `<VotingSuspendedBadge>`.
**DoD** : domaine + BDD verts ; quorum recalculé hors lots suspendus.

---

## H11 — Budget f64→Decimal (CL4)

**Goal/parent** : ADR-0007 / INV-L11 / FR-CL4.
**AC 4-cat** : `@happy` provision mensuelle Decimal exacte ; `@edge` budget/12 sans dérive ; `@security` pas d'overflow ; `@negative` budget négatif rejeté typé.
**Files** : `budget.rs` (f64→Decimal sur 4 champs) + DTO + repo + tests ; DB déjà Decimal.
**DoD** : `git grep "f64" backend/src/domain/entities/budget.rs` → 0 (hors surface) ; tests verts.

---

## H12 — DistributionCriteria (CL4)

**Goal/parent** : Art. 3.86 / INV-L10 / FR-CL4.
**AC 4-cat** : `@happy` répartition `value` par quotité ; `@edge` `utility` base alternative ; `@security` critère non voté refusé ; `@negative` somme ≠ total → erreur.
**Files** : `charge_distribution.rs` (`enum DistributionCriteria`, param) ; clarif `quota_percentage` (lot) vs `ownership_percentage` (copropriétaire) ; migration `distribution_criteria` ; tests.
**DoD** : tests verts ; calcul = `(unit.quota/total)*ownership_percentage`.

---

## H13 — Fonds réserve/roulement (CL4) · 2 passes

**Goal/parent** : Art. 3.86 §3 / loi 2019 / INV-L9 / FR-CL4.
**AC 4-cat** : `@happy` réserve ≥5% OK + appels typés ; `@edge` exactement 5% OK / 4,99% KO / renoncé 4/5 ; `@security` seuil non contournable ; `@negative` `RESERVE_FUND_INSUFFICIENT` typé.
**Files** : migration `acps` (3 colonnes funds) + `call_for_funds.fund_type` ; `acp.rs` (`assert_reserve_fund_compliant`) ; `error.rs` ; BDD `reserve_fund.feature` ; FE `<ReserveFundIndicator>`.
**DoD** : tests verts ; comptes distincts modélisés.

---

## H14 — Doc CONVOCATIONS_AG.rst (CL7)

**Goal/parent** : Art. 3.87 §3 / INV-L13 / FR-CL7.
**AC** : table unique 15 j toutes AG ; mention urgence sans seuil ; supprimer « 8 j AGE » ; citer Art. 3.87 §3 ; vérifier code (`convocation.rs` déjà 15 j partout).
**Files** : `docs/CONVOCATIONS_AG.rst`.
**DoD** : doc corrigée relue ; pas de distinction AGO/AGE 15/8 dans le code.

---

## H15 — Migration units.organization_id→acp_id (CL6) · 2 passes

**Goal/parent** : cohérence #602 / FR-CL6.
**AC 4-cat** : `@happy` 3 étapes + backfill `building.acp_id` ; `@edge` unit orpheline signalée ; `@security` isolation #603 via acp_id ; `@negative` `.down` restaure organization_id.
**Files** : migrations 3 étapes + `.down` ; `unit.rs` (champ) + DTO + use-cases + handlers (#603 scope_guard) ; tests.
**DoD** : migration réversible ; tests units verts ; `make ci` vert.

---

## H16 — Associations partielles (CL5) · 2 passes · ⛔ DIFFÉRÉ v0.2.0 (D6 @gilmry)

> Conservé pour mémoire v0.2.0. NON inclus dans le périmètre v0.1.0 (décision PO 2026-06-15). Les migrations associées (`partial_associations`, `buildings.partial_association_id`, `units.particular_quota`) ne sont PAS exécutées en v0.1.0.

**Goal/parent** : Art. 3.86 / INV-L12 / FR-CL5.
**AC 4-cat** : `@happy` AP + quotités particulières conformes + charges PA scopées ; `@edge` AP sans personnalité (4/5) / lot hors AP ; `@security` `has_legal_personality` interdit si ACP parent sans personnalité ; `@negative` quotités particulières incohérentes → conformité PA KO.
**Files** : migration `partial_associations` + `buildings.partial_association_id` + `units.particular_quota` ; `partial_association.rs` (entité + `assert_conformant` PA) ; `charge_distribution` scope PA ; BDD `partial_association.feature` ; FE.
**DoD** : domaine + BDD verts ; quotités 2 niveaux selon ADR H0.

---

## Mémoires appliquées
`quota-basis-acte-de-base` · `admin-publishes-conform-buildings` · `validate-before-compute` · `world-model-seed` · `no-f64-in-money` · `tdd-bdd-four-categories` · `multirole-narrative-scenarios` · `docker-parallelism-bottleneck` · `data-testid-systematic` · `a11y-wcag-aa-baseline`.

## Signature
```
Mary/John/Winston : SIGNED v1.0 par @gilmry 2026-06-15
Bob (Stories)     : SIGNED v1.0 par @gilmry 2026-06-15
```
→ Validation signée (`validation.md`). Passes d'agent CL0→CL7 autorisées.
