use crate::application::ports::age_request_repository::AgeRequestRepository;
use crate::domain::entities::age_request::{AgeRequest, AgeRequestCosignatory, AgeRequestStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresAgeRequestRepository {
    pool: DbPool,
}

impl PostgresAgeRequestRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn row_to_age_request(row: &sqlx::postgres::PgRow) -> AgeRequest {
    let status_str: String = row.get("status");
    let status = AgeRequestStatus::from_db_string(&status_str).unwrap_or(AgeRequestStatus::Draft);

    AgeRequest {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        building_id: row.get("building_id"),
        title: row.get("title"),
        description: row.get("description"),
        status,
        created_by: row.get("created_by"),
        cosignatories: Vec::new(), // Chargé séparément
        total_shares_pct: row.get::<f64, _>("total_shares_pct"),
        threshold_pct: row.get::<f64, _>("threshold_pct"),
        threshold_reached: row.get("threshold_reached"),
        threshold_reached_at: row.get("threshold_reached_at"),
        submitted_to_syndic_at: row.get("submitted_to_syndic_at"),
        syndic_deadline_at: row.get("syndic_deadline_at"),
        syndic_response_at: row.get("syndic_response_at"),
        syndic_notes: row.get("syndic_notes"),
        auto_convocation_triggered: row.get("auto_convocation_triggered"),
        meeting_id: row.get("meeting_id"),
        concertation_poll_id: row.get("concertation_poll_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cosignatory(row: &sqlx::postgres::PgRow) -> AgeRequestCosignatory {
    AgeRequestCosignatory {
        id: row.get("id"),
        age_request_id: row.get("age_request_id"),
        owner_id: row.get("owner_id"),
        shares_pct: row.get::<f64, _>("shares_pct"),
        signed_at: row.get("signed_at"),
    }
}

#[async_trait]
impl AgeRequestRepository for PostgresAgeRequestRepository {
    async fn create(&self, req: &AgeRequest) -> Result<AgeRequest, String> {
        sqlx::query(
            r#"
            INSERT INTO age_requests (
                id, organization_id, building_id, title, description,
                status, created_by,
                total_shares_pct, threshold_pct, threshold_reached, threshold_reached_at,
                submitted_to_syndic_at, syndic_deadline_at, syndic_response_at, syndic_notes,
                auto_convocation_triggered, meeting_id, concertation_poll_id,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6::age_request_status, $7,
                $8, $9, $10, $11,
                $12, $13, $14, $15,
                $16, $17, $18,
                $19, $20
            )
            "#,
        )
        .bind(req.id)
        .bind(req.organization_id)
        .bind(req.building_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.status.to_db_str())
        .bind(req.created_by)
        .bind(req.total_shares_pct)
        .bind(req.threshold_pct)
        .bind(req.threshold_reached)
        .bind(req.threshold_reached_at)
        .bind(req.submitted_to_syndic_at)
        .bind(req.syndic_deadline_at)
        .bind(req.syndic_response_at)
        .bind(&req.syndic_notes)
        .bind(req.auto_convocation_triggered)
        .bind(req.meeting_id)
        .bind(req.concertation_poll_id)
        .bind(req.created_at)
        .bind(req.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating age_request: {}", e))?;

        Ok(req.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AgeRequest>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   status::TEXT, created_by,
                   total_shares_pct::FLOAT8, threshold_pct::FLOAT8,
                   threshold_reached, threshold_reached_at,
                   submitted_to_syndic_at, syndic_deadline_at, syndic_response_at, syndic_notes,
                   auto_convocation_triggered, meeting_id, concertation_poll_id,
                   created_at, updated_at
            FROM age_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding age_request: {}", e))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let mut req = row_to_age_request(&row);
        req.cosignatories = self.find_cosignatories(id).await?;
        Ok(Some(req))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<AgeRequest>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   status::TEXT, created_by,
                   total_shares_pct::FLOAT8, threshold_pct::FLOAT8,
                   threshold_reached, threshold_reached_at,
                   submitted_to_syndic_at, syndic_deadline_at, syndic_response_at, syndic_notes,
                   auto_convocation_triggered, meeting_id, concertation_poll_id,
                   created_at, updated_at
            FROM age_requests
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error listing age_requests by building: {}", e))?;

        let mut requests = Vec::new();
        for row in &rows {
            let mut req = row_to_age_request(row);
            req.cosignatories = self.find_cosignatories(req.id).await?;
            requests.push(req);
        }
        Ok(requests)
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<AgeRequest>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   status::TEXT, created_by,
                   total_shares_pct::FLOAT8, threshold_pct::FLOAT8,
                   threshold_reached, threshold_reached_at,
                   submitted_to_syndic_at, syndic_deadline_at, syndic_response_at, syndic_notes,
                   auto_convocation_triggered, meeting_id, concertation_poll_id,
                   created_at, updated_at
            FROM age_requests
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error listing age_requests by org: {}", e))?;

        let mut requests = Vec::new();
        for row in &rows {
            let mut req = row_to_age_request(row);
            req.cosignatories = self.find_cosignatories(req.id).await?;
            requests.push(req);
        }
        Ok(requests)
    }

    async fn update(&self, req: &AgeRequest) -> Result<AgeRequest, String> {
        sqlx::query(
            r#"
            UPDATE age_requests SET
                title = $2,
                description = $3,
                status = $4::age_request_status,
                total_shares_pct = $5,
                threshold_pct = $6,
                threshold_reached = $7,
                threshold_reached_at = $8,
                submitted_to_syndic_at = $9,
                syndic_deadline_at = $10,
                syndic_response_at = $11,
                syndic_notes = $12,
                auto_convocation_triggered = $13,
                meeting_id = $14,
                concertation_poll_id = $15,
                updated_at = $16
            WHERE id = $1
            "#,
        )
        .bind(req.id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.status.to_db_str())
        .bind(req.total_shares_pct)
        .bind(req.threshold_pct)
        .bind(req.threshold_reached)
        .bind(req.threshold_reached_at)
        .bind(req.submitted_to_syndic_at)
        .bind(req.syndic_deadline_at)
        .bind(req.syndic_response_at)
        .bind(&req.syndic_notes)
        .bind(req.auto_convocation_triggered)
        .bind(req.meeting_id)
        .bind(req.concertation_poll_id)
        .bind(req.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating age_request: {}", e))?;

        Ok(req.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM age_requests WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting age_request: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn add_cosignatory(&self, cosignatory: &AgeRequestCosignatory) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO age_request_cosignatories (id, age_request_id, owner_id, shares_pct, signed_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (age_request_id, owner_id) DO NOTHING
            "#,
        )
        .bind(cosignatory.id)
        .bind(cosignatory.age_request_id)
        .bind(cosignatory.owner_id)
        .bind(cosignatory.shares_pct)
        .bind(cosignatory.signed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error adding cosignatory: {}", e))?;

        Ok(())
    }

    async fn remove_cosignatory(
        &self,
        age_request_id: Uuid,
        owner_id: Uuid,
    ) -> Result<bool, String> {
        let result = sqlx::query(
            "DELETE FROM age_request_cosignatories WHERE age_request_id = $1 AND owner_id = $2",
        )
        .bind(age_request_id)
        .bind(owner_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error removing cosignatory: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_cosignatories(
        &self,
        age_request_id: Uuid,
    ) -> Result<Vec<AgeRequestCosignatory>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, age_request_id, owner_id, shares_pct::FLOAT8, signed_at
            FROM age_request_cosignatories
            WHERE age_request_id = $1
            ORDER BY signed_at ASC
            "#,
        )
        .bind(age_request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error loading cosignatories: {}", e))?;

        Ok(rows.iter().map(row_to_cosignatory).collect())
    }

    async fn find_expired_deadlines(&self) -> Result<Vec<AgeRequest>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   status::TEXT, created_by,
                   total_shares_pct::FLOAT8, threshold_pct::FLOAT8,
                   threshold_reached, threshold_reached_at,
                   submitted_to_syndic_at, syndic_deadline_at, syndic_response_at, syndic_notes,
                   auto_convocation_triggered, meeting_id, concertation_poll_id,
                   created_at, updated_at
            FROM age_requests
            WHERE status = 'submitted'
              AND syndic_deadline_at IS NOT NULL
              AND syndic_deadline_at <= NOW()
            ORDER BY syndic_deadline_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding expired deadlines: {}", e))?;

        let mut requests = Vec::new();
        for row in &rows {
            let mut req = row_to_age_request(row);
            req.cosignatories = self.find_cosignatories(req.id).await?;
            requests.push(req);
        }
        Ok(requests)
    }
}
