use crate::application::ports::security_incident_repository::{
    SecurityIncidentFilters, SecurityIncidentRepository,
};
use crate::domain::entities::SecurityIncident;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresSecurityIncidentRepository {
    pool: DbPool,
}

impl PostgresSecurityIncidentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_incident(row: &sqlx::postgres::PgRow) -> SecurityIncident {
        let data_categories: Option<Vec<String>> = row
            .try_get("data_categories_affected")
            .ok()
            .unwrap_or(None);

        SecurityIncident {
            id: row.get("id"),
            organization_id: row.try_get("organization_id").ok().flatten(),
            severity: row.get("severity"),
            incident_type: row.get("incident_type"),
            title: row.get("title"),
            description: row.get("description"),
            data_categories_affected: data_categories.unwrap_or_default(),
            affected_subjects_count: row.try_get("affected_subjects_count").ok().flatten(),
            discovery_at: row.get("discovery_at"),
            notification_at: row.try_get("notification_at").ok().flatten(),
            apd_reference_number: row.try_get("apd_reference_number").ok().flatten(),
            status: row.get("status"),
            reported_by: row.get("reported_by"),
            investigation_notes: row.try_get("investigation_notes").ok().flatten(),
            root_cause: row.try_get("root_cause").ok().flatten(),
            remediation_steps: row.try_get("remediation_steps").ok().flatten(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl SecurityIncidentRepository for PostgresSecurityIncidentRepository {
    async fn create(&self, incident: &SecurityIncident) -> Result<SecurityIncident, String> {
        let row = sqlx::query(
            r#"
            INSERT INTO security_incidents (
                id, organization_id, severity, incident_type, title, description,
                data_categories_affected, affected_subjects_count,
                discovery_at, status, reported_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING
                id, organization_id, severity, incident_type, title, description,
                data_categories_affected, affected_subjects_count, discovery_at,
                notification_at, apd_reference_number, status, reported_by,
                investigation_notes, root_cause, remediation_steps, created_at, updated_at
            "#,
        )
        .bind(incident.id)
        .bind(incident.organization_id)
        .bind(&incident.severity)
        .bind(&incident.incident_type)
        .bind(&incident.title)
        .bind(&incident.description)
        .bind(&incident.data_categories_affected)
        .bind(incident.affected_subjects_count)
        .bind(incident.discovery_at)
        .bind(&incident.status)
        .bind(incident.reported_by)
        .bind(incident.created_at)
        .bind(incident.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create security incident: {}", e))?;

        Ok(Self::row_to_incident(&row))
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<Option<SecurityIncident>, String> {
        let row = if let Some(org_id) = organization_id {
            sqlx::query(
                r#"
                SELECT id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                FROM security_incidents
                WHERE id = $1 AND organization_id = $2
                "#,
            )
            .bind(id)
            .bind(org_id)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                FROM security_incidents
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
        }
        .map_err(|e| format!("Failed to fetch security incident: {}", e))?;

        Ok(row.map(|r| Self::row_to_incident(&r)))
    }

    async fn find_all(
        &self,
        organization_id: Option<Uuid>,
        filters: SecurityIncidentFilters,
    ) -> Result<(Vec<SecurityIncident>, i64), String> {
        let offset = (filters.page - 1) * filters.per_page;

        let rows = match (organization_id, &filters.severity, &filters.status) {
            (Some(org_id), Some(sev), Some(st)) => {
                sqlx::query(
                    r#"
                    SELECT id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    WHERE organization_id = $1 AND severity = $2 AND status = $3
                    ORDER BY discovery_at DESC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(org_id)
                .bind(sev)
                .bind(st)
                .bind(filters.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(org_id), Some(sev), None) => {
                sqlx::query(
                    r#"
                    SELECT id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    WHERE organization_id = $1 AND severity = $2
                    ORDER BY discovery_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(org_id)
                .bind(sev)
                .bind(filters.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(org_id), None, Some(st)) => {
                sqlx::query(
                    r#"
                    SELECT id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    WHERE organization_id = $1 AND status = $2
                    ORDER BY discovery_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(org_id)
                .bind(st)
                .bind(filters.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(org_id), None, None) => {
                sqlx::query(
                    r#"
                    SELECT id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    WHERE organization_id = $1
                    ORDER BY discovery_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(org_id)
                .bind(filters.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            _ => {
                sqlx::query(
                    r#"
                    SELECT id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    ORDER BY discovery_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(filters.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| format!("Failed to list security incidents: {}", e))?;

        let total: i64 = if let Some(org_id) = organization_id {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM security_incidents WHERE organization_id = $1",
            )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to count incidents: {}", e))?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM security_incidents")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Failed to count incidents: {}", e))?
        };

        let incidents = rows.iter().map(Self::row_to_incident).collect();
        Ok((incidents, total))
    }

    async fn report_to_apd(
        &self,
        id: Uuid,
        organization_id: Option<Uuid>,
        apd_reference_number: String,
        investigation_notes: Option<String>,
    ) -> Result<Option<SecurityIncident>, String> {
        let row = if let Some(org_id) = organization_id {
            sqlx::query(
                r#"
                UPDATE security_incidents
                SET notification_at = now(),
                    apd_reference_number = $1,
                    status = 'reported',
                    investigation_notes = $2,
                    updated_at = now()
                WHERE id = $3 AND organization_id = $4
                RETURNING
                    id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                "#,
            )
            .bind(&apd_reference_number)
            .bind(&investigation_notes)
            .bind(id)
            .bind(org_id)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                UPDATE security_incidents
                SET notification_at = now(),
                    apd_reference_number = $1,
                    status = 'reported',
                    investigation_notes = $2,
                    updated_at = now()
                WHERE id = $3
                RETURNING
                    id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                "#,
            )
            .bind(&apd_reference_number)
            .bind(&investigation_notes)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
        }
        .map_err(|e| format!("Failed to report incident to APD: {}", e))?;

        Ok(row.map(|r| Self::row_to_incident(&r)))
    }

    async fn find_overdue(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<SecurityIncident>, String> {
        let rows = if let Some(org_id) = organization_id {
            sqlx::query(
                r#"
                SELECT id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                FROM security_incidents
                WHERE organization_id = $1
                  AND notification_at IS NULL
                  AND status IN ('detected', 'investigating', 'contained')
                  AND discovery_at < (NOW() - INTERVAL '72 hours')
                ORDER BY discovery_at ASC
                "#,
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                FROM security_incidents
                WHERE notification_at IS NULL
                  AND status IN ('detected', 'investigating', 'contained')
                  AND discovery_at < (NOW() - INTERVAL '72 hours')
                ORDER BY discovery_at ASC
                "#,
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| format!("Failed to fetch overdue incidents: {}", e))?;

        Ok(rows.iter().map(Self::row_to_incident).collect())
    }
}
