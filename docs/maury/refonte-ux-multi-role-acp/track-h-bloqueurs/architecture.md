---
feature: refonte-ux-multi-role-acp/track-h-bloqueurs
phase: C (Application + Data architecture TOGAF)
status: SIGNED v1.0 par @gilmry 2026-06-15
date: 2026-06-15
authors: [Claude Opus 4.7 (drafting), @gilmry (signature 2026-06-15)]
depends_on: brief.md (SIGNED v1.0 2026-06-15), prd.md (SIGNED v1.0 2026-06-15)
---

# Architecture Track H — Bloqueurs légaux v0.1.0

> Phase C TOGAF — décrit l'application architecture (Domain types, Use case patterns, FE patterns) et data architecture (DB queries pour BuildingMetrics + MeetingCompletionChecklist). Pour les stories briefables, voir `stories.md`.

## 1. Stack confirmé

- **Backend** : Rust + Actix-web 4 + sqlx (déjà en place).
- **Domain** : `building.rs`, `meeting.rs`, nouveaux types `BuildingNotConformantError`, `MeetingNotCompletableError`, `MissingInvariant`.
- **Frontend** : Astro static + Svelte 5 runes. Composants atomiques nouveaux : `ConformityBanner.svelte`, `MissingInvariantsList.svelte`.
- **Tests** : sqlx integration + cucumber-rs BDD + Vitest + Playwright (project chromium).

## 2. Architecture hexagonale — pattern erreur typée

### 2.1. Domain layer — `BuildingNotConformantError`

```rust
// backend/src/domain/entities/building.rs (extension + BUG FIX)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingNotConformantError {
    pub building_id: Uuid,
    pub units_delta: i32,     // self.total_units - metrics.units_count
    pub quota_delta: Decimal, // Decimal::from(self.total_tantiemes) - metrics.quota_sum
    pub quota_basis: i32,     // self.total_tantiemes (acte de base — 1000, 10000, autre)
}

impl Building {
    /// BUG FIX : utilise `self.total_tantiemes` au lieu de la constante
    /// `CONFORMANT_QUOTA_TOTAL = dec!(1000)`. Les immeubles dont l'acte de
    /// base définit 10000 (ou autre) étaient mal classifiés non-conformes.
    pub fn is_conformant(&self, metrics: &BuildingMetrics) -> bool {
        Self::compute_is_conformant(self.total_units, self.total_tantiemes, metrics)
    }

    /// Variante statique enrichie (pour tests purs).
    pub fn compute_is_conformant(
        declared_units: i32,
        total_tantiemes: i32,
        metrics: &BuildingMetrics,
    ) -> bool {
        metrics.units_count == declared_units
            && metrics.quota_sum == Decimal::from(total_tantiemes)
    }

    pub fn quota_delta(&self, metrics: &BuildingMetrics) -> Decimal {
        Decimal::from(self.total_tantiemes) - metrics.quota_sum
    }

    pub fn assert_conformant(
        &self,
        metrics: &BuildingMetrics,
    ) -> Result<(), BuildingNotConformantError> {
        if !self.is_conformant(metrics) {
            return Err(BuildingNotConformantError {
                building_id: self.id,
                units_delta: self.total_units - metrics.units_count,
                quota_delta: self.quota_delta(metrics),
                quota_basis: self.total_tantiemes,
            });
        }
        Ok(())
    }
}
```

**Bug fix capturé** : `CONFORMANT_QUOTA_TOTAL = dec!(1000)` constante **supprimée** ; les call-sites doivent migrer vers `self.total_tantiemes` ou `Decimal::from(building.total_tantiemes)`. Note `validate_unit_shares_distribution()` (ligne 246) qui hard-code `> 1000` doit aussi être corrigée pour utiliser le param `total_tantiemes`.

**Invariants** :
- Calcul Decimal strict (déjà respecté).
- `assert_conformant()` ne lit JAMAIS DB — receveur `BuildingMetrics` (struct pure data) que use-case charge en amont.
- Pas de dépendance sqlx/actix dans `building.rs`.
- `quota_basis` exposé dans l'erreur → FE peut afficher "9975 / 10000" pas "9975 / 1000".

