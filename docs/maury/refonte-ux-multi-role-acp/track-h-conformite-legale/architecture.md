---
feature: refonte-ux-multi-role-acp/track-h-conformite-legale
phase: C (Application + Data architecture TOGAF)
status: Draft 0.1 — Maury-grade
date: 2026-06-15
authors: [Claude Opus 4.8 (drafting), @gilmry (signature pending)]
depends_on: brief.md, prd.md
---

# Architecture Track H — Conformité légale copropriété

> Phase C TOGAF. Modèle de données cible, refonte domaine, migrations réversibles, patterns. Réutilise les patterns du kit sœur `track-h-bloqueurs/` (BuildingNotConformantError, AppError variants, From<> bridges, ports/adapters).

## 1. Modèle de données cible (hybride)

```mermaid
graph TD
    ORG[Organization<br/>cabinet syndic] -. 0..1 .-> ACP
    ACP[ACP<br/>+total_tantiemes ACTE DE BASE<br/>+reserve_fund_balance<br/>+working_capital_balance<br/>+reserve_fund_waived] -->|1..N| BLD[Building / bloc<br/>total_tantiemes = sous-total<br/>+partial_association_id?]
    ACP -->|0..N| PA[PartialAssociation<br/>+total_tantiemes particulier<br/>+has_legal_personality<br/>+bce_number?]
    PA -. 0..1 .- BLD
    BLD -->|1..N| UNIT[Unit<br/>quota = quotité générale ACP<br/>+particular_quota? PA<br/>+acp_id #602]
    UNIT -->|M:N| UO[UnitOwner<br/>+ownership_type<br/>+is_voting_representative<br/>ownership_percentage]
```

