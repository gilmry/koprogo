---
audit: f64 monetary usage
date: 2026-04-30
auditor: rust-expert (Claude)
story: EXP-002
status: complete
severity: HIGH (PCMN compliance impacted)
---

# Audit f64 — usage monétaire et comptable

## Contexte

Suite au finding utilisateur 2026-04-29 (*"on a trouvé aussi des f64 dans le code alors que pour la comptabilité il faut pas"*) et à la règle non-négociable formalisée dans la mémoire `project_no-f64-in-money.md`, cet audit recense **toutes** les occurrences de `f64` / `f32` dans le code monétaire et comptable de KoproGo.

**Règle** : tout calcul monétaire ou comptable utilise `rust_decimal::Decimal` (ou `i64` cents) — **jamais** `f64`/`f32` (IEEE 754 cause des arrondis silencieux incompatibles avec PCMN belge AR 12/07/2012).

## Méthode

```bash
grep -rnE '\bf64\b|\bf32\b' backend/src/ --include='*.rs' | grep -v test | sort | uniq
grep -rl 'DOUBLE PRECISION' backend/migrations/
grep -rl 'NUMERIC' backend/migrations/
```

## Résumé exécutif

| Cluster | Sévérité | Files | Occurrences |
|---|---|---|---|
| Domain entities monétaires/comptables | 🔴 CRITICAL | 9 | ~80 |
| DTOs monétaires | 🟠 HIGH | 5 | ~75 |
| Use cases avec calculs financiers | 🟠 HIGH | 3 | ~40 |
| Repositories monétaires | 🟠 HIGH | 2 | ~26 |
| Entités/DTOs/UC pourcentages (votes, quotas, taux) | 🟡 MEDIUM | ~7 | ~50 |
| IoT / Energy / non-monétaire | 🟢 LOW | ~5 | ~75 |
| **Total impact monétaire strict** | — | **19 fichiers** | **~221** |
| Migrations SQL `DOUBLE PRECISION` (devraient être `NUMERIC`) | 🔴 CRITICAL | 9 | — |

**Constat** : `payment.rs` et `payment_method.rs` sont **déjà clean** (utilisent `i64 amount_cents`). Le reste du module finances/comptabilité utilise `f64` partout — incompatible avec la règle PCMN belge.

## Cluster 🔴 CRITICAL — Domain entities monétaires/comptables

| Fichier | f64 count | Impact PCMN |
|---|---|---|
| `domain/entities/etat_date.rs` | 17 | Document légal vente immobilière (Art. 577 CC belge) — exactness obligatoire |
| `domain/entities/invoice_line_item.rs` | 12 | Lignes de facturation TVA — calculs exactness obligatoire |
| `domain/entities/journal_entry.rs` | 11 | Écritures comptables (Noalyss-inspired PCMN) — exactness obligatoire |
| `domain/entities/charge_distribution.rs` | 10 | Allocation charges aux propriétaires — exactness obligatoire |
| `domain/entities/quote.rs` | 10 | Devis entrepreneurs (loi belge 3 quotes >5000€) — exactness obligatoire |
| `domain/entities/expense.rs` | 8 | Charges/factures (cœur comptable) — exactness obligatoire |
| `domain/entities/budget.rs` | 8 | Budget annuel + variance vs réalisé — exactness obligatoire |
| `domain/entities/owner_contribution.rs` | 2 | Contributions individuelles propriétaires — exactness obligatoire |
| `domain/entities/call_for_funds.rs` | 2 | Appels de fonds collectifs — exactness obligatoire |

**Risque concret** : sur un cumul de 12 mois × 10 immeubles × 50 charges/mois × 0.0001 € de drift IEEE 754 par opération, l'écart cumulé peut atteindre plusieurs euros annuels — **incompatible avec la révision des comptes par le commissaire aux comptes**.

## Cluster 🟠 HIGH — DTOs monétaires

| Fichier | f64 count |
|---|---|
| `application/dto/budget_dto.rs` | 23 |
| `application/dto/expense_dto.rs` | 22 |
| `application/dto/etat_date_dto.rs` | 14 |
| `application/dto/payment_reminder_dto.rs` | 8 |
| `application/dto/unit_owner_dto.rs` | 7 |

**Note** : `Decimal` doit avoir feature `serde-with-arbitrary-precision-arbitrary` activée dans `Cargo.toml` pour sérialisation API JSON sans perte. À vérifier en story de migration.

## Cluster 🟠 HIGH — Use cases avec calculs financiers

| Fichier | f64 count | Impact |
|---|---|---|
| `application/use_cases/financial_report_use_cases.rs` | 23 | Bilan + Compte de résultats — rapport légal, exactness obligatoire |
| `application/use_cases/charge_distribution_use_cases.rs` | 8 | Logique d'allocation — exactness obligatoire |
| `application/use_cases/journal_entry_use_cases.rs` | 8 | Validation écritures double-entry (débit = crédit) — exactness OBLIGATOIRE pour cette égalité |

**Risque comptable** : `journal_entry_use_cases` doit garantir `débit == crédit` par écriture. Avec f64, l'égalité peut être fausse silencieusement (`0.1 + 0.2 != 0.3`). Risque majeur pour la conformité comptable.

