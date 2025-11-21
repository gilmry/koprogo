use crate::application::ports::{PaymentRepository, PaymentStats};
use crate::domain::entities::{Payment, PaymentMethodType, TransactionStatus};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of PaymentRepository
pub struct PostgresPaymentRepository {
    pool: PgPool,
}

impl PostgresPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert TransactionStatus enum to database string
    fn status_to_db(status: &TransactionStatus) -> &'static str {
        match status {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Processing => "processing",
            TransactionStatus::RequiresAction => "requires_action",
            TransactionStatus::Succeeded => "succeeded",
            TransactionStatus::Failed => "failed",
            TransactionStatus::Cancelled => "cancelled",
            TransactionStatus::Refunded => "refunded",
        }
    }

    /// Convert database string to TransactionStatus enum
    fn status_from_db(s: &str) -> Result<TransactionStatus, String> {
        match s {
            "pending" => Ok(TransactionStatus::Pending),
            "processing" => Ok(TransactionStatus::Processing),
            "requires_action" => Ok(TransactionStatus::RequiresAction),
            "succeeded" => Ok(TransactionStatus::Succeeded),
            "failed" => Ok(TransactionStatus::Failed),
            "cancelled" => Ok(TransactionStatus::Cancelled),
            "refunded" => Ok(TransactionStatus::Refunded),
            _ => Err(format!("Invalid transaction status: {}", s)),
        }
    }

    /// Convert PaymentMethodType enum to database string
    fn method_type_to_db(method_type: &PaymentMethodType) -> &'static str {
        match method_type {
            PaymentMethodType::Card => "card",
            PaymentMethodType::SepaDebit => "sepa_debit",
            PaymentMethodType::BankTransfer => "bank_transfer",
            PaymentMethodType::Cash => "cash",
        }
    }

    /// Convert database string to PaymentMethodType enum
    fn method_type_from_db(s: &str) -> Result<PaymentMethodType, String> {
        match s {
            "card" => Ok(PaymentMethodType::Card),
            "sepa_debit" => Ok(PaymentMethodType::SepaDebit),
            "bank_transfer" => Ok(PaymentMethodType::BankTransfer),
            "cash" => Ok(PaymentMethodType::Cash),
            _ => Err(format!("Invalid payment method type: {}", s)),
        }
    }
}

