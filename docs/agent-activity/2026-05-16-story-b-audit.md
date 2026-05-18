## Story B — Audit f64 backend repositories (output #521)

**Scope** : 35 occurrences `f64` dans `backend/src/infrastructure/database/repositories/*.rs` (post Story A merge, commit 1f59cda). Croisé avec types SQL des migrations.

### Classification

Légende : **🔴 MON-NUM** = colonne NUMERIC/DECIMAL lue en f64 → **panic risk identique #521** + perte précision · **🟠 MON-CALC** = downcast/arith monétaire f64 (perte précision PCMN) · **🟢 OK-DOUBLE** = colonne DOUBLE PRECISION/REAL → f64 correct · **⚪ OK-CAST** = cast count/jours, non monétaire

| Fichier:ligne | Variable | Colonne SQL | Type SQL | Classif |
|---|---|---|---|---|
| age_request_repository_impl:31 | total_shares_pct | age_requests.total_shares_pct | NUMERIC(8,6) | 🔴 MON-NUM |
| age_request_repository_impl:32 | threshold_pct | age_requests.threshold_pct | NUMERIC(8,6) | 🔴 MON-NUM |
| age_request_repository_impl:52 | shares_pct | age_request_signatures.shares_pct | NUMERIC(8,6) | 🔴 MON-NUM |
| budget_repository_impl:441-443 | amount | budget (Decimal→f64 downcast) | NUMERIC | 🟠 MON-CALC |
| budget_repository_impl:513 | actual_total/months*12 | projection dérivée | — | 🟠 MON-CALC |
| contract_evaluation_repository_impl:419,426 | avg_score | global_score | NUMERIC(3,2) | 🔴 MON-NUM |
| energy_campaign_repository_impl:472 | total_kwh_electricity | total_kwh_electricity | DOUBLE PRECISION | 🟢 OK-DOUBLE |
| energy_campaign_repository_impl:473 | total_kwh_gas | total_kwh_gas | DOUBLE PRECISION | 🟢 OK-DOUBLE |
| energy_campaign_repository_impl:474 | avg_kwh_per_unit | avg_kwh_per_unit | DOUBLE PRECISION | 🟢 OK-DOUBLE |
| iot_repository_impl:364 | threshold_percentage | (param, pas colonne) | — | ⚪ OK-CAST |
| iot_repository_impl:380,405 | lookback_days as f64 | cast jours | — | ⚪ OK-CAST |
| payment_reminder_repository_impl:420 | get_total_owed → f64 | SUM(amount) | NUMERIC | 🟠 MON-CALC |
| payment_reminder_repository_impl:440 | (retour f64) | montant | NUMERIC | 🟠 MON-CALC |
| payment_reminder_repository_impl:461,491,496 | amount | expenses.amount | NUMERIC | 🟠 MON-CALC |
| payment_reminder_repository_impl:555 | (f64,f64,...) total/penalties | NUMERIC | 🟠 MON-CALC |
| resolution_repository_impl:323 | total_voting_power_pour | resolutions.total_voting_power_pour | DECIMAL(10,4) | 🔴 MON-NUM |
| resolution_repository_impl:324 | total_voting_power_contre | resolutions.total_voting_power_contre | DECIMAL(10,4) | 🔴 MON-NUM |
| resolution_repository_impl:325 | total_voting_power_abstention | resolutions.total_voting_power_abstention | DECIMAL(10,4) | 🔴 MON-NUM |
| resource_booking_repository_impl:666 | total_hours_booked | SUM(EXTRACT EPOCH /3600) | double precision (calc) | 🟢 OK-DOUBLE |
| service_provider_repository_impl:380 | rating_avg | service_providers.rating_avg | NUMERIC(3,2) | 🔴 MON-NUM |
| stats_repository_impl:224,301 | pending_total | SUM(expenses.amount) | NUMERIC | 🟠 MON-CALC |
| vote_repository_impl:296,314-316 | pour/contre/abstention_power | votes.voting_power | DECIMAL(10,4) | 🔴 MON-NUM (= #525) |
| vote_repository_impl:325,344 | total_proxy_power | votes.voting_power | DECIMAL(10,4) | 🔴 MON-NUM (= #525) |

### Synthèse

- **🔴 MON-NUM (panic risk actif, 14 occ.)** : `age_request` ×3, `contract_evaluation` ×2, `resolution` ×3, `service_provider` ×1, `vote` ×5. Ce sont des colonnes NUMERIC/DECIMAL lues en `row.get::<f64,_>` → **paniquent dès qu'une ligne existe** (exactement le bug #521). `vote_repository` = la cause de **#525** déjà ouvert.
- **🟠 MON-CALC (perte précision monétaire, ~10 occ.)** : `payment_reminder` ×7, `budget` ×3, `stats` ×2. Pas de panic (try_get/unwrap_or ou downcast explicite) mais **viole `no-f64-in-money`** : montants PCMN passés en f64 → arrondi IEEE754.
- **🟢🟡 OK (f64 légitime, ~8 occ.)** : `energy` kWh (DOUBLE PRECISION, grandeur physique), `iot` casts jours/seuils, `resource_booking` heures. Pas d'action.

### Incohérence schéma détectée

Deux conventions coexistent pour "quota / voting power" :
- `units.quota` + `meetings.total_quotas/present_quotas` = **DOUBLE PRECISION** (ancien)
- `resolutions.total_voting_power_*` + `votes.voting_power` = **DECIMAL(10,4)** (récent)

Le code lit f64 partout → correct pour DOUBLE, **panic pour DECIMAL**. Décision domaine requise : la puissance de vote / quorum (Art. 3.87 §5 CC) exige-t-elle l'exactitude décimale (→ tout NUMERIC + Decimal) ou un ratio approché suffit (→ tout DOUBLE + f64) ? À trancher avant Story C.

### Recommandation découpe Story C

Découper en **3 PRs par grappe sémantique** plutôt qu'un gros diff :

1. **C1 — Gouvernance (legal-precision)** : `resolution` + `vote` + `age_request` → Decimal. Inclut résolution #525. Priorité haute (panic actif + enjeu légal quorum).
2. **C2 — Monétaire calc** : `payment_reminder` + `budget` + `stats` → Decimal end-to-end (supprimer les `.to_f64()` downcast). Priorité haute (perte précision PCMN).
3. **C3 — Évaluations** : `contract_evaluation` + `service_provider` rating/score → Decimal. Priorité moyenne (panic mais non-légal).

Pré-requis transverse : trancher l'incohérence schéma quota (issue dédiée recommandée, ou décision dans #525).

🟢 OK-DOUBLE / ⚪ OK-CAST : **hors scope** Story C, aucune action.

cc #521 #525