### 2.2. Application layer — `From<>` bridges

```rust
// backend/src/application/error.rs (extension)

impl From<BuildingNotConformantError> for AppError {
    fn from(err: BuildingNotConformantError) -> Self {
        AppError::Validation {
            field: "building".to_string(),
            message: format!(
                "Building {} not conformant: {} units missing, quota delta {} / {} (acte de base)",
                err.building_id, err.units_delta, err.quota_delta, err.quota_basis
            ),
            details: Some(json!({
                "code": "BUILDING_NOT_CONFORMANT",
                "building_id": err.building_id,
                "units_delta": err.units_delta,
                "quota_delta": err.quota_delta.to_string(),
                "quota_basis": err.quota_basis,
            })),
        }
    }
}

impl From<BuildingNotConformantError> for String {
    fn from(err: BuildingNotConformantError) -> Self {
        format!(
            "Building {} not conformant: units delta {} quota delta {}",
            err.building_id, err.units_delta, err.quota_delta
        )
    }
}
```

**Pourquoi 2 `From<>` ?**
- `for AppError` : nouveaux use-cases (cf. mémoire `validate-before-compute`) renvoient `Result<_, AppError>` → mapping 422 propre.
- `for String` : use-cases legacy (call_for_funds, expense, ...) renvoient `Result<_, String>` historique. Pas de réécriture globale (hors-scope WBS) ; just bridge pour permettre `?` avec `assert_conformant()`. Mémoire `validate-before-compute` explicite ce trade-off.

### 2.3. Use case pattern — validate-before-compute

```rust
// backend/src/application/use_cases/call_for_funds_use_cases.rs (extension)

pub async fn send_call_for_funds(&self, id: Uuid) -> Result<CallForFunds, String> {
    let call = self.repository.find_by_id(id).await?
        .ok_or_else(|| "CallForFunds not found".to_string())?;

    // [NEW] Validate-before-compute gate
    let building = self.building_repository.find_by_id(call.building_id).await?
        .ok_or_else(|| "Building not found".to_string())?;
    let metrics = self.building_metrics_repository.compute(call.building_id).await?;
    building.assert_conformant(&metrics)?; // ? converts to String via From<>

    // [NEW] Audit on success too? — non, audit déclenché par From<> branch dans handler global
    // [...existing logic continue calculation...]
}
```

**Conventions** :
- Pre-check **avant** toute mutation/calcul (pas de side-effect avant validation).
- Charge `building` + `metrics` en parallèle (pas en série) si signature use-case permet (futures::join).
- Audit : intercepté par middleware Actix sur 422 response avec `code == BUILDING_NOT_CONFORMANT` → insert security_incident async (mémoire `validate-before-compute`).

### 2.4. Use cases impactés (FR-H2)

| Use case | File | Pre-check à ajouter | Result<E> actuel |
|---|---|---|---|
| `expense_use_cases::create_expense` | application/use_cases/expense_use_cases.rs | `building.assert_conformant(metrics)?` | `Result<_, AppError>` ✓ |
| `call_for_funds_use_cases::send_call_for_funds` | application/use_cases/call_for_funds_use_cases.rs | idem (via `for String`) | `Result<_, String>` legacy |
| `charge_distribution_use_case::compute_distribution` | application/use_cases/charge_distribution_use_case.rs | idem | `Result<_, AppError>` (post WP-A4) ✓ |
| `etat_date_use_cases::generate_etat_date` | application/use_cases/etat_date_use_cases.rs | idem (via `for String`) | `Result<_, String>` legacy |

## 3. Pattern Meeting.assert_can_complete()

### 3.1. Domain — types

