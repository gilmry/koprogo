use crate::application::ports::PaymentMethodRepository;
use crate::domain::entities::payment_method::{PaymentMethod, PaymentMethodType};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of PaymentMethodRepository
pub struct PostgresPaymentMethodRepository {
    pool: PgPool,
}

impl PostgresPaymentMethodRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert PaymentMethodType enum to database string
    fn method_type_to_db(method_type: &PaymentMethodType) -> &'static str {
        match method_type {
            PaymentMethodType::Card => "card",
            PaymentMethodType::SepaDebit => "sepa_debit",
        }
    }

    /// Convert database string to PaymentMethodType enum
    fn method_type_from_db(s: &str) -> Result<PaymentMethodType, String> {
        match s {
            "card" => Ok(PaymentMethodType::Card),
            "sepa_debit" => Ok(PaymentMethodType::SepaDebit),
            _ => Err(format!("Invalid payment method type: {}", s)),
        }
    }
}

#[async_trait]
impl PaymentMethodRepository for PostgresPaymentMethodRepository {
    async fn create(&self, payment_method: &PaymentMethod) -> Result<PaymentMethod, String> {
        let method_type_str = Self::method_type_to_db(&payment_method.method_type);

        let row = sqlx::query!(
            r#"
            INSERT INTO payment_methods (
                id, organization_id, owner_id, method_type,
                stripe_payment_method_id, stripe_customer_id, display_label,
                is_default, is_active, metadata, expires_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::payment_method_type, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, organization_id, owner_id,
                      method_type AS "method_type: String",
                      stripe_payment_method_id, stripe_customer_id, display_label,
                      is_default, is_active, metadata, expires_at,
                      created_at, updated_at
            "#,
            payment_method.id,
            payment_method.organization_id,
            payment_method.owner_id,
            method_type_str,
            &payment_method.stripe_payment_method_id,
            &payment_method.stripe_customer_id,
            &payment_method.display_label,
            payment_method.is_default,
            payment_method.is_active,
            payment_method.metadata.as_deref(),
            payment_method.expires_at,
            payment_method.created_at,
            payment_method.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create payment method: {}", e))?;

        Ok(PaymentMethod {
            id: row.id,
            organization_id: row.organization_id,
            owner_id: row.owner_id,
            method_type: Self::method_type_from_db(&row.method_type)?,
            stripe_payment_method_id: row.stripe_payment_method_id,
            stripe_customer_id: row.stripe_customer_id,
            display_label: row.display_label,
            is_default: row.is_default,
            is_active: row.is_active,
            metadata: row.metadata,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentMethod>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment method: {}", e))?;

        match row {
            Some(row) => Ok(Some(PaymentMethod {
                id: row.id,
                organization_id: row.organization_id,
                owner_id: row.owner_id,
                method_type: Self::method_type_from_db(&row.method_type)?,
                stripe_payment_method_id: row.stripe_payment_method_id,
                stripe_customer_id: row.stripe_customer_id,
                display_label: row.display_label,
                is_default: row.is_default,
                is_active: row.is_active,
                metadata: row.metadata,
                expires_at: row.expires_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_stripe_payment_method_id(
        &self,
        stripe_payment_method_id: &str,
    ) -> Result<Option<PaymentMethod>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE stripe_payment_method_id = $1
            "#,
            stripe_payment_method_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment method by Stripe ID: {}", e))?;

        match row {
            Some(row) => Ok(Some(PaymentMethod {
                id: row.id,
                organization_id: row.organization_id,
                owner_id: row.owner_id,
                method_type: Self::method_type_from_db(&row.method_type)?,
                stripe_payment_method_id: row.stripe_payment_method_id,
                stripe_customer_id: row.stripe_customer_id,
                display_label: row.display_label,
                is_default: row.is_default,
                is_active: row.is_active,
                metadata: row.metadata,
                expires_at: row.expires_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentMethod>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE owner_id = $1
            ORDER BY is_default DESC, created_at DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment methods by owner: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(PaymentMethod {
                    id: row.id,
                    organization_id: row.organization_id,
                    owner_id: row.owner_id,
                    method_type: Self::method_type_from_db(&row.method_type)?,
                    stripe_payment_method_id: row.stripe_payment_method_id,
                    stripe_customer_id: row.stripe_customer_id,
                    display_label: row.display_label,
                    is_default: row.is_default,
                    is_active: row.is_active,
                    metadata: row.metadata,
                    expires_at: row.expires_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_active_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentMethod>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE owner_id = $1 AND is_active = TRUE
            ORDER BY is_default DESC, created_at DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find active payment methods by owner: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(PaymentMethod {
                    id: row.id,
                    organization_id: row.organization_id,
                    owner_id: row.owner_id,
                    method_type: Self::method_type_from_db(&row.method_type)?,
                    stripe_payment_method_id: row.stripe_payment_method_id,
                    stripe_customer_id: row.stripe_customer_id,
                    display_label: row.display_label,
                    is_default: row.is_default,
                    is_active: row.is_active,
                    metadata: row.metadata,
                    expires_at: row.expires_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_default_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Option<PaymentMethod>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE owner_id = $1 AND is_default = TRUE AND is_active = TRUE
            LIMIT 1
            "#,
            owner_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find default payment method: {}", e))?;

        match row {
            Some(row) => Ok(Some(PaymentMethod {
                id: row.id,
                organization_id: row.organization_id,
                owner_id: row.owner_id,
                method_type: Self::method_type_from_db(&row.method_type)?,
                stripe_payment_method_id: row.stripe_payment_method_id,
                stripe_customer_id: row.stripe_customer_id,
                display_label: row.display_label,
                is_default: row.is_default,
                is_active: row.is_active,
                metadata: row.metadata,
                expires_at: row.expires_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentMethod>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment methods by organization: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(PaymentMethod {
                    id: row.id,
                    organization_id: row.organization_id,
                    owner_id: row.owner_id,
                    method_type: Self::method_type_from_db(&row.method_type)?,
                    stripe_payment_method_id: row.stripe_payment_method_id,
                    stripe_customer_id: row.stripe_customer_id,
                    display_label: row.display_label,
                    is_default: row.is_default,
                    is_active: row.is_active,
                    metadata: row.metadata,
                    expires_at: row.expires_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_owner_and_type(
        &self,
        owner_id: Uuid,
        method_type: PaymentMethodType,
    ) -> Result<Vec<PaymentMethod>, String> {
        let method_type_str = Self::method_type_to_db(&method_type);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, owner_id,
                   method_type AS "method_type: String",
                   stripe_payment_method_id, stripe_customer_id, display_label,
                   is_default, is_active, metadata, expires_at,
                   created_at, updated_at
            FROM payment_methods
            WHERE owner_id = $1 AND method_type = $2::payment_method_type
            ORDER BY is_default DESC, created_at DESC
            "#,
            owner_id,
            method_type_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment methods by owner and type: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(PaymentMethod {
                    id: row.id,
                    organization_id: row.organization_id,
                    owner_id: row.owner_id,
                    method_type: Self::method_type_from_db(&row.method_type)?,
                    stripe_payment_method_id: row.stripe_payment_method_id,
                    stripe_customer_id: row.stripe_customer_id,
                    display_label: row.display_label,
                    is_default: row.is_default,
                    is_active: row.is_active,
                    metadata: row.metadata,
                    expires_at: row.expires_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn update(&self, payment_method: &PaymentMethod) -> Result<PaymentMethod, String> {
        let method_type_str = Self::method_type_to_db(&payment_method.method_type);

        let row = sqlx::query!(
            r#"
            UPDATE payment_methods
            SET organization_id = $2,
                owner_id = $3,
                method_type = $4::payment_method_type,
                stripe_payment_method_id = $5,
                stripe_customer_id = $6,
                display_label = $7,
                is_default = $8,
                is_active = $9,
                metadata = $10,
                expires_at = $11,
                updated_at = $12
            WHERE id = $1
            RETURNING id, organization_id, owner_id,
                      method_type AS "method_type: String",
                      stripe_payment_method_id, stripe_customer_id, display_label,
                      is_default, is_active, metadata, expires_at,
                      created_at, updated_at
            "#,
            payment_method.id,
            payment_method.organization_id,
            payment_method.owner_id,
            method_type_str,
            &payment_method.stripe_payment_method_id,
            &payment_method.stripe_customer_id,
            &payment_method.display_label,
            payment_method.is_default,
            payment_method.is_active,
            payment_method.metadata.as_deref(),
            payment_method.expires_at,
            payment_method.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update payment method: {}", e))?;

        Ok(PaymentMethod {
            id: row.id,
            organization_id: row.organization_id,
            owner_id: row.owner_id,
            method_type: Self::method_type_from_db(&row.method_type)?,
            stripe_payment_method_id: row.stripe_payment_method_id,
            stripe_customer_id: row.stripe_customer_id,
            display_label: row.display_label,
            is_default: row.is_default,
            is_active: row.is_active,
            metadata: row.metadata,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!("DELETE FROM payment_methods WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete payment method: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn set_as_default(&self, id: Uuid, owner_id: Uuid) -> Result<PaymentMethod, String> {
        // Start a transaction to ensure atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Unset all other default payment methods for this owner
        sqlx::query!(
            "UPDATE payment_methods SET is_default = FALSE WHERE owner_id = $1",
            owner_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to unset default payment methods: {}", e))?;

        // Set this payment method as default
        let row = sqlx::query!(
            r#"
            UPDATE payment_methods
            SET is_default = TRUE, updated_at = NOW()
            WHERE id = $1 AND owner_id = $2
            RETURNING id, organization_id, owner_id,
                      method_type AS "method_type: String",
                      stripe_payment_method_id, stripe_customer_id, display_label,
                      is_default, is_active, metadata, expires_at,
                      created_at, updated_at
            "#,
            id,
            owner_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to set payment method as default: {}", e))?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(PaymentMethod {
            id: row.id,
            organization_id: row.organization_id,
            owner_id: row.owner_id,
            method_type: Self::method_type_from_db(&row.method_type)?,
            stripe_payment_method_id: row.stripe_payment_method_id,
            stripe_customer_id: row.stripe_customer_id,
            display_label: row.display_label,
            is_default: row.is_default,
            is_active: row.is_active,
            metadata: row.metadata,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn count_active_by_owner(&self, owner_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM payment_methods
            WHERE owner_id = $1 AND is_active = TRUE
            "#,
            owner_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count active payment methods: {}", e))?;

        Ok(row.count)
    }

    async fn has_active_payment_methods(&self, owner_id: Uuid) -> Result<bool, String> {
        let count = self.count_active_by_owner(owner_id).await?;
        Ok(count > 0)
    }
}
