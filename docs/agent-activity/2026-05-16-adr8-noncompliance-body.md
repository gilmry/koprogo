### Constat

L'audit Story B (#521) + vérification code pour Story C1 révèle que le **domaine Gouvernance** (quota copro, voting power, AGE shares, quorum) viole [ADR-0008](../adr/0008-numeric-vs-double-precision-postgresql.md) (Accepted, sign-off @gilmry) qui impose `NUMERIC` pour montants ET pourcentages (« quotités, taux TVA, taux pénalité, **voting power** » → `NUMERIC(7,4)`).

ADR-0008 listait déjà 2 de ces violations dans son propre tableau de contexte (`20240101000003_create_units.sql`, `20260312000000_add_quorum_to_meetings.sql`) — non encore remédiées.

[ADR-0009](../adr/0009-iot-energy-keep-f64.md) carve-out f64 = **IoT/énergie uniquement**. Gouvernance n'en fait pas partie → ADR-0007/0008 s'appliquent pleinement.

### Inventaire non-conformité (domaine Gouvernance)

| Élément | Type actuel | Attendu (ADR-0008) | Symptôme |
|---|---|---|---|
| `units.quota` (col) | `DOUBLE PRECISION` | `NUMERIC(10,4)` | 🔴 panic `ColumnDecode Decimal vs FLOAT8` (= **#525**) — `Unit.quota: Decimal` (ADR-0007) lu depuis colonne DOUBLE |
| `meetings.total_quotas` (col) | `DOUBLE PRECISION` | `NUMERIC(10,4)` | precision-loss quorum |
| `meetings.present_quotas` (col) | `DOUBLE PRECISION` | `NUMERIC(10,4)` | precision-loss quorum |
| `age_requests.total_shares_pct` (col) | `NUMERIC(8,6)` ✅ | — | 🔴 mais `age_request_repository_impl:31` lit `row.get::<f64>` → panic risk (code, pas schéma) |
| `age_requests.threshold_pct` (col) | `NUMERIC(8,6)` ✅ | — | idem :32 |
| `age_request_signatures.shares_pct` (col) | `NUMERIC(8,6)` ✅ | — | idem :52 |
| `AgeRequestSignature.shares_pct` (entité) | `f64` | `Decimal` | incohérent ADR-0007 |
| `etat_date.ordinary_charges_quota` (col) | `DECIMAL(5,2)` ✅ | — | 🔴 entité `f64` vs colonne DECIMAL → panic risk read |
| `etat_date.extraordinary_charges_quota` (col) | `DECIMAL(5,2)` ✅ | — | idem |
| `EtatDate.*_charges_quota` (entité) | `f64` | `Decimal` | incohérent ADR-0007 |
| `votes.voting_power` (col) | `DECIMAL(10,4)` ✅ | — | 🟠 `vote_repository` cast `::FLOAT8` puis `row.get::<f64>` → pas panic mais precision-loss non-conforme |
| `resolutions.total_voting_power_*` (col) | `DECIMAL(10,4)` ✅ | — | 🟠 `resolution_repository:323-325` params écriture `f64`→DECIMAL |

**Résumé** : 3 colonnes à migrer SQL (`units.quota`, `meetings.total_quotas`, `meetings.present_quotas`), 2 entités à passer `f64→Decimal` (`AgeRequestSignature`, `EtatDate`), ~5 repos à aligner (lecture Decimal, retrait des cast `::FLOAT8`, params Decimal).

### Cause

Migration historique pré-ADR-0008 (`20240101000003`, 2024-01) + migrations gouvernance récentes mixant les conventions. Le code aval (`unit_repository`, `age_request_repository`, `vote_repository`, `resolution_repository`, `etat_date_repository`) a accumulé des contournements (`::FLOAT8` cast, `row.get::<f64>`) au lieu de remédier la racine.

### Recette (= Story C1, #521)

Suivre le **§ Migration pattern d'ADR-0008** :

1. Migration SQL `ALTER COLUMN ... TYPE NUMERIC(...)` pour `units.quota`, `meetings.total_quotas/present_quotas` (v0.1.0 = pas de prod, pas de data loss réel — cf. memory `project_koprogo-current-state.md`).
2. Entités `AgeRequestSignature.shares_pct`, `EtatDate.*_charges_quota` : `f64 → rust_decimal::Decimal`.
3. Repos : `unit_repository`, `age_request_repository`, `etat_date_repository`, `vote_repository` (retirer `::FLOAT8`), `resolution_repository` (params `Decimal`) → lecture/écriture Decimal.
4. Tests RED-first 4 catégories par endpoint touché. **@security/@edge critiques légalement** : quorum AG Art. 3.87 §5 CC — un arrondi faux = AG contestable.
5. Ferme **#525**.

### Critères d'acceptation

- [ ] `grep -rn "DOUBLE PRECISION" backend/migrations/` → 0 sur colonnes gouvernance (quota/voting/quorum)
- [ ] `grep -rn "::FLOAT8\|get::<f64" backend/src/infrastructure/database/repositories/{unit,age_request,vote,resolution,etat_date}_repository_impl.rs` → 0
- [ ] Entités gouvernance : 0 `f64` sur champ à poids légal
- [ ] BDD 4 catégories vertes sur endpoints quorum/vote/AGE (dont scénario @security arrondi quorum)
- [ ] #525 fermé
- [ ] Conformité ADR-0008 re-vérifiée (commentaire de clôture citant les greps)

### Hors scope

- IoT/énergie : couvert ADR-0009, aucune action.
- Montants € purs (`expenses.amount` etc.) : couvert par #521 Story C2 (grappe monétaire séparée).

### Refs

- ADR-0007, ADR-0008, ADR-0009
- #521 (épic f64→Decimal), #525 (quota Decimal/FLOAT8 panic)
- memory `project_no-f64-in-money.md`