```rust
// backend/src/domain/entities/meeting.rs (extension)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissingInvariant {
    ConvocationsNotSent,
    VotesNotClosed { open_resolutions: i32 },
    AttendanceNotRecorded,
    QuorumNotReached { attended_quotas: Decimal, total_quotas: Decimal },
    MinutesDraftMissing,
}

#[derive(Debug, Clone)]
pub struct MeetingCompletionChecklist {
    pub convocations_sent: bool,
    pub open_resolutions: i32,    // 0 = all closed
    pub attendance_recorded: bool,
    pub attended_quotas: Decimal, // somme quotas présents+représentés
    pub total_quotas: Decimal,    // somme quotas building (déjà 1000 si conforme)
    pub minutes_draft_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeetingNotCompletableError {
    pub meeting_id: Uuid,
    pub missing: Vec<MissingInvariant>,
}

impl Meeting {
    pub fn assert_can_complete(
        &self,
        checklist: &MeetingCompletionChecklist,
    ) -> Result<(), MeetingNotCompletableError> {
        let mut missing = Vec::new();

        if !checklist.convocations_sent {
            missing.push(MissingInvariant::ConvocationsNotSent);
        }
        if checklist.open_resolutions > 0 {
            missing.push(MissingInvariant::VotesNotClosed {
                open_resolutions: checklist.open_resolutions,
            });
        }
        if !checklist.attendance_recorded {
            missing.push(MissingInvariant::AttendanceNotRecorded);
        }

        // Quorum > 50% (Art. 3.87 §4 base — majorité simple présents+représentés)
        let half = checklist.total_quotas / dec!(2);
        if checklist.attended_quotas <= half {
            missing.push(MissingInvariant::QuorumNotReached {
                attended_quotas: checklist.attended_quotas,
                total_quotas: checklist.total_quotas,
            });
        }

        if !checklist.minutes_draft_exists {
            missing.push(MissingInvariant::MinutesDraftMissing);
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(MeetingNotCompletableError {
                meeting_id: self.id,
                missing,
            })
        }
    }
}

impl From<MeetingNotCompletableError> for AppError {
    fn from(err: MeetingNotCompletableError) -> Self {
        AppError::Validation {
            field: "meeting".to_string(),
            message: format!(
                "Meeting {} not completable: {} missing invariant(s)",
                err.meeting_id, err.missing.len()
            ),
            details: Some(json!({
                "code": "MEETING_NOT_COMPLETABLE",
                "meeting_id": err.meeting_id,
                "missing": err.missing.iter().map(|m| match m {
                    MissingInvariant::ConvocationsNotSent => json!({"type": "ConvocationsNotSent"}),
                    MissingInvariant::VotesNotClosed { open_resolutions } =>
                        json!({"type": "VotesNotClosed", "open_resolutions": open_resolutions}),
                    MissingInvariant::AttendanceNotRecorded => json!({"type": "AttendanceNotRecorded"}),
                    MissingInvariant::QuorumNotReached { attended_quotas, total_quotas } =>
                        json!({
                            "type": "QuorumNotReached",
                            "attended_quotas": attended_quotas.to_string(),
                            "total_quotas": total_quotas.to_string(),
                        }),
                    MissingInvariant::MinutesDraftMissing => json!({"type": "MinutesDraftMissing"}),
                }).collect::<Vec<_>>(),
            })),
        }
    }
}
```

### 3.2. Port — `MeetingCompletionCheckerPort`

```rust
// backend/src/application/ports/meeting_completion_checker.rs (NOUVEAU)

#[async_trait]
pub trait MeetingCompletionCheckerPort: Send + Sync {
    async fn build_checklist(
        &self,
        meeting_id: Uuid,
    ) -> Result<MeetingCompletionChecklist, AppError>;
}
```

Adapter PostgreSQL :

```rust
// backend/src/infrastructure/database/repositories/meeting_completion_checker_impl.rs (NOUVEAU)

pub struct MeetingCompletionCheckerImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
impl MeetingCompletionCheckerPort for MeetingCompletionCheckerImpl {
    async fn build_checklist(
        &self,
        meeting_id: Uuid,
    ) -> Result<MeetingCompletionChecklist, AppError> {
        // 1 query agrégée: COUNT convocations.sent, COUNT resolutions WHERE status='open',
        //                 EXISTS attendance, SUM attended_quotas, total_quotas, EXISTS minutes
        // (cf. SQL détaillé en stories.md H3)
        // ...
    }
}
```

