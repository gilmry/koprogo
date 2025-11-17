use crate::domain::entities::{Payment, TransactionStatus};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    /// Create a new payment
    async fn create(&self, payment: &Payment) -> Result<Payment, String>;

    /// Find payment by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, String>;

    /// Find payment by Stripe payment intent ID
    async fn find_by_stripe_payment_intent_id(
        &self,
        stripe_payment_intent_id: &str,
    ) -> Result<Option<Payment>, String>;

    /// Find payment by idempotency key (prevents duplicate charges)
    async fn find_by_idempotency_key(
        &self,
        organization_id: Uuid,
        idempotency_key: &str,
    ) -> Result<Option<Payment>, String>;

    /// Find all payments for an owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Payment>, String>;

    /// Find all payments for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Payment>, String>;

    /// Find all payments for an expense
    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<Payment>, String>;

    /// Find all payments for an organization
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Payment>, String>;

    /// Find payments by status
    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: TransactionStatus,
    ) -> Result<Vec<Payment>, String>;

    /// Find payments by status and building
    async fn find_by_building_and_status(
        &self,
        building_id: Uuid,
        status: TransactionStatus,
    ) -> Result<Vec<Payment>, String>;

    /// Find pending payments (for background processing)
    async fn find_pending(&self, organization_id: Uuid) -> Result<Vec<Payment>, String>;

    /// Find failed payments (for retry or analysis)
    async fn find_failed(&self, organization_id: Uuid) -> Result<Vec<Payment>, String>;

    /// Update payment
    async fn update(&self, payment: &Payment) -> Result<Payment, String>;

    /// Delete payment (soft delete recommended in production)
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Get total amount paid for an expense (sum of succeeded payments)
    async fn get_total_paid_for_expense(&self, expense_id: Uuid) -> Result<i64, String>;

    /// Get total amount paid by owner (sum of succeeded payments)
    async fn get_total_paid_by_owner(&self, owner_id: Uuid) -> Result<i64, String>;

    /// Get total amount paid for building (sum of succeeded payments)
    async fn get_total_paid_for_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Get payment statistics for owner
    async fn get_owner_payment_stats(
        &self,
        owner_id: Uuid,
    ) -> Result<PaymentStats, String>;

    /// Get payment statistics for building
    async fn get_building_payment_stats(
        &self,
        building_id: Uuid,
    ) -> Result<PaymentStats, String>;
}

/// Payment statistics
#[derive(Debug, Clone)]
pub struct PaymentStats {
    pub total_count: i64,
    pub succeeded_count: i64,
    pub failed_count: i64,
    pub pending_count: i64,
    pub total_amount_cents: i64,
    pub total_succeeded_cents: i64,
    pub total_refunded_cents: i64,
    pub net_amount_cents: i64, // succeeded - refunded
}
