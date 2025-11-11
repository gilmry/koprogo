use crate::application::ports::OwnerContributionRepository;
use crate::domain::entities::{
    ContributionPaymentStatus, ContributionType, OwnerContribution, PaymentMethod,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresOwnerContributionRepository {
    pool: PgPool,
}

impl PostgresOwnerContributionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Helper to convert DB row to domain entity
    fn row_to_entity(row: sqlx::postgres::PgRow) -> Result<OwnerContribution, String> {
        use sqlx::Row;

        let contribution_type_str: String = row.get("contribution_type");
        let contribution_type = match contribution_type_str.as_str() {
            "regular" => ContributionType::Regular,
            "extraordinary" => ContributionType::Extraordinary,
            "advance" => ContributionType::Advance,
            "adjustment" => ContributionType::Adjustment,
            _ => {
                return Err(format!(
                    "Unknown contribution type: {}",
                    contribution_type_str
                ))
            }
        };

        let payment_status_str: String = row.get("payment_status");
        let payment_status = match payment_status_str.as_str() {
            "pending" => ContributionPaymentStatus::Pending,
            "paid" => ContributionPaymentStatus::Paid,
            "partial" => ContributionPaymentStatus::Partial,
            "cancelled" => ContributionPaymentStatus::Cancelled,
            _ => return Err(format!("Unknown payment status: {}", payment_status_str)),
        };

        let payment_method: Option<String> = row.get("payment_method");
        let payment_method = payment_method
            .map(|pm| match pm.as_str() {
                "bank_transfer" => Ok(PaymentMethod::BankTransfer),
                "cash" => Ok(PaymentMethod::Cash),
                "check" => Ok(PaymentMethod::Check),
                "domiciliation" => Ok(PaymentMethod::Domiciliation),
                _ => Err(format!("Unknown payment method: {}", pm)),
            })
            .transpose()?;

        let amount: sqlx::types::Decimal = row.get("amount");
        let amount = amount
            .to_string()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse amount: {}", e))?;

        Ok(OwnerContribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            owner_id: row.get("owner_id"),
            unit_id: row.get("unit_id"),
            description: row.get("description"),
            amount,
            account_code: row.get("account_code"),
            contribution_type,
            contribution_date: row.get("contribution_date"),
            payment_date: row.get("payment_date"),
            payment_method,
            payment_reference: row.get("payment_reference"),
            payment_status,
            call_for_funds_id: row.get("call_for_funds_id"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
        })
    }
}

#[async_trait]
impl OwnerContributionRepository for PostgresOwnerContributionRepository {
    async fn create(&self, contribution: &OwnerContribution) -> Result<OwnerContribution, String> {
        let contribution_type = match contribution.contribution_type {
            ContributionType::Regular => "regular",
            ContributionType::Extraordinary => "extraordinary",
            ContributionType::Advance => "advance",
            ContributionType::Adjustment => "adjustment",
        };

        let payment_status = match contribution.payment_status {
            ContributionPaymentStatus::Pending => "pending",
            ContributionPaymentStatus::Paid => "paid",
            ContributionPaymentStatus::Partial => "partial",
            ContributionPaymentStatus::Cancelled => "cancelled",
        };

        let payment_method = contribution.payment_method.as_ref().map(|pm| match pm {
            PaymentMethod::BankTransfer => "bank_transfer",
            PaymentMethod::Cash => "cash",
            PaymentMethod::Check => "check",
            PaymentMethod::Domiciliation => "domiciliation",
        });

        let row = sqlx::query(
            r#"
            INSERT INTO owner_contributions (
                id, organization_id, owner_id, unit_id,
                description, amount, account_code,
                contribution_type, contribution_date, payment_date,
                payment_method, payment_reference, payment_status,
                call_for_funds_id, notes, created_at, updated_at, created_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            RETURNING *
            "#,
        )
        .bind(contribution.id)
        .bind(contribution.organization_id)
        .bind(contribution.owner_id)
        .bind(contribution.unit_id)
        .bind(&contribution.description)
        .bind(contribution.amount)
        .bind(&contribution.account_code)
        .bind(contribution_type)
        .bind(contribution.contribution_date)
        .bind(contribution.payment_date)
        .bind(payment_method)
        .bind(&contribution.payment_reference)
        .bind(payment_status)
        .bind(contribution.call_for_funds_id)
        .bind(&contribution.notes)
        .bind(contribution.created_at)
        .bind(contribution.updated_at)
        .bind(contribution.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create owner contribution: {}", e))?;

        Self::row_to_entity(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OwnerContribution>, String> {
        let row = sqlx::query("SELECT * FROM owner_contributions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find owner contribution by id: {}", e))?;

        match row {
            Some(r) => Ok(Some(Self::row_to_entity(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        let rows = sqlx::query(
            "SELECT * FROM owner_contributions WHERE organization_id = $1 ORDER BY contribution_date DESC",
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find owner contributions by organization: {}", e))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerContribution>, String> {
        let rows = sqlx::query(
            "SELECT * FROM owner_contributions WHERE owner_id = $1 ORDER BY contribution_date DESC",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find owner contributions by owner: {}", e))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    async fn update(&self, contribution: &OwnerContribution) -> Result<OwnerContribution, String> {
        let contribution_type = match contribution.contribution_type {
            ContributionType::Regular => "regular",
            ContributionType::Extraordinary => "extraordinary",
            ContributionType::Advance => "advance",
            ContributionType::Adjustment => "adjustment",
        };

        let payment_status = match contribution.payment_status {
            ContributionPaymentStatus::Pending => "pending",
            ContributionPaymentStatus::Paid => "paid",
            ContributionPaymentStatus::Partial => "partial",
            ContributionPaymentStatus::Cancelled => "cancelled",
        };

        let payment_method = contribution.payment_method.as_ref().map(|pm| match pm {
            PaymentMethod::BankTransfer => "bank_transfer",
            PaymentMethod::Cash => "cash",
            PaymentMethod::Check => "check",
            PaymentMethod::Domiciliation => "domiciliation",
        });

        let row = sqlx::query(
            r#"
            UPDATE owner_contributions SET
                organization_id = $2,
                owner_id = $3,
                unit_id = $4,
                description = $5,
                amount = $6,
                account_code = $7,
                contribution_type = $8,
                contribution_date = $9,
                payment_date = $10,
                payment_method = $11,
                payment_reference = $12,
                payment_status = $13,
                notes = $14,
                updated_at = $15,
                created_by = $16
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(contribution.id)
        .bind(contribution.organization_id)
        .bind(contribution.owner_id)
        .bind(contribution.unit_id)
        .bind(&contribution.description)
        .bind(contribution.amount)
        .bind(&contribution.account_code)
        .bind(contribution_type)
        .bind(contribution.contribution_date)
        .bind(contribution.payment_date)
        .bind(payment_method)
        .bind(&contribution.payment_reference)
        .bind(payment_status)
        .bind(&contribution.notes)
        .bind(contribution.updated_at)
        .bind(contribution.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update owner contribution: {}", e))?;

        Self::row_to_entity(row)
    }
}
