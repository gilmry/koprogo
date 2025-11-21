use crate::application::dto::{
    CreatePaymentRequest, PaymentResponse, PaymentStatsResponse, RefundPaymentRequest,
};
use crate::application::ports::{
    PaymentMethodRepository, PaymentRepository, PaymentStats,
};
use crate::domain::entities::{Payment, TransactionStatus};
use std::sync::Arc;
use uuid::Uuid;

pub struct PaymentUseCases {
    payment_repository: Arc<dyn PaymentRepository>,
    payment_method_repository: Arc<dyn PaymentMethodRepository>,
}

impl PaymentUseCases {
    pub fn new(
        payment_repository: Arc<dyn PaymentRepository>,
        payment_method_repository: Arc<dyn PaymentMethodRepository>,
    ) -> Self {
        Self {
            payment_repository,
            payment_method_repository,
        }
    }

    /// Create a new payment
    ///
    /// Generates a unique idempotency key to prevent duplicate charges.
    /// Checks for existing payment with same idempotency key (prevents retries from creating duplicates).
    pub async fn create_payment(
        &self,
        organization_id: Uuid,
        request: CreatePaymentRequest,
    ) -> Result<PaymentResponse, String> {
        // Generate idempotency key (organization_id + building_id + owner_id + timestamp + random)
        let idempotency_key = format!(
            "{}-{}-{}-{}",
            organization_id,
            request.building_id,
            request.owner_id,
            Uuid::new_v4()
        );

        // Check if payment with same idempotency key already exists
        if let Some(existing_payment) = self
            .payment_repository
            .find_by_idempotency_key(organization_id, &idempotency_key)
            .await?
        {
            // Return existing payment (idempotent)
            return Ok(PaymentResponse::from(existing_payment));
        }

        // Create new payment
        let payment = Payment::new(
            organization_id,
            request.building_id,
            request.owner_id,
            request.expense_id,
            request.amount_cents,
            request.payment_method_type,
            idempotency_key,
            request.description,
        )?;

        let created = self.payment_repository.create(&payment).await?;
        Ok(PaymentResponse::from(created))
    }

    /// Get payment by ID
    pub async fn get_payment(&self, id: Uuid) -> Result<Option<PaymentResponse>, String> {
        match self.payment_repository.find_by_id(id).await? {
            Some(payment) => Ok(Some(PaymentResponse::from(payment))),
            None => Ok(None),
        }
    }

    /// Get payment by Stripe payment intent ID
    pub async fn get_payment_by_stripe_intent(
        &self,
        stripe_payment_intent_id: &str,
    ) -> Result<Option<PaymentResponse>, String> {
        match self
            .payment_repository
            .find_by_stripe_payment_intent_id(stripe_payment_intent_id)
            .await?
        {
            Some(payment) => Ok(Some(PaymentResponse::from(payment))),
            None => Ok(None),
        }
    }

