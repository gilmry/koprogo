---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: B (Business architecture TOGAF)
status: Draft 0.1 — Maury-grade
date: 2026-06-15
authors: [Claude Opus 4.8 (drafting), @gilmry (signature pending)]
depends_on: brief.md (Draft v0.1)
---

# PRD Track H — Conformité légale copropriété

> Phase B TOGAF. Functional Requirements regroupés par work package WBS (FR-CL1..7). Pour les types Rust / migrations / mermaid, voir `architecture.md`. Pour les stories briefables, voir `stories.md`.

## FR-CL1 — Socle conformité ACP hybride (Art. 3.84) · stories H4-H7 · BLOQUEUR

### Goal métier
L'acte de base d'une copropriété fixe le dénominateur des quotités (1000/10000) et la répartition par lot. Ce dénominateur appartient à la **copropriété (ACP)**, pas à un immeuble isolé. KoproGo doit (a) stocker `acps.total_tantiemes`, (b) évaluer la conformité au niveau ACP (somme de tous les lots de tous les blocs == dénominateur), (c) bloquer tout calcul opérationnel sur une ACP non conforme.

### User journey
1. Admin crée l'ACP « Résidence Les Iris » avec `total_tantiemes=10000` (groupe 3 blocs A/B/C).
2. Il rattache 3 buildings (blocs) et saisit les lots ; tant que `Σ quotités tous blocs ≠ 10000` → ACP non conforme.
3. Syndic tente `POST /call-for-funds/send` → backend résout `building.acp_id`, charge `AcpMetrics`, `acp.assert_conformant()?` → **422 `ACP_NOT_CONFORMANT`** avec `units_delta`, `quota_delta`, `quota_basis=10000`.
4. Admin complète les lots → conforme → le calcul passe.

### AC 4-cat
- `@happy` — ACP conforme (mono ou multi-blocs, acte 1000 ET 10000) → calculs 200.
- `@edge` — ACP 10000 avec dérive 0,1 → 422 `quota_delta=0.1, quota_basis=10000`.
- `@security` — bypass API direct sur ACP non conforme → 422 + (audit reporté CL via `validate-before-compute`).
- `@negative` — ACP/building inexistant → 404 avant le check.

### Reste du WP
- H4 migration `acps.total_tantiemes` + backfill réversible. H5 `Acp::assert_conformant` + erreur typée. H6 `AcpRepository::find_by_id_with_metrics`. H7 bascule des 4 gates building→ACP (retravaille WP-H2).

## FR-CL2 — Borne quotité par acte de base (bug 10000) · story H8

### Goal
Une unit doit pouvoir porter une quotité cohérente avec l'acte de base de son ACP (ex. 5000 sur base 10000). La constante `Unit::MAX_QUOTA=dec!(1000)` rejette ce cas à tort.

### AC 4-cat
- `@happy` — unit quota 5000 sur ACP base 10000 acceptée.
- `@edge` — unit quota == total_tantiemes acceptée ; > total_tantiemes rejetée (typé).
- `@security` — pas de contournement de la borne agrégée.
- `@negative` — quota ≤ 0 rejeté (conservé).

## FR-CL3 — Gouvernance AG conforme (Art. 3.87 §1/§5/§7) · stories H9-H10-H17 · BLOQUEUR

### Goal métier
Une AG ne délibère valablement que si le **double quorum** est atteint (têtes ET quotités) ; les votes respectent la limite de procurations ; un lot indivis/démembré sans représentant désigné a son **droit de vote suspendu**.

### User journeys
**a) Quorum double** — Syndic ouvre l'AG : 8 copropriétaires sur 20 présents (40% têtes) mais représentant 600/1000 quotités. Avant fix : quorum OK (quotités>50%). Après : **KO** (têtes ≤ 50%) → 2e convocation.
**b) Procurations** — Un mandataire cumule 5 procurations représentant 4% des voix → OK (≤10%) ; 5 procurations à 15% → rejet `PROXY_LIMIT_EXCEEDED`.
**c) Suspension** — Lot 12 en usufruit (usufruitier + nu-propriétaire), aucun représentant désigné → `Unit::voting_right_status()==Suspended` → tentative de vote `POST /votes` → **422 `VOTING_RIGHT_SUSPENDED`** ; le lot ne compte ni en têtes ni en quotités pour le quorum. Dès qu'un représentant est désigné (`is_voting_representative=true`), le vote redevient possible.

### AC 4-cat (résumé)
- `@happy` — double quorum atteint + représentant désigné → vote enregistré.
- `@edge` — quorum à la borne (exactement 50% têtes OU quotités) → KO (majorité stricte) ; proxy à exactement 10% → OK.
- `@security` — vote sur lot suspendu rejeté ; proxy au-delà de 3/10% rejeté ; `attendees_count` falsifié ignoré (source = checklist).
- `@negative` — meeting inexistant / déjà completed → erreurs typées.

## FR-CL4 — Finances conformes (Art. 3.86, loi 2019) · stories H11-H12-H13

### Goal métier
Budget exact (Decimal), répartition des charges selon le critère de l'acte (valeur/utilité/mixte), **fonds de réserve (≥5% des charges ordinaires N-1) + fonds de roulement** obligatoires sur comptes distincts.

