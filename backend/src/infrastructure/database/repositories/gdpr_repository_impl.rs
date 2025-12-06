use crate::application::ports::GdprRepository;
use crate::domain::entities::gdpr_export::{
    DocumentData, ExpenseData, GdprExport, MeetingData, OwnerData, UnitOwnershipData, UserData,
};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL implementation of GdprRepository
pub struct PostgresGdprRepository {
    pool: Arc<PgPool>,
}

impl PostgresGdprRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GdprRepository for PostgresGdprRepository {
    async fn aggregate_user_data(
        &self,
        user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<GdprExport, String> {
        // Fetch user data
        let user_data = self.fetch_user_data(user_id).await?;

        // Fetch owner profiles linked to this user
        let owner_profiles = self.fetch_owner_profiles(user_id, organization_id).await?;

        // Fetch unit ownership data
        let units = self.fetch_unit_ownership(user_id, organization_id).await?;

        // Fetch expenses
        let expenses = self.fetch_expenses(user_id, organization_id).await?;

        // Fetch documents
        let documents = self.fetch_documents(user_id, organization_id).await?;

        // Fetch meetings
        let meetings = self.fetch_meetings(user_id, organization_id).await?;

        // Build GdprExport aggregate
        let mut export = GdprExport::new(user_data);

        for owner in owner_profiles {
            export.add_owner_profile(owner);
        }

        for unit in units {
            export.add_unit_ownership(unit);
        }

        for expense in expenses {
            export.add_expense(expense);
        }

        for document in documents {
            export.add_document(document);
        }

        for meeting in meetings {
            export.add_meeting(meeting);
        }

        Ok(export)
    }

    async fn anonymize_user(&self, user_id: Uuid) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET
                email = CONCAT('anonymized-', id::text, '@deleted.local'),
                first_name = 'Anonymized',
                last_name = 'User',
                is_anonymized = TRUE,
                anonymized_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND is_anonymized = FALSE
            "#,
            user_id
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to anonymize user: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("User not found or already anonymized".to_string());
        }

        Ok(())
    }

    async fn anonymize_owner(&self, owner_id: Uuid) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE owners
            SET
                email = NULL,
                phone = NULL,
                address = NULL,
                city = NULL,
                postal_code = NULL,
                country = NULL,
                first_name = 'Anonymized',
                last_name = 'User',
                is_anonymized = TRUE,
                anonymized_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND is_anonymized = FALSE
            "#,
            owner_id
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to anonymize owner: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Owner not found or already anonymized".to_string());
        }

        Ok(())
    }

    async fn find_owner_ids_by_user(
        &self,
        user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<Uuid>, String> {
        // Fetch owners by user_id foreign key
        let query = if let Some(org_id) = organization_id {
            sqlx::query!(
                r#"
                SELECT id
                FROM owners
                WHERE user_id = $1 AND organization_id = $2 AND (is_anonymized IS NULL OR is_anonymized = FALSE)
                "#,
                user_id,
                org_id
            )
            .fetch_all(self.pool.as_ref())
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT id
                FROM owners
                WHERE user_id = $1 AND (is_anonymized IS NULL OR is_anonymized = FALSE)
                "#,
                user_id
            )
            .fetch_all(self.pool.as_ref())
            .await
        };

        let records = query.map_err(|e| format!("Failed to fetch owner IDs: {}", e))?;

        Ok(records.into_iter().map(|r| r.id).collect())
    }

    async fn check_legal_holds(&self, user_id: Uuid) -> Result<Vec<String>, String> {
        let mut holds = Vec::new();

        // Check for unpaid expenses linked to user's owner profiles
        let unpaid_expenses = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM expenses e
            INNER JOIN units u ON e.building_id = u.building_id
            INNER JOIN unit_owners uo ON u.id = uo.unit_id
            INNER JOIN owners o ON uo.owner_id = o.id
            WHERE o.user_id = $1 AND e.payment_status != 'paid'
            "#,
            user_id
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to check unpaid expenses: {}", e))?;

        if unpaid_expenses.count.unwrap_or(0) > 0 {
            holds.push(format!(
                "Unpaid expenses: {}",
                unpaid_expenses.count.unwrap_or(0)
            ));
        }

        // Future: Add more legal hold checks here
        // - Ongoing legal proceedings
        // - Pending transactions
        // - Regulatory requirements

        Ok(holds)
    }

    async fn is_user_anonymized(&self, user_id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            SELECT is_anonymized
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to check user anonymization status: {}", e))?;

        match result {
            Some(record) => Ok(record.is_anonymized),
            None => Err("User not found".to_string()),
        }
    }
}