### 3.3. Use case `complete_meeting()` extension

```rust
// backend/src/application/use_cases/meeting_use_cases.rs (extension)

pub async fn complete_meeting(
    &self,
    meeting_id: Uuid,
    _attendees_count: i32,
) -> Result<Meeting, AppError> {
    let mut meeting = self.repository.find_by_id(meeting_id).await?
        .ok_or_else(|| AppError::NotFound {/*...*/})?;

    // [NEW] Validate-before-compute gate Art. 3.87 §3-5 CC
    let checklist = self.completion_checker.build_checklist(meeting_id).await?;
    meeting.assert_can_complete(&checklist)?; // ? via From<> → 422

    // Conserve existing complete() state machine (mémoire CRITICAL.md : don't break BC)
    meeting.complete_internal()?; // renomme l'ancien complete() en complete_internal()
    self.repository.save(&meeting).await?;
    Ok(meeting)
}
```

**Note importante** : on n'utilise plus le param `attendees_count` (déprécié — la vraie source = checklist.attended_quotas). Garder param dans signature pour compat handler, mais commentaire `// deprecated, voir checklist`.

## 4. Frontend pattern — `ConformityBanner`

### 4.1. Composant atomique `ConformityBanner.svelte`

```svelte
<!-- frontend/src/lib/components/shared/ConformityBanner.svelte -->
<script lang="ts">
  import type { ConformityStatus } from '$lib/types/conformity';
  import { t } from '$lib/i18n';

  let { status, buildingId, buildingName }: {
    status: ConformityStatus;
    buildingId: string;
    buildingName: string;
  } = $props();

  let unitsDeltaLabel = $derived(
    status.units_delta > 0
      ? t('conformity.units_missing', { n: status.units_delta })
      : t('conformity.units_extra', { n: Math.abs(status.units_delta) })
  );
</script>

{#if !status.is_conformant}
  <div
    role="alert"
    aria-live="polite"
    class="bg-red-50 border-l-4 border-red-500 text-red-800 p-4 my-4 flex items-start gap-3"
    data-testid="conformity-banner"
  >
    <span aria-hidden="true" class="text-xl">⚠️</span>
    <div class="flex-1">
      <strong>{t('conformity.banner_title', { name: buildingName })}</strong>
      <ul class="mt-1 list-disc list-inside text-sm">
        {#if status.units_delta !== 0}
          <li data-testid="conformity-units-delta">{unitsDeltaLabel}</li>
        {/if}
        {#if status.quota_delta !== '0' && status.quota_delta !== '0.0'}
          <li data-testid="conformity-quota-delta">
            {t('conformity.quota_off', { delta: status.quota_delta })}
          </li>
        {/if}
      </ul>
      <p class="mt-1 text-sm">{t('conformity.contact_admin')}</p>
    </div>
  </div>
{/if}
```

### 4.2. Pattern d'usage dans pages

```svelte
<!-- frontend/src/components/buildings/BuildingDetail.svelte (extension) -->
<script>
  // ... existing imports ...
  import ConformityBanner from '$lib/components/shared/ConformityBanner.svelte';

  // Conformity status loaded via existing building_response_dto
  let conformityStatus = $derived({
    is_conformant: building.is_conformant,
    units_delta: building.total_units - building.units_count,
    quota_delta: building.quota_delta,
  });

  let canCompute = $derived(conformityStatus.is_conformant);
</script>

<ConformityBanner
  status={conformityStatus}
  buildingId={building.id}
  buildingName={building.name}
/>

<Button
  disabled={!canCompute}
  aria-disabled={!canCompute}
  on:click={handleNewExpense}
  data-testid="expense-create-button"
>
  {t('expenses.create')}
</Button>
```

### 4.3. Pattern toast 422 narratif

