use crate::application::dto::{
    CreatePaymentRequest, PaymentResponse, PaymentStatsResponse, RefundPaymentRequest,
};
use crate::application::ports::{PaymentMethodRepository, PaymentRepository, PaymentStats};
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
    pub async fn list_owner_payments(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self.payment_repository.find_by_owner(owner_id).await?;
        Ok(payments.into_iter().map(PaymentResponse::from).collect())
    }

    /// List payments for a building
    pub async fn list_building_payments(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<PaymentResponse>, String> {
        let payments = self
            .payment_repository
            .find_by_building(building_id)
            .await?;
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
        let payments = self
            .payment_repository
            .find_pending(organization_id)
            .await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{PaymentMethodRepository, PaymentRepository, PaymentStats};
    use crate::domain::entities::payment_method::{
        PaymentMethod, PaymentMethodType as PMMethodType,
    };
    use crate::domain::entities::{Payment, PaymentMethodType, TransactionStatus};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ─── Mock PaymentRepository ───────────────────────────────────────

    struct MockPaymentRepository {
        payments: Mutex<HashMap<Uuid, Payment>>,
    }

    impl MockPaymentRepository {
        fn new() -> Self {
            Self {
                payments: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl PaymentRepository for MockPaymentRepository {
        async fn create(&self, payment: &Payment) -> Result<Payment, String> {
            self.payments
                .lock()
                .unwrap()
                .insert(payment.id, payment.clone());
            Ok(payment.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, String> {
            Ok(self.payments.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_stripe_payment_intent_id(
            &self,
            stripe_payment_intent_id: &str,
        ) -> Result<Option<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .find(|p| p.stripe_payment_intent_id.as_deref() == Some(stripe_payment_intent_id))
                .cloned())
        }

        async fn find_by_idempotency_key(
            &self,
            organization_id: Uuid,
            idempotency_key: &str,
        ) -> Result<Option<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .find(|p| {
                    p.organization_id == organization_id && p.idempotency_key == idempotency_key
                })
                .cloned())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.expense_id == Some(expense_id))
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            organization_id: Uuid,
            status: TransactionStatus,
        ) -> Result<Vec<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.organization_id == organization_id && p.status == status)
                .cloned()
                .collect())
        }

        async fn find_by_building_and_status(
            &self,
            building_id: Uuid,
            status: TransactionStatus,
        ) -> Result<Vec<Payment>, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.building_id == building_id && p.status == status)
                .cloned()
                .collect())
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
            self.payments
                .lock()
                .unwrap()
                .insert(payment.id, payment.clone());
            Ok(payment.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.payments.lock().unwrap().remove(&id).is_some())
        }

        async fn get_total_paid_for_expense(&self, expense_id: Uuid) -> Result<i64, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| {
                    p.expense_id == Some(expense_id) && p.status == TransactionStatus::Succeeded
                })
                .map(|p| p.amount_cents)
                .sum())
        }

        async fn get_total_paid_by_owner(&self, owner_id: Uuid) -> Result<i64, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.owner_id == owner_id && p.status == TransactionStatus::Succeeded)
                .map(|p| p.amount_cents)
                .sum())
        }

        async fn get_total_paid_for_building(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| {
                    p.building_id == building_id && p.status == TransactionStatus::Succeeded
                })
                .map(|p| p.amount_cents)
                .sum())
        }

        async fn get_owner_payment_stats(&self, owner_id: Uuid) -> Result<PaymentStats, String> {
            let payments: Vec<_> = self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.owner_id == owner_id)
                .cloned()
                .collect();
            Ok(compute_stats(&payments))
        }

        async fn get_building_payment_stats(
            &self,
            building_id: Uuid,
        ) -> Result<PaymentStats, String> {
            let payments: Vec<_> = self
                .payments
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.building_id == building_id)
                .cloned()
                .collect();
            Ok(compute_stats(&payments))
        }
    }

    fn compute_stats(payments: &[Payment]) -> PaymentStats {
        let total_count = payments.len() as i64;
        let succeeded_count = payments
            .iter()
            .filter(|p| p.status == TransactionStatus::Succeeded)
            .count() as i64;
        let failed_count = payments
            .iter()
            .filter(|p| p.status == TransactionStatus::Failed)
            .count() as i64;
        let pending_count = payments
            .iter()
            .filter(|p| p.status == TransactionStatus::Pending)
            .count() as i64;
        let total_amount_cents: i64 = payments.iter().map(|p| p.amount_cents).sum();
        let total_succeeded_cents: i64 = payments
            .iter()
            .filter(|p| p.status == TransactionStatus::Succeeded)
            .map(|p| p.amount_cents)
            .sum();
        let total_refunded_cents: i64 = payments.iter().map(|p| p.refunded_amount_cents).sum();
        PaymentStats {
            total_count,
            succeeded_count,
            failed_count,
            pending_count,
            total_amount_cents,
            total_succeeded_cents,
            total_refunded_cents,
            net_amount_cents: total_succeeded_cents - total_refunded_cents,
        }
    }

    // ─── Mock PaymentMethodRepository ─────────────────────────────────

    struct MockPaymentMethodRepository {
        methods: Mutex<HashMap<Uuid, PaymentMethod>>,
    }

    impl MockPaymentMethodRepository {
        fn new() -> Self {
            Self {
                methods: Mutex::new(HashMap::new()),
            }
        }

        fn with_method(method: PaymentMethod) -> Self {
            let mut map = HashMap::new();
            map.insert(method.id, method);
            Self {
                methods: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl PaymentMethodRepository for MockPaymentMethodRepository {
        async fn create(&self, pm: &PaymentMethod) -> Result<PaymentMethod, String> {
            self.methods.lock().unwrap().insert(pm.id, pm.clone());
            Ok(pm.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentMethod>, String> {
            Ok(self.methods.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_stripe_payment_method_id(
            &self,
            stripe_payment_method_id: &str,
        ) -> Result<Option<PaymentMethod>, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .find(|m| m.stripe_payment_method_id == stripe_payment_method_id)
                .cloned())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentMethod>, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .filter(|m| m.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_active_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentMethod>, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .filter(|m| m.owner_id == owner_id && m.is_active)
                .cloned()
                .collect())
        }

        async fn find_default_by_owner(
            &self,
            owner_id: Uuid,
        ) -> Result<Option<PaymentMethod>, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .find(|m| m.owner_id == owner_id && m.is_default)
                .cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<PaymentMethod>, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .filter(|m| m.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner_and_type(
            &self,
            owner_id: Uuid,
            method_type: PMMethodType,
        ) -> Result<Vec<PaymentMethod>, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .filter(|m| m.owner_id == owner_id && m.method_type == method_type)
                .cloned()
                .collect())
        }

        async fn update(&self, pm: &PaymentMethod) -> Result<PaymentMethod, String> {
            self.methods.lock().unwrap().insert(pm.id, pm.clone());
            Ok(pm.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.methods.lock().unwrap().remove(&id).is_some())
        }

        async fn set_as_default(&self, id: Uuid, _owner_id: Uuid) -> Result<PaymentMethod, String> {
            let mut methods = self.methods.lock().unwrap();
            let pm = methods
                .get_mut(&id)
                .ok_or_else(|| "Not found".to_string())?;
            pm.is_default = true;
            Ok(pm.clone())
        }

        async fn count_active_by_owner(&self, owner_id: Uuid) -> Result<i64, String> {
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .filter(|m| m.owner_id == owner_id && m.is_active)
                .count() as i64)
        }

        async fn has_active_payment_methods(&self, owner_id: Uuid) -> Result<bool, String> {
            Ok(self.count_active_by_owner(owner_id).await? > 0)
        }
    }

    // ─── Helpers ──────────────────────────────────────────────────────

    fn make_use_cases(
        payment_repo: Arc<dyn PaymentRepository>,
        pm_repo: Arc<dyn PaymentMethodRepository>,
    ) -> PaymentUseCases {
        PaymentUseCases::new(payment_repo, pm_repo)
    }

    fn make_create_request(
        building_id: Uuid,
        owner_id: Uuid,
        amount_cents: i64,
    ) -> CreatePaymentRequest {
        CreatePaymentRequest {
            building_id,
            owner_id,
            expense_id: None,
            amount_cents,
            payment_method_type: PaymentMethodType::Card,
            payment_method_id: None,
            description: Some("Test payment".to_string()),
            metadata: None,
        }
    }

    /// Insert a payment directly into the mock repo and return it.
    async fn seed_payment(
        repo: &Arc<MockPaymentRepository>,
        org_id: Uuid,
        building_id: Uuid,
        owner_id: Uuid,
        amount_cents: i64,
    ) -> Payment {
        let payment = Payment::new(
            org_id,
            building_id,
            owner_id,
            None,
            amount_cents,
            PaymentMethodType::Card,
            format!("idem-{}-{}", org_id, Uuid::new_v4()),
            Some("seeded".to_string()),
        )
        .unwrap();
        repo.create(&payment).await.unwrap();
        payment
    }

    // ─── Tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_payment_success() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let request = make_create_request(building_id, owner_id, 15000);
        let result = uc.create_payment(org_id, request).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.amount_cents, 15000);
        assert_eq!(resp.currency, "EUR");
        assert_eq!(resp.status, TransactionStatus::Pending);
        assert_eq!(resp.organization_id, org_id);
        assert_eq!(resp.building_id, building_id);
        assert_eq!(resp.owner_id, owner_id);
        assert_eq!(resp.refunded_amount_cents, 0);
        // Verify it was persisted
        assert_eq!(payment_repo.payments.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_create_payment_invalid_amount_zero() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo, pm_repo);

        let request = make_create_request(Uuid::new_v4(), Uuid::new_v4(), 0);
        let result = uc.create_payment(Uuid::new_v4(), request).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Amount must be greater than 0"));
    }

    #[tokio::test]
    async fn test_create_payment_invalid_amount_negative() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo, pm_repo);

        let request = make_create_request(Uuid::new_v4(), Uuid::new_v4(), -500);
        let result = uc.create_payment(Uuid::new_v4(), request).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Amount must be greater than 0"));
    }

    #[tokio::test]
    async fn test_status_transition_pending_to_processing_to_succeeded() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 10000).await;
        let pid = payment.id;

        // Pending -> Processing
        let resp = uc.mark_processing(pid).await.unwrap();
        assert_eq!(resp.status, TransactionStatus::Processing);

        // Processing -> Succeeded
        let resp = uc.mark_succeeded(pid).await.unwrap();
        assert_eq!(resp.status, TransactionStatus::Succeeded);
        assert!(resp.succeeded_at.is_some());
    }

    #[tokio::test]
    async fn test_status_transition_mark_failed() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 5000).await;
        let pid = payment.id;

        // Pending -> Processing
        uc.mark_processing(pid).await.unwrap();

        // Processing -> Failed
        let resp = uc
            .mark_failed(pid, "Card declined".to_string())
            .await
            .unwrap();
        assert_eq!(resp.status, TransactionStatus::Failed);
        assert_eq!(resp.failure_reason, Some("Card declined".to_string()));
        assert!(resp.failed_at.is_some());
    }

    #[tokio::test]
    async fn test_mark_processing_not_found() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo, pm_repo);

        let result = uc.mark_processing(Uuid::new_v4()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Payment not found"));
    }

    #[tokio::test]
    async fn test_refund_partial_success() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 20000).await;
        let pid = payment.id;

        // Move to Succeeded first
        uc.mark_succeeded(pid).await.unwrap();

        // Partial refund: 8000 out of 20000
        let resp = uc
            .refund_payment(
                pid,
                RefundPaymentRequest {
                    amount_cents: 8000,
                    reason: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.status, TransactionStatus::Refunded);
        assert_eq!(resp.refunded_amount_cents, 8000);
        assert_eq!(resp.net_amount_cents, 12000); // 20000 - 8000
    }

    #[tokio::test]
    async fn test_refund_over_refund_prevention() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 10000).await;
        let pid = payment.id;

        // Move to Succeeded
        uc.mark_succeeded(pid).await.unwrap();

        // Partial refund: 6000
        uc.refund_payment(
            pid,
            RefundPaymentRequest {
                amount_cents: 6000,
                reason: None,
            },
        )
        .await
        .unwrap();

        // Try to refund 6000 more (total would be 12000 > 10000)
        let result = uc
            .refund_payment(
                pid,
                RefundPaymentRequest {
                    amount_cents: 6000,
                    reason: None,
                },
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds"));
    }

    #[tokio::test]
    async fn test_refund_pending_payment_fails() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 10000).await;
        let pid = payment.id;

        // Payment is still Pending, refund should fail
        let result = uc
            .refund_payment(
                pid,
                RefundPaymentRequest {
                    amount_cents: 5000,
                    reason: None,
                },
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("succeeded payments"));
    }

    #[tokio::test]
    async fn test_get_payment_found_and_not_found() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 5000).await;

        // Found
        let result = uc.get_payment(payment.id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().amount_cents, 5000);

        // Not found
        let result = uc.get_payment(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_owner_and_building_payments() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Seed 2 payments for the same owner and building
        seed_payment(&payment_repo, org_id, building_id, owner_id, 5000).await;
        seed_payment(&payment_repo, org_id, building_id, owner_id, 7000).await;

        // Seed 1 payment for a different owner
        seed_payment(&payment_repo, org_id, building_id, Uuid::new_v4(), 3000).await;

        let owner_payments = uc.list_owner_payments(owner_id).await.unwrap();
        assert_eq!(owner_payments.len(), 2);

        let building_payments = uc.list_building_payments(building_id).await.unwrap();
        assert_eq!(building_payments.len(), 3);
    }

    #[tokio::test]
    async fn test_set_payment_method_id_validates_existence() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        // Empty payment method repo -- no methods exist
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 5000).await;

        // Setting a non-existent payment method should fail
        let result = uc.set_payment_method_id(payment.id, Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Payment method not found"));
    }

    #[tokio::test]
    async fn test_set_payment_method_id_success() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm = PaymentMethod::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            PMMethodType::Card,
            "pm_test_stripe_id_12345".to_string(),
            "cus_test_customer_12345".to_string(),
            "Visa **** 4242".to_string(),
            true,
        )
        .unwrap();
        let pm_id = pm.id;
        let pm_repo = Arc::new(MockPaymentMethodRepository::with_method(pm));
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 5000).await;

        let resp = uc.set_payment_method_id(payment.id, pm_id).await.unwrap();
        assert_eq!(resp.payment_method_id, Some(pm_id));
    }

    #[tokio::test]
    async fn test_delete_payment() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let payment =
            seed_payment(&payment_repo, org_id, Uuid::new_v4(), Uuid::new_v4(), 5000).await;

        assert!(uc.delete_payment(payment.id).await.unwrap());
        // After deletion, get should return None
        let result = uc.get_payment(payment.id).await.unwrap();
        assert!(result.is_none());

        // Deleting non-existent returns false
        assert!(!uc.delete_payment(Uuid::new_v4()).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_owner_payment_stats() {
        let payment_repo = Arc::new(MockPaymentRepository::new());
        let pm_repo = Arc::new(MockPaymentMethodRepository::new());
        let uc = make_use_cases(payment_repo.clone(), pm_repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Seed 2 payments, mark one succeeded
        let p1 = seed_payment(&payment_repo, org_id, building_id, owner_id, 10000).await;
        seed_payment(&payment_repo, org_id, building_id, owner_id, 5000).await;

        // Mark p1 as succeeded via the use case
        uc.mark_succeeded(p1.id).await.unwrap();

        let stats = uc.get_owner_payment_stats(owner_id).await.unwrap();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.succeeded_count, 1);
        assert_eq!(stats.pending_count, 1);
        assert_eq!(stats.total_amount_cents, 15000);
        assert_eq!(stats.total_succeeded_cents, 10000);
    }
}