impl PostgresGdprRepository {
    /// Fetch user data from database
    async fn fetch_user_data(&self, user_id: Uuid) -> Result<UserData, String> {
        let record = sqlx::query!(
            r#"
            SELECT id, email, first_name, last_name, organization_id,
                   is_active, is_anonymized, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to fetch user data: {}", e))?;

        let record = record.ok_or("User not found".to_string())?;

        Ok(UserData {
            id: record.id,
            email: record.email,
            first_name: record.first_name,
            last_name: record.last_name,
            organization_id: record.organization_id,
            is_active: record.is_active,
            is_anonymized: record.is_anonymized,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    /// Fetch owner profiles linked to user
    async fn fetch_owner_profiles(
        &self,
        user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<OwnerData>, String> {
        let query = if let Some(org_id) = organization_id {
            sqlx::query!(
                r#"
                SELECT id, organization_id, first_name, last_name,
                       email, phone, address, city, postal_code, country,
                       is_anonymized, created_at, updated_at
                FROM owners
                WHERE user_id = $1 AND organization_id = $2
                "#,
                user_id,
                org_id
            )
            .fetch_all(self.pool.as_ref())
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT id, organization_id, first_name, last_name,
                       email, phone, address, city, postal_code, country,
                       is_anonymized, created_at, updated_at
                FROM owners
                WHERE user_id = $1
                "#,
                user_id
            )
            .fetch_all(self.pool.as_ref())
            .await
        };

        let records = query.map_err(|e| format!("Failed to fetch owner profiles: {}", e))?;

        Ok(records
            .into_iter()
            .map(|r| OwnerData {
                id: r.id,
                organization_id: r.organization_id,
                first_name: r.first_name,
                last_name: r.last_name,
                email: r.email,
                phone: r.phone,
                address: r.address,
                city: r.city,
                postal_code: r.postal_code,
                country: r.country,
                is_anonymized: r.is_anonymized.unwrap_or(false),
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    /// Fetch unit ownership data
    async fn fetch_unit_ownership(
        &self,
        user_id: Uuid,
        _organization_id: Option<Uuid>,
    ) -> Result<Vec<UnitOwnershipData>, String> {
        let records = sqlx::query!(
            r#"
            SELECT b.name as building_name, b.address as building_address,
                   u.unit_number, u.floor, uo.ownership_percentage,
                   uo.start_date, uo.end_date, uo.is_primary_contact
            FROM unit_owners uo
            INNER JOIN units u ON uo.unit_id = u.id
            INNER JOIN buildings b ON u.building_id = b.id
            INNER JOIN owners o ON uo.owner_id = o.id
            WHERE o.user_id = $1
            ORDER BY b.name, u.unit_number
            "#,
            user_id
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to fetch unit ownership: {}", e))?;

        Ok(records
            .into_iter()
            .map(|r| UnitOwnershipData {
                building_name: r.building_name,
                building_address: r.building_address,
                unit_number: r.unit_number,
                floor: r.floor,
                ownership_percentage: r.ownership_percentage,
                start_date: r.start_date,
                end_date: r.end_date,
                is_primary_contact: r.is_primary_contact,
            })
            .collect())
    }

    /// Fetch expenses related to user
    async fn fetch_expenses(
        &self,
        user_id: Uuid,
        _organization_id: Option<Uuid>,
    ) -> Result<Vec<ExpenseData>, String> {
        let records = sqlx::query!(
            r#"
            SELECT DISTINCT e.description, e.amount, e.expense_date as due_date,
                   (e.payment_status = 'paid') as paid, b.name as building_name
            FROM expenses e
            INNER JOIN buildings b ON e.building_id = b.id
            INNER JOIN units u ON b.id = u.building_id
            INNER JOIN unit_owners uo ON u.id = uo.unit_id
            INNER JOIN owners o ON uo.owner_id = o.id
            WHERE o.user_id = $1
            ORDER BY e.expense_date DESC
            "#,
            user_id
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to fetch expenses: {}", e))?;

        Ok(records
            .into_iter()
            .map(|r| ExpenseData {
                description: r.description,
                amount: r.amount,
                due_date: r.due_date,
                paid: r.paid.unwrap_or(false),
                building_name: r.building_name,
            })
            .collect())
    }

    /// Fetch documents related to user
    async fn fetch_documents(
        &self,
        user_id: Uuid,
        _organization_id: Option<Uuid>,
    ) -> Result<Vec<DocumentData>, String> {
        let records = sqlx::query!(
            r#"
            SELECT DISTINCT d.title, d.document_type::text as document_type, d.created_at as uploaded_at, b.name as building_name
            FROM documents d
            LEFT JOIN buildings b ON d.building_id = b.id
            WHERE d.uploaded_by = $1
            ORDER BY d.created_at DESC
            "#,
            user_id
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to fetch documents: {}", e))?;

        Ok(records
            .into_iter()
            .map(|r| DocumentData {
                title: r.title,
                document_type: r.document_type.unwrap_or_else(|| "unknown".to_string()),
                uploaded_at: r.uploaded_at,
                building_name: Some(r.building_name),
            })
            .collect())
    }

    /// Fetch meetings attended by user
    async fn fetch_meetings(
        &self,
        _user_id: Uuid,
        _organization_id: Option<Uuid>,
    ) -> Result<Vec<MeetingData>, String> {
        let records = sqlx::query!(
            r#"
            SELECT DISTINCT m.title, m.scheduled_date as meeting_date, m.agenda::text as agenda, b.name as building_name
            FROM meetings m
            INNER JOIN buildings b ON m.building_id = b.id
            ORDER BY m.scheduled_date DESC
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| format!("Failed to fetch meetings: {}", e))?;

        Ok(records
            .into_iter()
            .map(|r| MeetingData {
                title: r.title,
                meeting_date: r.meeting_date,
                agenda: r.agenda,
                building_name: r.building_name,
            })
            .collect())
    }
}