```typescript
// frontend/src/lib/utils/conformity.ts (NOUVEAU)

export function showConformityToast(error: unknown): void {
  if (isConformityError(error)) {
    const { details } = error;
    toast.error({
      title: t('conformity.toast_title'),
      message: t('conformity.toast_message', {
        units: details.units_delta,
        quota: details.quota_delta,
      }),
      duration: 8000, // 8s narratif (mémoire toast verbose pour erreurs)
    });
    return;
  }
  // ... fallback générique ...
}
```

### 4.4. Composant `MissingInvariantsList.svelte` (FR-H3 FE)

```svelte
<!-- frontend/src/lib/components/shared/MissingInvariantsList.svelte -->
<script lang="ts">
  import { t } from '$lib/i18n';
  import type { MissingInvariant } from '$lib/types/meeting';

  let { invariants }: { invariants: MissingInvariant[] } = $props();
</script>

<ul class="space-y-2" data-testid="missing-invariants-list">
  {#each invariants as inv}
    <li class="flex items-start gap-2" data-testid="missing-invariant-{inv.type.toLowerCase()}">
      <span aria-hidden="true" class="text-red-600">✗</span>
      <span>{t(`meeting.missing.${inv.type}`, inv)}</span>
    </li>
  {/each}
</ul>
```

## 5. Data architecture

### 5.1. Building metrics — déjà existant

```sql
-- Calculé dans building_metrics_repository_impl.rs (déjà en place)
SELECT
  COUNT(*) FILTER (WHERE building_id = $1) AS units_count,
  COALESCE(SUM(quota) FILTER (WHERE building_id = $1), 0) AS quota_sum
FROM units;
```

Pas de nouvelle table, pas de migration.

### 5.2. Meeting completion checklist — query agrégée

```sql
-- backend/src/infrastructure/database/repositories/meeting_completion_checker_impl.rs

SELECT
  -- Convocations sent (Art. 3.87 §3)
  EXISTS(SELECT 1 FROM convocations WHERE meeting_id = $1 AND sent_at IS NOT NULL) AS convocations_sent,
  -- Open resolutions (votes pas clôturés Art. 3.87 §4)
  (SELECT COUNT(*) FROM resolutions WHERE meeting_id = $1 AND status = 'open')::int AS open_resolutions,
  -- Attendance recorded (Art. 3.87 §5)
  EXISTS(SELECT 1 FROM attendances WHERE meeting_id = $1) AS attendance_recorded,
  -- Attended quotas (présents + représentés)
  COALESCE(
    (SELECT SUM(u.quota)
     FROM attendances a
     JOIN units u ON u.id = a.unit_id
     WHERE a.meeting_id = $1 AND a.status IN ('present', 'represented')
    ), 0
  ) AS attended_quotas,
  -- Total quotas du building du meeting
  COALESCE(
    (SELECT SUM(quota) FROM units WHERE building_id = (SELECT building_id FROM meetings WHERE id = $1)), 0
  ) AS total_quotas,
  -- Minutes draft exists
  EXISTS(SELECT 1 FROM minutes WHERE meeting_id = $1) AS minutes_draft_exists
;
```

**Performance** : 1 query, 6 sous-queries indépendantes. Index existants sur `meeting_id` font le job. ≤ 5ms attendu sur Postgres bêta.

### 5.3. Security incident — extension type enum

```sql
-- backend/migrations/YYYYMMDD_security_incident_track_h_types.sql (NOUVEAU)
-- type column si CHECK constraint :
ALTER TABLE security_incidents
  DROP CONSTRAINT IF EXISTS security_incidents_type_check;

ALTER TABLE security_incidents
  ADD CONSTRAINT security_incidents_type_check
  CHECK (type IN (
    'AUTH_FAIL', 'AUTH_BRUTE_FORCE', 'TOKEN_EXPIRED', 'INVALID_TOKEN',
    'PERMISSION_DENIED', 'RBAC_VIOLATION', 'CROSS_ORG_ACCESS',
    'BUILDING_NOT_CONFORMANT',    -- NOUVEAU
    'MEETING_NOT_COMPLETABLE'     -- NOUVEAU
  ));
```

