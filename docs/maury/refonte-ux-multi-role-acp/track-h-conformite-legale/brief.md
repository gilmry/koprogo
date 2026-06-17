---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: A (Vision TOGAF)
status: Draft 0.1 — Maury-grade
date: 2026-06-15
authors: [Claude Opus 4.8 (drafting), @gilmry (signature pending)]
related_issues: [553, 554, 561, 580, 584, 618]
---

# Brief Track H — Conformité légale copropriété

## 1. Vision

**KoproGo doit être un outil de gestion de copropriété belge juridiquement irréprochable.** Tout calcul (charges, quorum, vote, appels de fonds, état daté, clôture d'AG) doit reposer sur un modèle fidèle au Code civil Livre 3 (titre copropriété, réforme 2018/2020) et à la loi du 18/06/2018. La revue domain du 2026-06-15 a montré que le modèle actuel comporte des **divergences structurelles** (acte de base au mauvais niveau, quorum incomplet, fonds de réserve absent, droit de vote des lots démembrés non géré).

Cette refonte établit le **modèle hybride** : l'acte de base (dénominateur des quotités) est porté par la **copropriété (ACP)**, avec des **sous-totaux par bloc** (associations partielles). La conformité s'évalue à **deux niveaux** (ACP agrégat = source de vérité légale ; building = sous-total bloc). Tout est cadré sur le texte légal, pas sur des conventions hard-codées.

## 2. Personas concernés

### 2.1. Admin garant de l'acte de base (interne — @gilmry)
- **Rôle** : saisit la fiche copropriété (ACP), son acte de base (dénominateur 1000/10000), répartit les quotités par lot et par bloc.
- **Responsabilité légale** : garantit `Σ quotités = total_tantiemes` de l'acte de base (Art. 3.84) avant que la copropriété soit exploitable.
- **Besoin** : voir la conformité ACP (agrégat tous blocs) + par bloc ; bloquer l'exploitation si non conforme.

### 2.2. Syndic exploitant
- **Rôle** : génère charges, appels de fonds, AG, états datés, constitue les fonds.
- **Responsabilité légale** : Art. 3.86/3.89 — chiffres exacts ; fonds de réserve/roulement obligatoires ; convocations 15 j ; quorum double ; respect des suspensions de vote.
- **Besoin** : erreurs typées explicites (pas de calcul faux silencieux), gates qui empêchent les décisions nulles.

### 2.3. Copropriétaire (plein / usufruitier / nu-propriétaire / indivisaire)
- **Rôle** : vote en AG, paie ses charges selon ses quotités.
- **Responsabilité** : désigner un représentant unique si lot démembré/indivis (Art. 3.87 §1).
- **Besoin** : que son droit de vote soit correctement actif ou suspendu selon sa situation ; charges réparties au juste critère (valeur/utilité).

## 3. Capacités business (CB-L)

| CB | Description | WP |
|---|---|---|
| **CB-L1** | Acte de base (dénominateur quotités) porté par l'ACP ; conformité ACP-level (Σ units tous blocs == acte de base). | CL1 |
| **CB-L2** | Conformité par bloc conservée (sous-total = association partielle). | CL1 |
| **CB-L3** | Tout calcul opérationnel gated par la conformité ACP (422 typé si non conforme). | CL1 |
| **CB-L4** | Quotité d'un lot non bornée à 1000 (acte 10000 supporté lot par lot). | CL2 |
| **CB-L5** | Quorum AG **double** (têtes ET quotités) ; 2e convocation si non atteint. | CL3 |
| **CB-L6** | Gates votes : quorum requis + limite procurations (3/10%) appliqués. | CL3 |
| **CB-L7** | Lot indivis/démembré sans représentant → **droit de vote suspendu**. | CL3 |
| **CB-L8** | Fonds de réserve (≥5%) + fonds de roulement modélisés, comptes distincts. | CL4 |
| **CB-L9** | Répartition des charges paramétrable (valeur / utilité / mixte). | CL4 |
| **CB-L10** | Budget en Decimal exact (pas f64). | CL4 |
| **CB-L11** | Associations partielles (personnalité juridique propre optionnelle, quotités 2 niveaux). | CL5 |
| **CB-L12** | Units rattachées à l'ACP (`acp_id`) en cohérence post-#602. | CL6 |
| **CB-L13** | Délai de convocation correct (15 j toutes AG). | CL7 |

## 4. Invariants techniques (INV-L)

| INV | Énoncé | Article |
|---|---|---|
| **INV-L1** | `Acp::is_conformant(metrics) == (Σ units_count tous blocs == total_units_acp ?) && (Σ quota tous blocs == acps.total_tantiemes)`. Decimal strict. | 3.84 |
| **INV-L2** | `acps.total_tantiemes` est l'unique source de vérité du dénominateur ; `buildings.total_tantiemes` = sous-total bloc (dérivé/saisi association partielle). | 3.84 |
| **INV-L3** | `AcpNotConformantError { acp_id, units_delta, quota_delta, quota_basis }` typé → AppError::AcpNotConformant 422. | 3.84, CRITICAL #4 |
| **INV-L4** | Les 4 gates validate-before-compute (expense/call_for_funds/charge_distribution/etat_date) vérifient la conformité **ACP** (résolvent building.acp_id). | 3.85/3.86 |
| **INV-L5** | `Unit.quota ∈ (0, acp.total_tantiemes]` — pas de constante 1000 hard-codée. | 3.84 |
| **INV-L6** | Quorum = `attended_owners*2 > total_owners` **ET** `attended_quotas > total_quotas/2`. | 3.87 §5 |
| **INV-L7** | `validate_proxy_mandate` (≤3 mandats OU ≤10% des voix) appliqué à l'enregistrement de vote. | 3.87 §7 |
| **INV-L8** | `Unit::voting_right_status()==Suspended` si lot démembré/indivis sans `is_voting_representative` désigné ; vote rejeté `VotingRightSuspended` 422. | 3.87 §1 |
| **INV-L9** | Fonds : `acps.reserve_fund_balance ≥ 0.05 × charges_ordinaires_n1` sauf `reserve_fund_waived` (vote 4/5) ; réserve + roulement comptes distincts. | 3.86 §3 / loi 2019 |
| **INV-L10** | `DistributionCriteria {value|utility|mixed}` ; répartition par quotité du lot, puis pondérée par `ownership_percentage` multi-propriétaires. | 3.86 |
| **INV-L11** | Tout montant monétaire/quota en `rust_decimal::Decimal` (jamais f64). | ADR-0007 |
| **INV-L12** | `partial_associations.has_legal_personality=true` interdit si ACP parent sans personnalité ; quotités 2 niveaux (générale ACP + particulière PA). | 3.86 |
| **INV-L13** | Convocation ≥ 15 j toutes AG (ordinaire ET extraordinaire), urgence sans seuil chiffré. | 3.87 §3 |
| **INV-L14** | Tests 4-cat (`@happy`+`@edge`+`@security`+`@negative`) RED-first par invariant et par use-case impacté. | CRITICAL #3 |

## 5. Critères de succès (SCB-L)

| SCB | Mesure |
|---|---|
| **SCB-L1** | `cargo test --lib acp::assert_conformant_tests` → 4-cat GREEN (acte 1000 + 10000 + multi-blocs). |
| **SCB-L2** | BDD `validate_before_compute_acp.feature` → 4 use-cases gated ACP-level GREEN. |
| **SCB-L3** | `cargo test --lib meeting::quorum_double_tests` → têtes+quotités GREEN (bornes). |
| **SCB-L4** | BDD `voting_right_suspension.feature` → lot démembré/indivis sans représentant rejeté GREEN. |
| **SCB-L5** | `cargo test --lib budget` Decimal ; `acp::reserve_fund_tests` (5%) GREEN. |
| **SCB-L6** | BDD `partial_association.feature` → quotités 2 niveaux + conformité PA GREEN. |
| **SCB-L7** | Migration `units.acp_id` + `.down` réversibles ; backfill vérifié. |
| **SCB-L8** | `CONVOCATIONS_AG.rst` corrigé (15 j toutes AG). |
| **SCB-L9** | `make ci` GREEN sur feature/dev, sans nouveau testIgnore/continue-on-error. |
| **SCB-L10** | Aucun nouvel `unwrap`/`expect`/`Result<_, String>`/`f64` monétaire introduit. |

## 6. Hors-scope explicite

- Moteur comptable complet (régularisations annuelles, ventilation analytique fine) — v0.2.0.
- Signatures eIDAS / governance hybride distanciel (autres stories Maury slice 4).
- Vote électronique distant (#48) — autre story.
- Self-healing data migration des copropriétés existantes non conformes — admin corrige manuellement (rapport read-only autorisé).

## 7. Risques et mitigations

| Risque | Prob. | Impact | Mitigation |
|---|---|---|---|
| Migration multi-building : `acps.total_tantiemes = SUM(blocs)` faux si actes divergents | Moy | Élevé | `RAISE WARNING` + audit + validation admin (mémoire `admin-publishes-conform-buildings`). |
| Rétro-compat seeds mono-immeuble (bascule ACP-level) | Élevée | Élevé | Adapter seeds BDD/E2E (stash `git stash@{0}` repris en CL1) ; conformité ACP exige building rattaché conforme. |
| Quotités 2 niveaux (associations partielles) complexifie le modèle lot | Moy | Élevé | MVP : `units.particular_quota` nullable ; ADR tranche le schéma ; ne pas viser le moteur complet. |
| Quorum têtes : compter les copropriétaires distincts (`unit_owners`) | Moy | Moyen | `COUNT(DISTINCT owner_id)` ; présents dérivés des présences AG. |
| `present_quotas` DOUBLE PRECISION en DB (dette ADR-0008) | Faible | Moyen | Ne pas aggraver (têtes = i32) ; migration Decimal séparée. |
| Cascade `units.acp_id` (#603 scope_guard, handlers, DTO) | Moy | Moyen | Migration 3 étapes ; backfill depuis `building.acp_id` toujours présent post-#602. |
| Volume (16 stories) → dérive scope | Moy | Moyen | Gantt par passe d'agent + 1 PR/story + gate Maury ; chemin critique ≈ 9 passes. |

## 8. Budget tokens estimé

- 6 docs BMAD ≈ 2500-3000 lignes ≈ 130k tokens.
- Exec : ~22 passes d'agent (S=1/M=1/L=2 passes), chemin critique ≈ 9 passes ; budget agents ~600-900k tokens selon parallélisme.

## 9. Signature

```
Mary (Brief) : Draft v0.1 — signature pending @gilmry
```
→ Signer débloque le PRD (`prd.md`).