### 1.1. Migrations (toutes réversibles `.down`)
- **`..._add_acps_total_tantiemes.sql`** : `ALTER TABLE acps ADD COLUMN total_tantiemes INTEGER NOT NULL DEFAULT 1000 CHECK (total_tantiemes > 0)`.
- **`..._backfill_acps_total_tantiemes.sql`** : mono-building → `acps.total_tantiemes = building.total_tantiemes` ; multi-building → `SUM(buildings.total_tantiemes)` + `DO $$ ... RAISE WARNING` listant les ACPs multi-blocs à valider (pas d'EXCEPTION). `buildings.total_tantiemes` **conservé** = sous-total bloc (commentaire SQL redéfini).
- **`..._add_acps_funds.sql`** : `reserve_fund_balance DECIMAL(14,2) DEFAULT 0`, `working_capital_balance DECIMAL(14,2) DEFAULT 0`, `reserve_fund_waived BOOLEAN DEFAULT false`.
- ~~**`..._create_partial_associations.sql`**~~ — **DIFFÉRÉ v0.2.0 (D6)**.
- ~~**`..._add_units_particular_quota.sql`**~~ — **DIFFÉRÉ v0.2.0 (D6)**.
- **`..._units_organization_to_acp.sql`** (3 étapes, story H15) : add `units.acp_id` nullable → backfill `building.acp_id` → NOT NULL + drop `organization_id`.
- **`..._add_distribution_criteria.sql`** : `charge_distributions.distribution_criteria VARCHAR DEFAULT 'value' CHECK IN ('value','utility','mixed')` (ou sur `expenses`).
- **`..._add_meeting_owner_counts.sql`** : `meetings.present_owners_count INT NULL`, `meetings.total_owners_count INT NULL`.
- **`..._add_unit_owner_voting.sql`** : `unit_owners.ownership_type VARCHAR DEFAULT 'full_owner' CHECK IN ('full_owner','usufruct','bare_owner','indivisaire','emphyteote','superficiaire')`, `unit_owners.is_voting_representative BOOLEAN DEFAULT false`.
- **Budget (H11)** : DB déjà `DECIMAL(12,2)` — **pas de migration**, fix code only.

## 2. Conformité ACP-level (CL1)

### 2.1. Domaine — `AcpMetrics` + `Acp::assert_conformant`
```rust
// backend/src/domain/entities/acp.rs (extension)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpMetrics {
    pub units_count: i32,        // Σ units de TOUS les buildings de l'ACP
    pub quota_sum: Decimal,      // Σ quota (quotité générale) tous blocs
    pub buildings_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpNotConformantError {
    pub acp_id: Uuid,
    pub quota_delta: Decimal,    // total_tantiemes - quota_sum
    pub quota_basis: i32,        // acps.total_tantiemes (acte de base)
    pub units_delta: i32,        // si total_units agrégé connu, sinon 0
}

impl Acp {
    pub fn is_conformant(&self, m: &AcpMetrics) -> bool {
        m.quota_sum == Decimal::from(self.total_tantiemes)
    }
    pub fn assert_conformant(&self, m: &AcpMetrics) -> Result<(), AcpNotConformantError> {
        if !self.is_conformant(m) {
            return Err(AcpNotConformantError {
                acp_id: self.id,
                quota_delta: Decimal::from(self.total_tantiemes) - m.quota_sum,
                quota_basis: self.total_tantiemes,
                units_delta: 0,
            });
        }
        Ok(())
    }
}
```
Structurellement identique à `BuildingNotConformantError` (kit bloqueurs `building.rs`). `From<AcpNotConformantError> for AppError` → nouveau variant `AppError::AcpNotConformant` (422, payload `ACP_NOT_CONFORMANT`) + `From<...> for String` (bridge legacy, pattern `error.rs`).

### 2.2. Port + adapter
```rust
// application/ports/acp_repository.rs (extension)
async fn find_by_id_with_metrics(&self, acp_id: Uuid)
    -> Result<Option<(Acp, AcpMetrics)>, AppError>;
```
SQL agrégé (JOIN multi-building) :
```sql
SELECT a.*, 
  COALESCE(COUNT(u.id)::INT,0)               AS units_count,
  COALESCE(SUM(u.quota::NUMERIC),0::NUMERIC) AS quota_sum,
  COALESCE(COUNT(DISTINCT b.id)::INT,0)      AS buildings_count
FROM acps a
LEFT JOIN buildings b ON b.acp_id = a.id
LEFT JOIN units u     ON u.building_id = b.id
WHERE a.id = $1
GROUP BY a.id;
```

### 2.3. Bascule des 4 gates (H7, retravaille WP-H2)
Les helpers `assert_building_conformant(building_id)` (expense/call_for_funds/charge_distribution/etat_date) deviennent `assert_acp_conformant(building_id)` : résoudre `building.acp_id` → `acp_repo.find_by_id_with_metrics(acp_id)` → `acp.assert_conformant(&metrics)?`. Signatures use-cases inchangées (bridge `?`→String/AppError). `Building::assert_conformant` (H1) **conservé** pour les sous-totaux blocs/associations partielles.

## 3. Quorum double + représentant de vote (CL3)

### 3.1. Quorum double (H9, étend H3)
```rust
// meeting.rs
pub fn validate_quorum(
    &mut self,
    present_quotas: Decimal, total_quotas: Decimal,
    present_owners: i32, total_owners: i32,   // NOUVEAU : têtes
) -> Result<bool, String> {
    let quotas_ok = present_quotas > total_quotas / dec!(2);
    let heads_ok  = present_owners * 2 > total_owners;   // > moitié strict
    Ok(quotas_ok && heads_ok)
}
```
`MeetingCompletionChecklist` (kit bloqueurs H3) gagne `attended_owners: i32`, `total_owners: i32` ; `MissingInvariant::HeadCountQuorumNotReached { attended_owners, total_owners }`. Le `meeting_completion_checker_impl` ajoute `COUNT(DISTINCT owner_id)` (total) et les présents (depuis présences AG).

### 3.2. Représentant de vote / suspension (H17)
```rust
// unit.rs ou unit_owner.rs — domaine pur
pub enum VotingRightStatus { Active, Suspended }

impl Unit {
    pub fn voting_right_status(&self, owners: &[UnitOwner]) -> VotingRightStatus {
        let multi_or_dismembered = owners.len() > 1
            || owners.iter().any(|o| o.ownership_type != OwnershipType::FullOwner);
        let has_representative = owners.iter().any(|o| o.is_voting_representative);
        if multi_or_dismembered && !has_representative {
            VotingRightStatus::Suspended
        } else {
            VotingRightStatus::Active
        }
    }
}
```
Erreur typée `VotingRightSuspended { unit_id }` → 422 `VOTING_RIGHT_SUSPENDED`. Gate dans le use-case d'enregistrement de vote (H10). Un lot `Suspended` ne compte ni en têtes ni en quotités pour le quorum (ajuster le checker).

### 3.3. Gates votes (H10)
- `meeting.check_quorum_for_voting()?` (existe `meeting.rs`, non branché) appelé avant `record_vote`.
- `vote.rs validate_proxy_mandate()` (existe, ≤3 / ≤10%) appelé avec un port comptant les mandats/quotités déjà délégués.

## 4. Finances (CL4)

### 4.1. Budget Decimal (H11)
`budget.rs` : `f64` → `Decimal` sur `ordinary_budget`, `extraordinary_budget`, `total_budget`, `monthly_provision_amount`. DB déjà Decimal ; entity/DTO/repo lisent en Decimal. `monthly_provision = total_budget / dec!(12)`.

### 4.2. DistributionCriteria (H12)
```rust
pub enum DistributionCriteria { Value, Utility, Mixed }
```
`charge_distribution` : la part effective d'un copropriétaire = `(unit.quota / acp.total_tantiemes) * unit_owner.ownership_percentage` pour `Value` ; pour `Utility`, base alternative (superficie/usage) ; `Mixed` = combinaison votée. Clarifie la confusion `quota_percentage` (part du lot) vs `ownership_percentage` (part du copropriétaire dans le lot).

### 4.3. Fonds réserve/roulement (H13)
```rust
impl Acp {
    pub fn assert_reserve_fund_compliant(&self, ordinary_charges_n1: Decimal)
        -> Result<(), ReserveFundInsufficient> {
        if self.reserve_fund_waived { return Ok(()); }
        let min = ordinary_charges_n1 * dec!(0.05);
        if self.reserve_fund_balance < min {
            return Err(ReserveFundInsufficient { acp_id: self.id, required: min, current: self.reserve_fund_balance });
        }
        Ok(())
    }
}
```
`call_for_funds.fund_type ∈ {ordinary, working_capital, reserve}` ; comptes distincts (Art. 3.86 §3).

## 5. Associations partielles (CL5) · ⛔ DIFFÉRÉ v0.2.0 (D6 @gilmry)

> NON implémenté en v0.1.0 (décision PO 2026-06-15). Les migrations `partial_associations`, `buildings.partial_association_id`, `units.particular_quota` (cf. §1.1) sont **différées**. Conception conservée pour v0.2.0.

- Entité `PartialAssociation { id, acp_id, name, has_legal_personality, bce_number, total_tantiemes }` ; `building.partial_association_id`.
- **Quotités 2 niveaux** : `unit.quota` = quotité générale (dénominateur ACP) ; `unit.particular_quota` (nullable) = quotité dans les communs particuliers de la PA (dénominateur PA).
- `PartialAssociation::assert_conformant` : `Σ particular_quota des units des buildings de la PA == PA.total_tantiemes`.
- Charges PA : `charge_distribution` peut cibler une `partial_association_id` (répartition sur `particular_quota` du périmètre).
- Invariant : `has_legal_personality=true` interdit si `acp.has_legal_personality=false` (à exposer sur ACP — champ dérivé ou ajouté ; trancher en H0-ADR).

## 6. Patterns tests (TDD/BDD 4-cat RED-first)

### 6.1. Domaine inline (exemple ACP)
```rust
#[cfg(test)] mod assert_conformant_tests {
    // @happy mono 1000 ; @happy multi-blocs 10000 (3 buildings somment à 10000) ;
    // @edge dérive 0.1 ; @security metrics forgé ; @negative empty
}
```
### 6.2. BDD features (nouvelles)
`validate_before_compute_acp.feature` (4 use-cases × ACP mono/multi-blocs), `quorum_double.feature` (têtes+quotités bornes), `voting_right_suspension.feature` (usufruit/indivision), `reserve_fund.feature` (5%), `partial_association.feature` (quotités 2 niveaux). Seeds via use-cases (mémoire `world-model-seed`) ; toujours 2 actes de référence (1000 ET 10000).

## 7. Frontend (impact)
- `<AcpConformityBanner>` (réutilise pattern `ConformityBanner` H1) — niveau ACP.
- `<VotingSuspendedBadge>` sur un lot suspendu ; bouton vote `disabled` + tooltip.
- `<ReserveFundIndicator>` (≥5%) ; toast `RESERVE_FUND_INSUFFICIENT`.
- i18n FR/NL/EN/DE pour tous messages.

## 8. Risques techniques
Voir `brief.md §7`. Points durs : backfill multi-building (WARNING+admin), rétro-compat seeds (stash repris en H7/H9), quotités 2 niveaux (MVP `particular_quota` nullable), `present_quotas` DOUBLE PRECISION (ne pas aggraver).

## 9. Signature
```
Mary (Brief) : pending @gilmry
John (PRD)   : pending @gilmry
Winston (Arch): Draft v0.1 — signature pending @gilmry
```
→ Signer débloque les Stories (`stories.md`).
