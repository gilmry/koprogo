use crate::application::dto::etat_date_dto::EtatDateStatsResponse;
use crate::application::dto::PageRequest;
use crate::application::ports::EtatDateRepository;
use crate::domain::entities::{EtatDate, EtatDateLanguage, EtatDateStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;

use sqlx::Row;
use uuid::Uuid;

pub struct PostgresEtatDateRepository {
    pool: DbPool,
}

impl PostgresEtatDateRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EtatDateRepository for PostgresEtatDateRepository {
    async fn create(&self, etat_date: &EtatDate) -> Result<EtatDate, String> {
        let status_str = match etat_date.status {
            EtatDateStatus::Requested => "requested",
            EtatDateStatus::InProgress => "in_progress",
            EtatDateStatus::Generated => "generated",
            EtatDateStatus::Delivered => "delivered",
            EtatDateStatus::Expired => "expired",
        };

        let language_str = match etat_date.language {
            EtatDateLanguage::Fr => "fr",
            EtatDateLanguage::Nl => "nl",
            EtatDateLanguage::De => "de",
        };

        sqlx::query(
            r#"
            INSERT INTO etats_dates (
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status, language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area,
                ordinary_charges_quota, extraordinary_charges_quota,
                owner_balance, arrears_amount, monthly_provision_amount,
                total_balance, approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                CAST($9 AS etat_date_status), CAST($10 AS etat_date_language), $11,
                $12, $13, $14,
                $15, $16, $17, $18, $19,
                $20, $21,
                $22, $23, $24,
                $25, $26,
                $27, $28,
                $29, $30
            )
            "#,
        )
        .bind(etat_date.id)
        .bind(etat_date.organization_id)
        .bind(etat_date.building_id)
        .bind(etat_date.unit_id)
        .bind(etat_date.reference_date)
        .bind(etat_date.requested_date)
        .bind(etat_date.generated_date)
        .bind(etat_date.delivered_date)
        .bind(status_str)
        .bind(language_str)
        .bind(&etat_date.reference_number)
        .bind(&etat_date.notary_name)
        .bind(&etat_date.notary_email)
        .bind(&etat_date.notary_phone)
        .bind(&etat_date.building_name)
        .bind(&etat_date.building_address)
        .bind(&etat_date.unit_number)
        .bind(&etat_date.unit_floor)
        .bind(etat_date.unit_area)
        .bind(etat_date.ordinary_charges_quota)
        .bind(etat_date.extraordinary_charges_quota)
        .bind(etat_date.owner_balance)
        .bind(etat_date.arrears_amount)
        .bind(etat_date.monthly_provision_amount)
        .bind(etat_date.total_balance)
        .bind(etat_date.approved_works_unpaid)
        .bind(&etat_date.additional_data)
        .bind(&etat_date.pdf_file_path)
        .bind(etat_date.created_at)
        .bind(etat_date.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(etat_date.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<EtatDate>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| self.row_to_etat_date(row)))
    }

    async fn find_by_reference_number(
        &self,
        reference_number: &str,
    ) -> Result<Option<EtatDate>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE reference_number = $1
            "#,
        )
        .bind(reference_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| self.row_to_etat_date(row)))
    }

    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<EtatDate>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE unit_id = $1
            ORDER BY requested_date DESC
            "#,
        )
        .bind(unit_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_etat_date(row))
            .collect())
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<EtatDate>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE building_id = $1
            ORDER BY requested_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_etat_date(row))
            .collect())
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        status: Option<EtatDateStatus>,
    ) -> Result<(Vec<EtatDate>, i64), String> {
        let offset = (page_request.page - 1) * page_request.per_page;

        let mut query_str = String::from(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE 1=1
            "#,
        );

        let mut count_query_str = String::from("SELECT COUNT(*) FROM etats_dates WHERE 1=1");

        if organization_id.is_some() {
            query_str.push_str(" AND organization_id = $1");
            count_query_str.push_str(" AND organization_id = $1");
        }

        if status.is_some() {
            let param_num = if organization_id.is_some() {
                "$2"
            } else {
                "$1"
            };
            query_str.push_str(&format!(
                " AND status = CAST({} AS etat_date_status)",
                param_num
            ));
            count_query_str.push_str(&format!(
                " AND status = CAST({} AS etat_date_status)",
                param_num
            ));
        }

        query_str.push_str(" ORDER BY requested_date DESC");

        let param_num_offset = match (organization_id.is_some(), status.is_some()) {
            (true, true) => "$3",
            (true, false) | (false, true) => "$2",
            (false, false) => "$1",
        };
        let param_num_limit = match (organization_id.is_some(), status.is_some()) {
            (true, true) => "$4",
            (true, false) | (false, true) => "$3",
            (false, false) => "$2",
        };

        query_str.push_str(&format!(
            " OFFSET {} LIMIT {}",
            param_num_offset, param_num_limit
        ));

        // Build query with dynamic bindings
        let mut query = sqlx::query(&query_str);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);

        if let Some(org_id) = organization_id {
            query = query.bind(org_id);
            count_query = count_query.bind(org_id);
        }

        if let Some(st) = &status {
            let status_str = match st {
                EtatDateStatus::Requested => "requested",
                EtatDateStatus::InProgress => "in_progress",
                EtatDateStatus::Generated => "generated",
                EtatDateStatus::Delivered => "delivered",
                EtatDateStatus::Expired => "expired",
            };
            query = query.bind(status_str);
            count_query = count_query.bind(status_str);
        }

        query = query.bind(offset as i64).bind(page_request.per_page as i64);

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let etats = rows
            .into_iter()
            .map(|row| self.row_to_etat_date(row))
            .collect();

        Ok((etats, total))
    }

    async fn find_overdue(&self, organization_id: Uuid) -> Result<Vec<EtatDate>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE organization_id = $1
              AND status IN ('requested', 'in_progress')
              AND requested_date < NOW() - INTERVAL '10 days'
            ORDER BY requested_date ASC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_etat_date(row))
            .collect())
    }

    async fn find_expired(&self, organization_id: Uuid) -> Result<Vec<EtatDate>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, unit_id,
                reference_date, requested_date, generated_date, delivered_date,
                status::text AS status, language::text AS language, reference_number,
                notary_name, notary_email, notary_phone,
                building_name, building_address, unit_number, unit_floor, unit_area::FLOAT8 AS unit_area,
                ordinary_charges_quota::FLOAT8 AS ordinary_charges_quota, extraordinary_charges_quota::FLOAT8 AS extraordinary_charges_quota,
                owner_balance::FLOAT8 AS owner_balance, arrears_amount::FLOAT8 AS arrears_amount, monthly_provision_amount::FLOAT8 AS monthly_provision_amount,
                total_balance::FLOAT8 AS total_balance, approved_works_unpaid::FLOAT8 AS approved_works_unpaid,
                additional_data, pdf_file_path,
                created_at, updated_at
            FROM etats_dates
            WHERE organization_id = $1
              AND reference_date < NOW() - INTERVAL '90 days'
            ORDER BY reference_date ASC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_etat_date(row))
            .collect())
    }

    async fn update(&self, etat_date: &EtatDate) -> Result<EtatDate, String> {
        let status_str = match etat_date.status {
            EtatDateStatus::Requested => "requested",
            EtatDateStatus::InProgress => "in_progress",
            EtatDateStatus::Generated => "generated",
            EtatDateStatus::Delivered => "delivered",
            EtatDateStatus::Expired => "expired",
        };

        sqlx::query(
            r#"
            UPDATE etats_dates
            SET generated_date = $1,
                delivered_date = $2,
                status = CAST($3 AS etat_date_status),
                owner_balance = $4,
                arrears_amount = $5,
                monthly_provision_amount = $6,
                total_balance = $7,
                approved_works_unpaid = $8,
                additional_data = $9,
                pdf_file_path = $10,
                updated_at = $11
            WHERE id = $12
            "#,
        )
        .bind(etat_date.generated_date)
        .bind(etat_date.delivered_date)
        .bind(status_str)
        .bind(etat_date.owner_balance)
        .bind(etat_date.arrears_amount)
        .bind(etat_date.monthly_provision_amount)
        .bind(etat_date.total_balance)
        .bind(etat_date.approved_works_unpaid)
        .bind(&etat_date.additional_data)
        .bind(&etat_date.pdf_file_path)
        .bind(etat_date.updated_at)
        .bind(etat_date.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(etat_date.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM etats_dates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_stats(&self, organization_id: Uuid) -> Result<EtatDateStatsResponse, String> {
        let stats = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_requests,
                COUNT(*) FILTER (WHERE status = 'requested') as requested_count,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress_count,
                COUNT(*) FILTER (WHERE status = 'generated') as generated_count,
                COUNT(*) FILTER (WHERE status = 'delivered') as delivered_count,
                COUNT(*) FILTER (WHERE reference_date < NOW() - INTERVAL '90 days') as expired_count,
                COUNT(*) FILTER (WHERE status IN ('requested', 'in_progress') AND requested_date < NOW() - INTERVAL '10 days') as overdue_count,
                COALESCE(AVG(EXTRACT(EPOCH FROM (COALESCE(generated_date, NOW()) - requested_date)) / 86400), 0)::FLOAT8 as avg_processing_days
            FROM etats_dates
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(EtatDateStatsResponse {
            total_requests: stats.get("total_requests"),
            requested_count: stats.get("requested_count"),
            in_progress_count: stats.get("in_progress_count"),
            generated_count: stats.get("generated_count"),
            delivered_count: stats.get("delivered_count"),
            expired_count: stats.get("expired_count"),
            overdue_count: stats.get("overdue_count"),
            average_processing_days: stats.get("avg_processing_days"),
        })
    }

    async fn count_by_status(
        &self,
        organization_id: Uuid,
        status: EtatDateStatus,
    ) -> Result<i64, String> {
        let status_str = match status {
            EtatDateStatus::Requested => "requested",
            EtatDateStatus::InProgress => "in_progress",
            EtatDateStatus::Generated => "generated",
            EtatDateStatus::Delivered => "delivered",
            EtatDateStatus::Expired => "expired",
        };

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM etats_dates WHERE organization_id = $1 AND status = CAST($2 AS etat_date_status)",
        )
        .bind(organization_id)
        .bind(status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }
}

impl PostgresEtatDateRepository {
    fn row_to_etat_date(&self, row: sqlx::postgres::PgRow) -> EtatDate {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "in_progress" => EtatDateStatus::InProgress,
            "generated" => EtatDateStatus::Generated,
            "delivered" => EtatDateStatus::Delivered,
            "expired" => EtatDateStatus::Expired,
            _ => EtatDateStatus::Requested,
        };

        let language_str: String = row.get("language");
        let language = match language_str.as_str() {
            "nl" => EtatDateLanguage::Nl,
            "de" => EtatDateLanguage::De,
            _ => EtatDateLanguage::Fr,
        };

        EtatDate {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            building_id: row.get("building_id"),
            unit_id: row.get("unit_id"),
            reference_date: row.get("reference_date"),
            requested_date: row.get("requested_date"),
            generated_date: row.get("generated_date"),
            delivered_date: row.get("delivered_date"),
            status,
            language,
            reference_number: row.get("reference_number"),
            notary_name: row.get("notary_name"),
            notary_email: row.get("notary_email"),
            notary_phone: row.get("notary_phone"),
            building_name: row.get("building_name"),
            building_address: row.get("building_address"),
            unit_number: row.get("unit_number"),
            unit_floor: row.get("unit_floor"),
            unit_area: row.get("unit_area"),
            ordinary_charges_quota: row.get("ordinary_charges_quota"),
            extraordinary_charges_quota: row.get("extraordinary_charges_quota"),
            owner_balance: row.get("owner_balance"),
            arrears_amount: row.get("arrears_amount"),
            monthly_provision_amount: row.get("monthly_provision_amount"),
            total_balance: row.get("total_balance"),
            approved_works_unpaid: row.get("approved_works_unpaid"),
            additional_data: row.get("additional_data"),
            pdf_file_path: row.get("pdf_file_path"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