### User journeys
- **Budget** : provision mensuelle = `budget_annuel / 12` en Decimal exact (plus de dérive f64 `6249.999`).
- **Critère utilité** : l'AG vote « chauffage réparti par superficie » → `distribution_criteria=utility` → répartition non basée sur les quotités-valeur.
- **Fonds réserve** : à la clôture, `reserve_fund_balance < 0.05 × charges_ordinaires_n1` et `reserve_fund_waived=false` → alerte/blocage `RESERVE_FUND_INSUFFICIENT` ; appels de fonds typés `fund_type ∈ {ordinary, working_capital, reserve}`.

### AC 4-cat
- `@happy` — budget Decimal ; appel réserve séparé ; réserve ≥ 5% OK.
- `@edge` — réserve exactement 5% OK ; 4,99% KO ; renonciation 4/5 → check désactivé.
- `@security` — pas de répartition sur critère non voté ; pas de contournement du seuil 5%.
- `@negative` — `total_budget` négatif rejeté typé.

## FR-CL5 — Associations partielles (Art. 3.86) · story H16 · ⛔ DIFFÉRÉ v0.2.0 (D6 @gilmry)

> NON inclus dans v0.1.0 (décision PO 2026-06-15). Spécifié ci-dessous pour mémoire v0.2.0. Les migrations `partial_associations` / `units.particular_quota` ne sont pas exécutées en v0.1.0.

### Goal métier
Un groupe d'immeubles peut avoir des **associations partielles** (par bloc) gérant les parties communes particulières (ascenseur/toiture d'un bloc), avec leur **propre personnalité juridique** (si l'ACP principale en a) ou non (vote AG 4/5), et leurs **propres quotités** (dénominateur particulier).

### User journey
- ACP « Les Iris » (général 10000) ; bloc A constitue une association partielle « AP Bloc A » (particulier 1000) gérant son ascenseur. Un lot du bloc A a une quotité **générale** (dans les 10000 de l'ACP) ET une quotité **particulière** (dans les 1000 de l'AP). Les charges d'ascenseur sont réparties sur les quotités particulières du bloc A uniquement.

### AC 4-cat
- `@happy` — création AP + quotités particulières conformes (Σ == dénominateur PA) ; charges PA réparties sur le périmètre.
- `@edge` — AP sans personnalité (créée par 4/5) ; lot sans quotité particulière (hors AP).
- `@security` — `has_legal_personality=true` interdit si ACP parent sans personnalité.
- `@negative` — quotités particulières incohérentes → conformité PA KO typée.

## FR-CL6 — Migration units.acp_id (cohérence #602) · story H15

### Goal
Aligner `units` sur le modèle ACP (comme `buildings.acp_id` post-#602) : remplacer `organization_id` par `acp_id`.

### AC 4-cat
- `@happy` — migration 3 étapes ; backfill depuis `building.acp_id` ; lecture/écriture units OK.
- `@edge` — units orphelines (building sans acp) → backfill signalé.
- `@security` — isolation #603 (scope_guard) préservée via acp_id.
- `@negative` — `.down` restaure `organization_id`.

## FR-CL7 — Doc convocation (Art. 3.87 §3) · story H14

### Goal
Corriger `CONVOCATIONS_AG.rst` : **15 j minimum pour toutes les AG**, urgence sans seuil chiffré (supprimer le « 8 j AGE » erroné). Le code (`convocation.rs`) est déjà correct.

### AC
- Doc citée Art. 3.87 §3 ; table unique 15 j ; mention urgence ; vérifier qu'aucun code ne distingue AGO/AGE 15/8.

## NFR transverses

- **NFR-1 (sécurité)** : impossibilité de bypass des gates par API directe (test `@security`).
- **NFR-2 (a11y WCAG 2.1 AA)** : bandeaux/erreurs FE (banner ACP non conforme, suspension vote) avec `role="alert"`, contraste, texte explicite.
- **NFR-3 (i18n)** : messages FR/NL/EN/DE via `useTranslations()`.
- **NFR-4 (Result<E> typé)** : aucun nouvel `unwrap`/`expect`/`Result<_, String>` hors bridge legacy justifié.
- **NFR-5 (no f64)** : Decimal partout (montants + quotités).
- **NFR-6 (migration réversible)** : chaque migration a son `.down`.

## Matrice de traçabilité

| FR | CB-L | INV-L | SCB-L | Stories | WP |
|---|---|---|---|---|---|
| FR-CL1 | CB-L1/2/3 | INV-L1/2/3/4 | SCB-L1/2 | H4-H7 | CL1 |
| FR-CL2 | CB-L4 | INV-L5 | SCB-L1 | H8 | CL2 |
| FR-CL3 | CB-L5/6/7 | INV-L6/7/8 | SCB-L3/4 | H9/H10/H17 | CL3 |
| FR-CL4 | CB-L8/9/10 | INV-L9/10/11 | SCB-L5 | H11/H12/H13 | CL4 |
| FR-CL5 | CB-L11 | INV-L12 | SCB-L6 | H16 | CL5 |
| FR-CL6 | CB-L12 | — | SCB-L7 | H15 | CL6 |
| FR-CL7 | CB-L13 | INV-L13 | SCB-L8 | H14 | CL7 |

## Signature

```
Mary (Brief) : signature pending @gilmry
John (PRD)   : Draft v0.1 — signature pending @gilmry
```
→ Signer débloque l'Architecture (`architecture.md`).
