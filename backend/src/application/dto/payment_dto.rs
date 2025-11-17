use crate::domain::entities::{Payment, PaymentMethodType, TransactionStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub owner_id: Uuid,
    pub expense_id: Option<Uuid>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: TransactionStatus,
    pub payment_method_type: PaymentMethodType,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub payment_method_id: Option<Uuid>,
    pub idempotency_key: String,
    pub description: Option<String>,
    pub metadata: Option<String>,
    pub failure_reason: Option<String>,
    pub refunded_amount_cents: i64,
    pub net_amount_cents: i64, // amount_cents - refunded_amount_cents
    pub succeeded_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Payment> for PaymentResponse {
    fn from(payment: Payment) -> Self {
        let net_amount_cents = payment.get_net_amount_cents();

        Self {
            id: payment.id,
            organization_id: payment.organization_id,
            building_id: payment.building_id,
            owner_id: payment.owner_id,
            expense_id: payment.expense_id,
            amount_cents: payment.amount_cents,
            currency: payment.currency,
            status: payment.status,
            payment_method_type: payment.payment_method_type,
            stripe_payment_intent_id: payment.stripe_payment_intent_id,
            stripe_customer_id: payment.stripe_customer_id,
            payment_method_id: payment.payment_method_id,
            idempotency_key: payment.idempotency_key,
            description: payment.description,
            metadata: payment.metadata,
            failure_reason: payment.failure_reason,
            refunded_amount_cents: payment.refunded_amount_cents,
            net_amount_cents,
            succeeded_at: payment.succeeded_at,
            failed_at: payment.failed_at,
            cancelled_at: payment.cancelled_at,
            created_at: payment.created_at,
            updated_at: payment.updated_at,
        }
    }
}

/// Create payment request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub building_id: Uuid,
    pub owner_id: Uuid,
    pub expense_id: Option<Uuid>,
    pub amount_cents: i64,
    pub payment_method_type: PaymentMethodType,
    pub payment_method_id: Option<Uuid>, // If using saved payment method
    pub description: Option<String>,
    pub metadata: Option<String>,
}

/// Refund payment request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundPaymentRequest {
    pub amount_cents: i64,
    pub reason: Option<String>,
}

/// Payment statistics response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatsResponse {
    pub total_count: i64,
    pub succeeded_count: i64,
    pub failed_count: i64,
    pub pending_count: i64,
    pub total_amount_cents: i64,
    pub total_succeeded_cents: i64,
    pub total_refunded_cents: i64,
    pub net_amount_cents: i64,
}