Note : si `type` est `TEXT` libre (pas check), pas de migration. À vérifier dans story H1 brief.

## 6. Tests architecture

### 6.1. Domain unit tests 4-cat — pattern

```rust
// backend/src/domain/entities/building.rs (in-module #[cfg(test)])

#[cfg(test)]
mod assert_conformant_tests {
    use super::*;

    fn make_building(total: i32) -> Building {
        Building::new(...).unwrap() // helpers existants
    }

    // @happy — building conforme passe
    #[test]
    fn happy_returns_ok_when_conformant() {
        let b = make_building(10);
        let m = BuildingMetrics { units_count: 10, quota_sum: dec!(1000) };
        assert!(b.assert_conformant(&m).is_ok());
    }

    // @edge — quota dérive de 0.1 = NON conforme strict (Decimal)
    #[test]
    fn edge_quota_off_by_one_tenth_fails() {
        let b = make_building(10); // total_tantiemes = 1000 par défaut
        let m = BuildingMetrics { units_count: 10, quota_sum: dec!(999.9) };
        let err = b.assert_conformant(&m).unwrap_err();
        assert_eq!(err.quota_delta, dec!(0.1));
        assert_eq!(err.units_delta, 0);
        assert_eq!(err.quota_basis, 1000);
    }

    // @edge — building avec acte de base 10000 (cas immeuble à 182 lots fractionnés)
    #[test]
    fn edge_quota_basis_10000_conformant() {
        let b = Building::new(/* ..., */ 182, 10000, None).unwrap();
        let m = BuildingMetrics { units_count: 182, quota_sum: dec!(10000) };
        assert!(b.assert_conformant(&m).is_ok());
    }

    // @edge — building 10000 drift 25.0 (1 lot manquant moyen)
    #[test]
    fn edge_quota_basis_10000_drift() {
        let b = Building::new(/* ..., */ 182, 10000, None).unwrap();
        let m = BuildingMetrics { units_count: 181, quota_sum: dec!(9975) };
        let err = b.assert_conformant(&m).unwrap_err();
        assert_eq!(err.units_delta, 1);
        assert_eq!(err.quota_delta, dec!(25));
        assert_eq!(err.quota_basis, 10000);
    }

    // @security — tampering metrics ne change pas vérité (assert_conformant pur)
    #[test]
    fn security_metrics_tampering_detected() {
        let b = make_building(10);
        let m = BuildingMetrics { units_count: 9, quota_sum: dec!(1000) }; // forgé
        let err = b.assert_conformant(&m).unwrap_err();
        assert_eq!(err.units_delta, 1);
    }

    // @negative — empty metrics
    #[test]
    fn negative_empty_metrics() {
        let b = make_building(10);
        let m = BuildingMetrics { units_count: 0, quota_sum: dec!(0) };
        let err = b.assert_conformant(&m).unwrap_err();
        assert_eq!(err.units_delta, 10);
        assert_eq!(err.quota_delta, dec!(1000));
    }
}
```

### 6.2. BDD feature pattern — `validate_before_compute.feature`

