//! Track H Story H3 — Adapter sqlx pour `MeetingCompletionCheckerPort`.
//!
//! Une seule query SQL agrégée construit la checklist Art. 3.87 §3-5 CC :
//! convocations envoyées, résolutions en cours, présences enregistrées,
//! quotas (présents+représentés + total building), minutes draft.
//!
//! **Performance** : 6 sous-queries indépendantes, ≤ 5ms sur Postgres bêta.
//!
//! **Types** : `meetings.present_quotas` et `units.quota` sont `NUMERIC` en DB
//! depuis `20260516000000_alter_governance_to_numeric.sql` (WP-A1), et lus
//! directement en `Decimal` — aucun `f64` sur le chemin (ADR-0008, #661).
//! Le commentaire de compatibilité précédent, qui décrivait un boundary
//! `DOUBLE PRECISION → Decimal` « à perte IEEE754 acceptable », ne
//! correspondait plus à la DB depuis cette migration.

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::Row;
use uuid::Uuid;

use crate::application::ports::MeetingCompletionCheckerPort;
use crate::domain::entities::MeetingCompletionChecklist;
use crate::infrastructure::database::pool::DbPool;

pub struct PostgresMeetingCompletionChecker {
    pool: DbPool,
}

impl PostgresMeetingCompletionChecker {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MeetingCompletionCheckerPort for PostgresMeetingCompletionChecker {
    async fn build_checklist(
        &self,
        meeting_id: Uuid,
    ) -> Result<MeetingCompletionChecklist, String> {
        // 1 round-trip, 6 sous-queries. Le building_id du meeting est résolu
        // inline pour `total_quotas` (SUM units.quota du building).
        //
        // **Schema notes** (cf. migrations 2024010100005 / 20251115120000 /
        // 20251119000000 / 20260312000000 / 20260323000001) :
        //  - `convocations.status = 'sent'` → convocations_sent
        //  - `resolutions.status = 'Pending'` → open_resolution
        //  - `meetings.present_quotas IS NOT NULL` → attendance_recorded
        //  - `meetings.present_quotas` → attended_quotas (DOUBLE PRECISION)
        //  - `SUM(units.quota) WHERE building_id = meetings.building_id`
        //    → total_quotas (Decimal natif via units.quota NUMERIC)
        //  - `meetings.minutes_document_id IS NOT NULL` → minutes_draft_exists
        //
        // Si le meeting n'existe pas, la sous-query "building_id" renvoie
        // NULL — le `COALESCE` en aval rend les valeurs sûres mais on
        // détecte explicitement le cas via une row meeting check préalable.
        let row = sqlx::query(
            r#"
            WITH m AS (
                SELECT id, building_id, present_quotas, present_owners_count, minutes_document_id
                FROM meetings
                WHERE id = $1
            )
            SELECT
                EXISTS(SELECT 1 FROM m) AS meeting_exists,
                EXISTS(
                    SELECT 1 FROM convocations c
                    JOIN m ON m.id = c.meeting_id
                    WHERE c.status = 'sent'
                ) AS convocations_sent,
                (
                    SELECT COUNT(*)
                    FROM resolutions r
                    JOIN m ON m.id = r.meeting_id
                    WHERE r.status = 'Pending'
                )::int AS open_resolutions,
                (SELECT present_quotas IS NOT NULL FROM m) AS attendance_recorded,
                COALESCE((SELECT present_quotas FROM m), 0) AS attended_quotas,
                COALESCE(
                    (SELECT SUM(u.quota)
                     FROM units u
                     JOIN m ON m.building_id = u.building_id),
                    0
                ) AS total_quotas,
                -- Story H9 — volet têtes du quorum double (Art. 3.87 §5).
                -- présents : saisi par le syndic (meetings.present_owners_count).
                COALESCE((SELECT present_owners_count FROM m), 0)::int AS present_owners_count,
                -- total : COUNT DISTINCT copropriétaires actifs du building.
                COALESCE((
                    SELECT COUNT(DISTINCT uo.owner_id)
                    FROM unit_owners uo
                    JOIN units u ON uo.unit_id = u.id
                    JOIN m ON m.building_id = u.building_id
                    WHERE uo.end_date IS NULL
                ), 0)::int AS total_owners_count,
                (SELECT minutes_document_id IS NOT NULL FROM m) AS minutes_draft_exists
            "#,
        )
        .bind(meeting_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("DB error building completion checklist: {}", e))?;

        let meeting_exists: bool = row.try_get("meeting_exists").unwrap_or(false);
        if !meeting_exists {
            return Err(format!("Meeting {} not found", meeting_id));
        }

        // #661 — `meetings.present_quotas` est NUMERIC(10,4) depuis la migration
        // `20260516000000_alter_governance_to_numeric.sql` (WP-A1). Le commentaire
        // précédent affirmait encore DOUBLE PRECISION et faisait transiter la
        // quote-part par un `f64` : un aller-retour NUMERIC→f64→Decimal sans
        // objet, sur le chemin de clôture d'AG (Art. 3.87 §5 CC).
        let attended_quotas: Decimal = row.try_get("attended_quotas").unwrap_or(Decimal::ZERO);

        let total_quotas: Decimal = row.try_get("total_quotas").unwrap_or(Decimal::ZERO);

        Ok(MeetingCompletionChecklist {
            convocations_sent: row.try_get("convocations_sent").unwrap_or(false),
            open_resolutions: row.try_get("open_resolutions").unwrap_or(0),
            attendance_recorded: row.try_get("attendance_recorded").unwrap_or(false),
            attended_quotas,
            total_quotas,
            // Story H9 — volet têtes du quorum double (Art. 3.87 §5).
            present_owners_count: row.try_get("present_owners_count").unwrap_or(0),
            total_owners_count: row.try_get("total_owners_count").unwrap_or(0),
            minutes_draft_exists: row.try_get("minutes_draft_exists").unwrap_or(false),
        })
    }
}