    /// List payments for an owner
    pub async fn list_owner_payments(&self, owner_id: Uuid) -> Result<Vec<PaymentResponse>, String> {
        let payments = self.payment_repository.find_by_owner(owner_id).await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List payments for a building
    pub async fn list_building_payments(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self.payment_repository.find_by_building(building_id).await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List payments for an expense
    pub async fn list_expense_payments(
        &self,
        expense_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self.payment_repository.find_by_expense(expense_id).await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List payments for an organization
    pub async fn list_organization_payments(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self
            .payment_repository
            .find_by_organization(organization_id)
            .await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List payments by status
    pub async fn list_payments_by_status(
        &self,
        organization_id: Uuid,
        status: TransactionStatus,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self
            .payment_repository
            .find_by_status(organization_id, status)
            .await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List pending payments (for background processing)
    pub async fn list_pending_payments(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self.payment_repository.find_pending(organization_id).await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List failed payments (for retry or analysis)
    pub async fn list_failed_payments(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self.payment_repository.find_failed(organization_id).await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// Mark payment as processing
    pub async fn mark_processing(&self, id: Uuid) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.mark_processing()?;

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Mark payment as requiring action (e.g., 3D Secure)
    pub async fn mark_requires_action(&self, id: Uuid) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.mark_requires_action()?;

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Mark payment as succeeded
    pub async fn mark_succeeded(&self, id: Uuid) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.mark_succeeded()?;

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Mark payment as failed
    pub async fn mark_failed(&self, id: Uuid, reason: String) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.mark_failed(reason)?;

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Mark payment as cancelled
    pub async fn mark_cancelled(&self, id: Uuid) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.mark_cancelled()?;

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Refund payment (partial or full)
    pub async fn refund_payment(
        &self,
        id: Uuid,
        request: RefundPaymentRequest,
    ) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.refund(request.amount_cents)?;

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Set Stripe payment intent ID
    pub async fn set_stripe_payment_intent_id(
        &self,
        id: Uuid,
        stripe_payment_intent_id: String,
    ) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.set_stripe_payment_intent_id(stripe_payment_intent_id);

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Set Stripe customer ID
    pub async fn set_stripe_customer_id(
        &self,
        id: Uuid,
        stripe_customer_id: String,
    ) -> Result<PaymentResponse, String> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.set_stripe_customer_id(stripe_customer_id);

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Set payment method ID
    pub async fn set_payment_method_id(
        &self,
        id: Uuid,
        payment_method_id: Uuid,
    ) -> Result<PaymentResponse, String> {
        // Verify payment method exists
        let _payment_method = self
            .payment_method_repository
            .find_by_id(payment_method_id)
            .await?
            .ok_or_else(|| "Payment method not found".to_string())?;

        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        payment.set_payment_method_id(payment_method_id);

        let updated = self.payment_repository.update(&payment).await?;
        Ok(PaymentResponse::from(updated))
    }

    /// Delete payment
    pub async fn delete_payment(&self, id: Uuid) -> Result<bool, String> {
        self.payment_repository.delete(id).await
    }

    /// Get total paid for expense
    pub async fn get_total_paid_for_expense(&self, expense_id: Uuid) -> Result<i64, String> {
        self.payment_repository
            .get_total_paid_for_expense(expense_id)
            .await
    }

    /// Get total paid by owner
    pub async fn get_total_paid_by_owner(&self, owner_id: Uuid) -> Result<i64, String> {
        self.payment_repository
            .get_total_paid_by_owner(owner_id)
            .await
    }

    /// Get total paid for building
    pub async fn get_total_paid_for_building(&self, building_id: Uuid) -> Result<i64, String> {
        self.payment_repository
            .get_total_paid_for_building(building_id)
            .await
    }

    /// Get payment statistics for owner
    pub async fn get_owner_payment_stats(
        &self,
        owner_id: Uuid,
    ) -> Result<PaymentStatsResponse, String> {
        let stats = self
            .payment_repository
            .get_owner_payment_stats(owner_id)
            .await?;
        Ok(Self::payment_stats_to_response(stats))
    }

    /// Get payment statistics for building
    pub async fn get_building_payment_stats(
        &self,
        building_id: Uuid,
    ) -> Result<PaymentStatsResponse, String> {
        let stats = self
            .payment_repository
            .get_building_payment_stats(building_id)
            .await?;
        Ok(Self::payment_stats_to_response(stats))
    }

    /// Convert PaymentStats to PaymentStatsResponse
    fn payment_stats_to_response(stats: PaymentStats) -> PaymentStatsResponse {
        PaymentStatsResponse {
            total_count: stats.total_count,
            succeeded_count: stats.succeeded_count,
            failed_count: stats.failed_count,
            pending_count: stats.pending_count,
            total_amount_cents: stats.total_amount_cents,
            total_succeeded_cents: stats.total_succeeded_cents,
            total_refunded_cents: stats.total_refunded_cents,
            net_amount_cents: stats.net_amount_cents,
        }
    }
}