```gherkin
# backend/tests/features/validate_before_compute.feature
Feature: Validate-before-compute on use-cases (Track H FR-H2)
  Background:
    # Building millièmes typique (1000)
    Given a conformant building "Conformant Towers" with 10 units summing to 1000 thousandths (total_tantiemes=1000)
    # Building avec acte de base 10000 (lots fractionnés finement)
    And a conformant building "Big Tower 182" with 182 units summing to 10000 (total_tantiemes=10000)
    # Drift typique
    And a non-conformant building "Drift Manor" with 9 units summing to 997.5 (total_tantiemes=1000)
    # Drift sur acte 10000
    And a non-conformant building "Drift Tower" with 181 units summing to 9975 (total_tantiemes=10000)

  @happy
  Scenario Outline: <use_case> succeeds on conformant building (any acte de base)
    When syndic <action> on building "<building>"
    Then the response is 200 OK

    Examples:
      | building            | use_case                     | action                       |
      | Conformant Towers   | create_expense              | submits a new expense        |
      | Big Tower 182       | create_expense              | submits a new expense        |
      | Conformant Towers   | send_call_for_funds         | sends a call for funds       |
      | Big Tower 182       | send_call_for_funds         | sends a call for funds       |

      | Conformant Towers   | compute_charge_distribution | computes charge distribution |
      | Big Tower 182       | compute_charge_distribution | computes charge distribution |
      | Conformant Towers   | generate_etat_date          | generates an etat de date    |
      | Big Tower 182       | generate_etat_date          | generates an etat de date    |

  @security
  Scenario Outline: <use_case> blocked on non-conformant building (bypass attempted, any quota_basis)
    When syndic <action> on building "<building>"
    Then the response is 422 BUILDING_NOT_CONFORMANT
    And the response details contain units_delta <units_delta> and quota_delta "<quota_delta>" and quota_basis <quota_basis>
    And a security_incident BUILDING_NOT_CONFORMANT is logged

    Examples:
      | building       | use_case                     | action                        | units_delta | quota_delta | quota_basis |
      | Drift Manor    | create_expense              | submits a new expense         | 1           | 2.5         | 1000        |
      | Drift Tower    | create_expense              | submits a new expense         | 1           | 25          | 10000       |
      | Drift Manor    | send_call_for_funds         | sends a call for funds        | 1           | 2.5         | 1000        |
      | Drift Tower    | send_call_for_funds         | sends a call for funds        | 1           | 25          | 10000       |
      | Drift Manor    | compute_charge_distribution | computes charge distribution  | 1           | 2.5         | 1000        |
      | Drift Tower    | compute_charge_distribution | computes charge distribution  | 1           | 25          | 10000       |
      | Drift Manor    | generate_etat_date          | generates an etat de date     | 1           | 2.5         | 1000        |
      | Drift Tower    | generate_etat_date          | generates an etat de date     | 1           | 25          | 10000       |

  @edge
  Scenario: building becomes conformant after admin fix
    Given building "Drift Manor" is non-conformant
    When admin adds the missing unit with quota 2.5
    Then "Drift Manor" is conformant
    And syndic create_expense succeeds with 200

  @negative
  Scenario: building does not exist
    When syndic submits expense on non-existent building
    Then the response is 404 NOT_FOUND
```

## 7. Risques techniques + mitigations

| Risque | Mitigation |
|---|---|
| Story H2 use-case legacy `Result<_, String>` refacto fait exploser scope | `From<BuildingNotConformantError> for String` + commentaire `// hérité, hors scope` ; pas de réécriture |
| Quorum check existant dans `complete()` doublé par `assert_can_complete()` | Renommer ancien `complete()` → `complete_internal()` (state-transition pure) ; `assert_can_complete()` = nouveau gate complet ; use-case appelle séquentiellement |
| FE `ConformityBanner` non-rendu si conformity_status absent du DTO | Tous endpoints buildings exposent déjà `is_conformant + units_count + total_units + quota_sum + quota_delta` (cartographie 2026-06-15). Si endpoint manque → typage frontend `api.d.ts` régénéré (gate Contract Types Check de CI). |
| Audit `security_incident` augmente fortement le volume rows | v0.1.0 bêta fermée (~10 syndics × ~5 tentatives/mois max). Volume négligeable. Index sur `created_at` existant. |
| Migration SQL `security_incidents_type_check` casse les rows existants | Vérifier d'abord si `type` est CHECK ou TEXT libre. Si CHECK, migration `DROP + ADD` sans data backfill (toutes valeurs déjà conformes). |

## 8. Signature

```
Mary (Brief)         : SIGNED v1.0 par @gilmry 2026-06-15
John (PRD)           : SIGNED v1.0 par @gilmry 2026-06-15
Winston (Arch)       : SIGNED v1.0 par @gilmry 2026-06-15
```

→ Stories débloquées (`stories.md`).