## Cluster 🟠 HIGH — Repositories (sérialisation PostgreSQL)

| Fichier | f64 count | Impact |
|---|---|---|
| `infrastructure/database/repositories/journal_entry_repository_impl.rs` | 19 | Récupération des écritures — la conversion `DOUBLE PRECISION → f64 → Decimal` perd la précision côté retour |
| `infrastructure/database/repositories/vote_repository_impl.rs` | 7 | voting_power (millièmes) — moins critique mais à clarifier |

## Cluster 🟡 MEDIUM — Pourcentages, votes, quotas

| Fichier | f64 count | Type |
|---|---|---|
| `application/use_cases/resolution_use_cases.rs` | 18 | voting_power (tantièmes/millièmes) |
| `domain/entities/resolution.rs` | 14 | voting_power |
| `domain/entities/age_request.rs` | 8 | shares_pct (1/5 cosignataires) |
| `domain/entities/ag_session.rs` | 7 | quorum_remote_contribution |
| `application/dto/ag_session_dto.rs` | 8 | physical_quotas / remote_quotas / combined_pct |
| `domain/entities/payment_reminder.rs` | 7 | penalty_rate (taux légal belge 8%) |

**Décision** : pourcentages devraient être `Decimal` pour cohérence (et le taux de pénalité belge **est un calcul de centimes**, donc CRITIQUE en réalité). Mais ces fichiers sont moins prioritaires que le cluster CRITICAL.

## Cluster 🟢 LOW — IoT / Energy / indicators non-monétaires

| Fichier | f64 count | Justification keep f64 |
|---|---|---|
| `domain/entities/energy_campaign.rs` | 16 | Consommations kWh, calculs statistiques (k-anonymity ≥ 5), pas de précision centime requise |
| `application/dto/energy_campaign_dto.rs` | 19 | idem |
| `application/dto/iot_dto.rs` | 18 | Lectures IoT (températures, débits) — analyses statistiques OK en f64 |
| `application/use_cases/energy_campaigns_use_cases.rs` | 9 | idem |

**Décision** : f64 acceptable pour ces domaines (mesures physiques ≠ comptabilité). Documenter explicitement ce choix dans un ADR.

## Migrations SQL — `DOUBLE PRECISION` détectées

| Migration | Tables impactées (à valider) |
|---|---|
| `20240101000003_create_units.sql` | `units.area_m2` ? (à vérifier) |
| `20240101000004_create_expenses.sql` | `expenses.amount` etc. — **CRITIQUE** |
| `20250127000000_refactor_owners_multitenancy.sql` | `unit_owners.percentage` — quota copro |
| `20251107120000_create_payment_reminders.sql` | `payment_reminders.amount_due` etc. |
| `20251201000000_create_iot_readings.sql` | `iot_readings.value` (acceptable LOW) |
| `20251203000000_create_work_reports.sql` | montants travaux ? (à vérifier) |
| `20251203000001_create_technical_inspections.sql` | montants inspections ? |
| `20251204000000_create_energy_buying_groups.sql` | consommations (acceptable LOW) |
| `20260312000000_add_quorum_to_meetings.sql` | quorum percentage (MEDIUM) |

**Action** : nouvelles migrations `ALTER TABLE ... ALTER COLUMN ... TYPE NUMERIC(15,2)` à créer pour les colonnes monétaires.

## Plan de migration — découpé en stories follow-up

Total estimé : ~6 stories (M-L), ~2 semaines de travail.

### EXP-003 — Migrate `expense` + `invoice_line_item` (M)