#[async_trait]
impl PaymentRepository for PostgresPaymentRepository {
    async fn create(&self, payment: &Payment) -> Result<Payment, String> {
        let status_str = Self::status_to_db(&payment.status);
        let method_type_str = Self::method_type_to_db(&payment.payment_method_type);

        let row = sqlx::query!(
            r#"
            INSERT INTO payments (
                id, organization_id, building_id, owner_id, expense_id,
                amount_cents, currency, status, payment_method_type,
                stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                idempotency_key, description, metadata, failure_reason,
                refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::TEXT::transaction_status, $9::TEXT::payment_method_type,
                    $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING id, organization_id, building_id, owner_id, expense_id,
                      amount_cents, currency, status AS "status: String",
                      payment_method_type AS "payment_method_type: String",
                      stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                      idempotency_key, description, metadata, failure_reason,
                      refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                      created_at, updated_at
            "#,
            payment.id,
            payment.organization_id,
            payment.building_id,
            payment.owner_id,
            payment.expense_id,
            payment.amount_cents,
            &payment.currency,
            status_str,
            method_type_str,
            payment.stripe_payment_intent_id.as_deref(),
            payment.stripe_customer_id.as_deref(),
            payment.payment_method_id,
            &payment.idempotency_key,
            payment.description.as_deref(),
            payment.metadata.as_deref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()),
            payment.failure_reason.as_deref(),
            payment.refunded_amount_cents,
            payment.succeeded_at,
            payment.failed_at,
            payment.cancelled_at,
            payment.created_at,
            payment.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create payment: {}", e))?;

        Ok(Payment {
            id: row.id,
            organization_id: row.organization_id,
            building_id: row.building_id,
            owner_id: row.owner_id,
            expense_id: row.expense_id,
            amount_cents: row.amount_cents,
            currency: row.currency,
            status: Self::status_from_db(&row.status)?,
            payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
            stripe_payment_intent_id: row.stripe_payment_intent_id,
            stripe_customer_id: row.stripe_customer_id,
            payment_method_id: row.payment_method_id,
            idempotency_key: row.idempotency_key,
            description: row.description,
            metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
            failure_reason: row.failure_reason,
            refunded_amount_cents: row.refunded_amount_cents,
            succeeded_at: row.succeeded_at,
            failed_at: row.failed_at,
            cancelled_at: row.cancelled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment: {}", e))?;

        match row {
            Some(row) => Ok(Some(Payment {
                id: row.id,
                organization_id: row.organization_id,
                building_id: row.building_id,
                owner_id: row.owner_id,
                expense_id: row.expense_id,
                amount_cents: row.amount_cents,
                currency: row.currency,
                status: Self::status_from_db(&row.status)?,
                payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                stripe_payment_intent_id: row.stripe_payment_intent_id,
                stripe_customer_id: row.stripe_customer_id,
                payment_method_id: row.payment_method_id,
                idempotency_key: row.idempotency_key,
                description: row.description,
                metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                failure_reason: row.failure_reason,
                refunded_amount_cents: row.refunded_amount_cents,
                succeeded_at: row.succeeded_at,
                failed_at: row.failed_at,
                cancelled_at: row.cancelled_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_stripe_payment_intent_id(
        &self,
        stripe_payment_intent_id: &str,
    ) -> Result<Option<Payment>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE stripe_payment_intent_id = $1
            "#,
            stripe_payment_intent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment by Stripe payment intent: {}", e))?;

        match row {
            Some(row) => Ok(Some(Payment {
                id: row.id,
                organization_id: row.organization_id,
                building_id: row.building_id,
                owner_id: row.owner_id,
                expense_id: row.expense_id,
                amount_cents: row.amount_cents,
                currency: row.currency,
                status: Self::status_from_db(&row.status)?,
                payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                stripe_payment_intent_id: row.stripe_payment_intent_id,
                stripe_customer_id: row.stripe_customer_id,
                payment_method_id: row.payment_method_id,
                idempotency_key: row.idempotency_key,
                description: row.description,
                metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                failure_reason: row.failure_reason,
                refunded_amount_cents: row.refunded_amount_cents,
                succeeded_at: row.succeeded_at,
                failed_at: row.failed_at,
                cancelled_at: row.cancelled_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_idempotency_key(
        &self,
        organization_id: Uuid,
        idempotency_key: &str,
    ) -> Result<Option<Payment>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE organization_id = $1 AND idempotency_key = $2
            "#,
            organization_id,
            idempotency_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment by idempotency key: {}", e))?;

        match row {
            Some(row) => Ok(Some(Payment {
                id: row.id,
                organization_id: row.organization_id,
                building_id: row.building_id,
                owner_id: row.owner_id,
                expense_id: row.expense_id,
                amount_cents: row.amount_cents,
                currency: row.currency,
                status: Self::status_from_db(&row.status)?,
                payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                stripe_payment_intent_id: row.stripe_payment_intent_id,
                stripe_customer_id: row.stripe_customer_id,
                payment_method_id: row.payment_method_id,
                idempotency_key: row.idempotency_key,
                description: row.description,
                metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                failure_reason: row.failure_reason,
                refunded_amount_cents: row.refunded_amount_cents,
                succeeded_at: row.succeeded_at,
                failed_at: row.failed_at,
                cancelled_at: row.cancelled_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Payment>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payments by owner: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Payment {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    owner_id: row.owner_id,
                    expense_id: row.expense_id,
                    amount_cents: row.amount_cents,
                    currency: row.currency,
                    status: Self::status_from_db(&row.status)?,
                    payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                    stripe_payment_intent_id: row.stripe_payment_intent_id,
                    stripe_customer_id: row.stripe_customer_id,
                    payment_method_id: row.payment_method_id,
                    idempotency_key: row.idempotency_key,
                    description: row.description,
                    metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                    failure_reason: row.failure_reason,
                    refunded_amount_cents: row.refunded_amount_cents,
                    succeeded_at: row.succeeded_at,
                    failed_at: row.failed_at,
                    cancelled_at: row.cancelled_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Payment>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payments by building: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Payment {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    owner_id: row.owner_id,
                    expense_id: row.expense_id,
                    amount_cents: row.amount_cents,
                    currency: row.currency,
                    status: Self::status_from_db(&row.status)?,
                    payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                    stripe_payment_intent_id: row.stripe_payment_intent_id,
                    stripe_customer_id: row.stripe_customer_id,
                    payment_method_id: row.payment_method_id,
                    idempotency_key: row.idempotency_key,
                    description: row.description,
                    metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                    failure_reason: row.failure_reason,
                    refunded_amount_cents: row.refunded_amount_cents,
                    succeeded_at: row.succeeded_at,
                    failed_at: row.failed_at,
                    cancelled_at: row.cancelled_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<Payment>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE expense_id = $1
            ORDER BY created_at DESC
            "#,
            expense_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payments by expense: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Payment {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    owner_id: row.owner_id,
                    expense_id: row.expense_id,
                    amount_cents: row.amount_cents,
                    currency: row.currency,
                    status: Self::status_from_db(&row.status)?,
                    payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                    stripe_payment_intent_id: row.stripe_payment_intent_id,
                    stripe_customer_id: row.stripe_customer_id,
                    payment_method_id: row.payment_method_id,
                    idempotency_key: row.idempotency_key,
                    description: row.description,
                    metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                    failure_reason: row.failure_reason,
                    refunded_amount_cents: row.refunded_amount_cents,
                    succeeded_at: row.succeeded_at,
                    failed_at: row.failed_at,
                    cancelled_at: row.cancelled_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Payment>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payments by organization: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Payment {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    owner_id: row.owner_id,
                    expense_id: row.expense_id,
                    amount_cents: row.amount_cents,
                    currency: row.currency,
                    status: Self::status_from_db(&row.status)?,
                    payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                    stripe_payment_intent_id: row.stripe_payment_intent_id,
                    stripe_customer_id: row.stripe_customer_id,
                    payment_method_id: row.payment_method_id,
                    idempotency_key: row.idempotency_key,
                    description: row.description,
                    metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                    failure_reason: row.failure_reason,
                    refunded_amount_cents: row.refunded_amount_cents,
                    succeeded_at: row.succeeded_at,
                    failed_at: row.failed_at,
                    cancelled_at: row.cancelled_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: TransactionStatus,
    ) -> Result<Vec<Payment>, String> {
        let status_str = Self::status_to_db(&status);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE organization_id = $1 AND status = $2::TEXT::transaction_status
            ORDER BY created_at DESC
            "#,
            organization_id,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payments by status: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Payment {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    owner_id: row.owner_id,
                    expense_id: row.expense_id,
                    amount_cents: row.amount_cents,
                    currency: row.currency,
                    status: Self::status_from_db(&row.status)?,
                    payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                    stripe_payment_intent_id: row.stripe_payment_intent_id,
                    stripe_customer_id: row.stripe_customer_id,
                    payment_method_id: row.payment_method_id,
                    idempotency_key: row.idempotency_key,
                    description: row.description,
                    metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                    failure_reason: row.failure_reason,
                    refunded_amount_cents: row.refunded_amount_cents,
                    succeeded_at: row.succeeded_at,
                    failed_at: row.failed_at,
                    cancelled_at: row.cancelled_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_building_and_status(
        &self,
        building_id: Uuid,
        status: TransactionStatus,
    ) -> Result<Vec<Payment>, String> {
        let status_str = Self::status_to_db(&status);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, owner_id, expense_id,
                   amount_cents, currency, status AS "status: String",
                   payment_method_type AS "payment_method_type: String",
                   stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                   idempotency_key, description, metadata, failure_reason,
                   refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                   created_at, updated_at
            FROM payments
            WHERE building_id = $1 AND status = $2::TEXT::transaction_status
            ORDER BY created_at DESC
            "#,
            building_id,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payments by building and status: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Payment {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    owner_id: row.owner_id,
                    expense_id: row.expense_id,
                    amount_cents: row.amount_cents,
                    currency: row.currency,
                    status: Self::status_from_db(&row.status)?,
                    payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
                    stripe_payment_intent_id: row.stripe_payment_intent_id,
                    stripe_customer_id: row.stripe_customer_id,
                    payment_method_id: row.payment_method_id,
                    idempotency_key: row.idempotency_key,
                    description: row.description,
                    metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
                    failure_reason: row.failure_reason,
                    refunded_amount_cents: row.refunded_amount_cents,
                    succeeded_at: row.succeeded_at,
                    failed_at: row.failed_at,
                    cancelled_at: row.cancelled_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_pending(&self, organization_id: Uuid) -> Result<Vec<Payment>, String> {
        self.find_by_status(organization_id, TransactionStatus::Pending)
            .await
    }

    async fn find_failed(&self, organization_id: Uuid) -> Result<Vec<Payment>, String> {
        self.find_by_status(organization_id, TransactionStatus::Failed)
            .await
    }

    async fn update(&self, payment: &Payment) -> Result<Payment, String> {
        let status_str = Self::status_to_db(&payment.status);
        let method_type_str = Self::method_type_to_db(&payment.payment_method_type);

        let row = sqlx::query!(
            r#"
            UPDATE payments
            SET organization_id = $2,
                building_id = $3,
                owner_id = $4,
                expense_id = $5,
                amount_cents = $6,
                currency = $7,
                status = $8::TEXT::transaction_status,
                payment_method_type = $9::TEXT::payment_method_type,
                stripe_payment_intent_id = $10,
                stripe_customer_id = $11,
                payment_method_id = $12,
                idempotency_key = $13,
                description = $14,
                metadata = $15,
                failure_reason = $16,
                refunded_amount_cents = $17,
                succeeded_at = $18,
                failed_at = $19,
                cancelled_at = $20,
                updated_at = $21
            WHERE id = $1
            RETURNING id, organization_id, building_id, owner_id, expense_id,
                      amount_cents, currency, status AS "status: String",
                      payment_method_type AS "payment_method_type: String",
                      stripe_payment_intent_id, stripe_customer_id, payment_method_id,
                      idempotency_key, description, metadata, failure_reason,
                      refunded_amount_cents, succeeded_at, failed_at, cancelled_at,
                      created_at, updated_at
            "#,
            payment.id,
            payment.organization_id,
            payment.building_id,
            payment.owner_id,
            payment.expense_id,
            payment.amount_cents,
            &payment.currency,
            status_str,
            method_type_str,
            payment.stripe_payment_intent_id.as_deref(),
            payment.stripe_customer_id.as_deref(),
            payment.payment_method_id,
            &payment.idempotency_key,
            payment.description.as_deref(),
            payment.metadata.as_deref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()),
            payment.failure_reason.as_deref(),
            payment.refunded_amount_cents,
            payment.succeeded_at,
            payment.failed_at,
            payment.cancelled_at,
            payment.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update payment: {}", e))?;

        Ok(Payment {
            id: row.id,
            organization_id: row.organization_id,
            building_id: row.building_id,
            owner_id: row.owner_id,
            expense_id: row.expense_id,
            amount_cents: row.amount_cents,
            currency: row.currency,
            status: Self::status_from_db(&row.status)?,
            payment_method_type: Self::method_type_from_db(&row.payment_method_type)?,
            stripe_payment_intent_id: row.stripe_payment_intent_id,
            stripe_customer_id: row.stripe_customer_id,
            payment_method_id: row.payment_method_id,
            idempotency_key: row.idempotency_key,
            description: row.description,
            metadata: row.metadata.map(|v: serde_json::Value| v.to_string()),
            failure_reason: row.failure_reason,
            refunded_amount_cents: row.refunded_amount_cents,
            succeeded_at: row.succeeded_at,
            failed_at: row.failed_at,
            cancelled_at: row.cancelled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!("DELETE FROM payments WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete payment: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_total_paid_for_expense(&self, expense_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_cents - refunded_amount_cents), 0)::BIGINT AS "total!"
            FROM payments
            WHERE expense_id = $1 AND status = 'succeeded'
            "#,
            expense_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get total paid for expense: {}", e))?;

        Ok(row.total)
    }

    async fn get_total_paid_by_owner(&self, owner_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_cents - refunded_amount_cents), 0)::BIGINT AS "total!"
            FROM payments
            WHERE owner_id = $1 AND status = 'succeeded'
            "#,
            owner_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get total paid by owner: {}", e))?;

        Ok(row.total)
    }

    async fn get_total_paid_for_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_cents - refunded_amount_cents), 0)::BIGINT AS "total!"
            FROM payments
            WHERE building_id = $1 AND status = 'succeeded'
            "#,
            building_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get total paid for building: {}", e))?;

        Ok(row.total)
    }

    async fn get_owner_payment_stats(&self, owner_id: Uuid) -> Result<PaymentStats, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) AS "total_count!",
                COUNT(*) FILTER (WHERE status = 'succeeded') AS "succeeded_count!",
                COUNT(*) FILTER (WHERE status = 'failed') AS "failed_count!",
                COUNT(*) FILTER (WHERE status = 'pending') AS "pending_count!",
                COALESCE(SUM(amount_cents)::BIGINT, 0) AS "total_amount_cents!",
                COALESCE((SUM(amount_cents) FILTER (WHERE status = 'succeeded'))::BIGINT, 0) AS "total_succeeded_cents!",
                COALESCE(SUM(refunded_amount_cents)::BIGINT, 0) AS "total_refunded_cents!"
            FROM payments
            WHERE owner_id = $1
            "#,
            owner_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get owner payment stats: {}", e))?;

        Ok(PaymentStats {
            total_count: row.total_count,
            succeeded_count: row.succeeded_count,
            failed_count: row.failed_count,
            pending_count: row.pending_count,
            total_amount_cents: row.total_amount_cents,
            total_succeeded_cents: row.total_succeeded_cents,
            total_refunded_cents: row.total_refunded_cents,
            net_amount_cents: row.total_succeeded_cents - row.total_refunded_cents,
        })
    }

    async fn get_building_payment_stats(
        &self,
        building_id: Uuid,
    ) -> Result<PaymentStats, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) AS "total_count!",
                COUNT(*) FILTER (WHERE status = 'succeeded') AS "succeeded_count!",
                COUNT(*) FILTER (WHERE status = 'failed') AS "failed_count!",
                COUNT(*) FILTER (WHERE status = 'pending') AS "pending_count!",
                COALESCE(SUM(amount_cents)::BIGINT, 0) AS "total_amount_cents!",
                COALESCE((SUM(amount_cents) FILTER (WHERE status = 'succeeded'))::BIGINT, 0) AS "total_succeeded_cents!",
                COALESCE(SUM(refunded_amount_cents)::BIGINT, 0) AS "total_refunded_cents!"
            FROM payments
            WHERE building_id = $1
            "#,
            building_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get building payment stats: {}", e))?;

        Ok(PaymentStats {
            total_count: row.total_count,
            succeeded_count: row.succeeded_count,
            failed_count: row.failed_count,
            pending_count: row.pending_count,
            total_amount_cents: row.total_amount_cents,
            total_succeeded_cents: row.total_succeeded_cents,
            total_refunded_cents: row.total_refunded_cents,
            net_amount_cents: row.total_succeeded_cents - row.total_refunded_cents,
        })
    }
}