- `domain/entities/expense.rs` (8 f64) + `invoice_line_item.rs` (12 f64) → Decimal
- `application/dto/expense_dto.rs` (22 f64) + tests
- `application/use_cases/expense_use_cases.rs` adapter
- `infrastructure/database/repositories/expense_repository_impl.rs` adapter
- Migration SQL `20260430000000_alter_expense_amounts_to_numeric.sql`
- Tests 4-cat (#427) sur calculs TVA et conversions

### EXP-004 — Migrate `budget` (M)

- `domain/entities/budget.rs` (8 f64) → Decimal
- `application/dto/budget_dto.rs` (23 f64)
- `application/ports/budget_repository.rs` (15 f64)
- Migration SQL ALTER pour table `budgets`
- Tests 4-cat sur calculs variance budget vs réel

### EXP-005 — Migrate `charge_distribution` (S-M)

- `domain/entities/charge_distribution.rs` (10 f64) → Decimal
- `application/use_cases/charge_distribution_use_cases.rs` (8 f64) — **CRITIQUE: somme des % par lot doit = 100% exactement**
- Migration SQL
- Tests 4-cat (`@security` : la somme des allocations doit être stricte)

### EXP-006 — Migrate `journal_entry` (L) **PRIORITÉ MAX**

- `domain/entities/journal_entry.rs` (11 f64) → Decimal
- `application/use_cases/journal_entry_use_cases.rs` (8 f64) — **VALIDATION débit==crédit ne tolère pas l'imprécision**
- `infrastructure/database/repositories/journal_entry_repository_impl.rs` (19 f64)
- Migration SQL
- Tests 4-cat critiques (`@negative` : tentative de saisie déséquilibrée → AppError typé)

### EXP-007 — Migrate `quote` + `etat_date` (M)

- `domain/entities/quote.rs` (10 f64) — devis loi belge 3-quotes-rule
- `domain/entities/etat_date.rs` (17 f64) — Art. 577 CC vente immobilière (légal)
- `application/dto/etat_date_dto.rs` (14 f64)
- Migration SQL
- Tests 4-cat

### EXP-008 — Migrate `owner_contribution` + `call_for_funds` (S)

- `domain/entities/owner_contribution.rs` (2 f64)
- `domain/entities/call_for_funds.rs` (2 f64)
- DTOs associés
- Migration SQL
- Tests 4-cat (`@security` : refus de contribution > montant call)

### FIN-001 — Migrate `financial_report_use_cases` (M-L)

- `application/use_cases/financial_report_use_cases.rs` (23 f64)
- Calculs Bilan + Compte de résultats — **rapport légal**
- Tests 4-cat exhaustifs (`@happy` cas standards, `@edge` montants extrêmes, `@negative` rapport avec données manquantes)

### PCT-001 — Migrate pourcentages (M, MEDIUM priority)

- `resolution.rs` + `resolution_use_cases.rs` (32 f64 total) — voting_power
- `age_request.rs` (8 f64) — shares_pct
- `ag_session.rs` + DTO (15 f64) — quorum
- `payment_reminder.rs` (7 f64) — taux pénalité légal belge

## ADR follow-up

À créer (story dédiée ou en début d'EXP-003) :

- `docs/adr/0001-decimal-vs-f64-for-money.md` : décision formelle, citation PCMN AR 12/07/2012, exemples concrets de drift, configuration Cargo.toml (`rust_decimal` features), pattern de conversion JSON.
- `docs/adr/0002-numeric-vs-double-precision-postgresql.md` : décision côté DB.
- `docs/adr/0003-iot-energy-keep-f64.md` : justification que les domaines non-comptables peuvent garder f64 (mesures physiques).

## Configuration Cargo.toml requise pour Decimal serde

```toml
rust_decimal = { version = "1.36", features = ["serde-with-arbitrary-precision", "macros"] }
```

(À vérifier : `serde-with-arbitrary-precision` active la sérialisation JSON sans perte de précision via String quand le nombre dépasse `f64`.)

## Risque transition (compilation tests)

Migrer une entité domain de `f64` vers `Decimal` casse en cascade :
- DTOs (utilisent f64 dans `From<Entity>`)
- Use cases (paramètres f64)
- Handlers (extraction f64 depuis JSON)
- Repositories (sérialisation PG)
- Migration SQL (type colonne)
- Tests (assertions sur f64)

→ Chaque story de migration doit toucher l'**ensemble vertical** (entité + DTO + use case + handler + repo + migration + tests) pour rester compilable. D'où la taille M des stories.

**Anti-pattern à éviter** : migrer entité seule + ajouter `as f64` / `f64::from(decimal)` partout pour préserver compilation. C'est pire que rien (fausse sécurité, perte de précision à chaque conversion).

## Hooks à activer en post-migration

- Hook clippy custom (à coder) : refuse `f64` dans `backend/src/domain/entities/*.rs` et `application/use_cases/*finance*` `*charge*` `*budget*` `*expense*`
- CI step : `grep -rE '\bf64\b' backend/src/domain/entities/ | grep -v iot | grep -v energy` doit retourner **0 lignes** post-migration

## Critères de succès global (post toutes les stories follow-up)

- [ ] 0 occurrence de `f64`/`f32` dans `backend/src/domain/entities/*.rs` excepté énergie/IoT (justifié par ADR)
- [ ] 0 occurrence dans `application/use_cases/*finance*`, `*charge*`, `*budget*`, `*expense*`, `*journal*`, `*quote*`, `*etat_date*`
- [ ] 0 colonne `DOUBLE PRECISION` dans les migrations SQL pour des montants/quotités
- [ ] Tests 4-cat (#427) passent par module migré
- [ ] ADR 0001/0002/0003 publiés et référencés
- [ ] CI hook anti-f64 actif sur les paths sensibles

---

🤖 **Audit produit par persona `rust-expert` (Claude) — Tier 2 logué.** Story EXP-002 livrée comme **document d'audit** (pas de migration code dans cette story — découpée en stories follow-up). Cette discipline (audit → propose → review → execute) suit la méthode Maury v1.1 phase 5 (Validation) : on rend la dette visible et priorisable avant de coder.

**À tagger en review** : `@gilmry` (sign-off), `code-reviewer` (cohérence cross-cutting), `security-officer` (impact PCMN compliance), `csi-analyst` (alimente CSI report mensuel comme "tech debt visible").

**Refs** : #425 (audit guardrails IA), #427 (TDD/BDD 4-cat), #429 (Tier 1/2), #430 (sprint pilote W18), `project_no-f64-in-money.md` (mémoire règle), `Maury/CHANGELOG.md` v1.1.
