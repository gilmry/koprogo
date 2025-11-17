use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment transaction status following Stripe webhook lifecycle
/// Note: This is different from expense::PaymentStatus which tracks expense payment state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// Payment intent created but not yet processed
    Pending,
    /// Payment is being processed by payment provider
    Processing,
    /// Payment requires additional action (e.g., 3D Secure)
    RequiresAction,
    /// Payment succeeded
    Succeeded,
    /// Payment failed (card declined, insufficient funds, etc.)
    Failed,
    /// Payment cancelled by user or system
    Cancelled,
    /// Payment was refunded (partial or full)
    Refunded,
}

/// Payment method type (extensible for future methods)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    /// Credit/debit card via Stripe
    Card,
    /// SEPA Direct Debit (Belgian bank transfer)
    SepaDebit,
    /// Manual bank transfer
    BankTransfer,
    /// Cash payment (recorded manually)
    Cash,
}

/// Payment entity - Represents a payment for an expense
///
/// Belgian property management context:
/// - Payments are always in EUR (Belgian currency)
/// - Linked to Expense entity (charge to co-owners)
/// - Supports Stripe (cards) and SEPA (bank transfers)
/// - Includes idempotency key for safe retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    /// Organization (multi-tenant isolation)
    pub organization_id: Uuid,
    /// Building this payment relates to
    pub building_id: Uuid,
    /// Owner making the payment
    pub owner_id: Uuid,
    /// Expense being paid (optional: could be general account credit)
    pub expense_id: Option<Uuid>,
    /// Payment amount in cents (EUR) - Stripe uses smallest currency unit
    pub amount_cents: i64,
    /// Currency (always EUR for Belgian context)
    pub currency: String,
    /// Payment transaction status
    pub status: TransactionStatus,
    /// Payment method type used
    pub payment_method_type: PaymentMethodType,
    /// Stripe payment intent ID (for card/SEPA payments)
    pub stripe_payment_intent_id: Option<String>,
    /// Stripe customer ID (for recurring customers)
    pub stripe_customer_id: Option<String>,
    /// Stored payment method ID (if saved for future use)
    pub payment_method_id: Option<Uuid>,
    /// Idempotency key for safe retries (prevents duplicate charges)
    pub idempotency_key: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional metadata (JSON) for extensibility
    pub metadata: Option<String>,
    /// Failure reason (if status = Failed)
    pub failure_reason: Option<String>,
    /// Refund amount in cents (if status = Refunded)
    pub refunded_amount_cents: i64,
    /// Date when payment succeeded (if status = Succeeded)
    pub succeeded_at: Option<DateTime<Utc>>,
    /// Date when payment failed (if status = Failed)
    pub failed_at: Option<DateTime<Utc>>,
    /// Date when payment was cancelled (if status = Cancelled)
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    /// Create a new payment intent
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID (multi-tenant)
    /// * `building_id` - Building ID
    /// * `owner_id` - Owner making the payment
    /// * `expense_id` - Optional expense being paid
    /// * `amount_cents` - Amount in cents (EUR)
    /// * `payment_method_type` - Payment method type
    /// * `idempotency_key` - Idempotency key for safe retries
    /// * `description` - Optional description
    ///
    /// # Returns
    /// * `Ok(Payment)` - New payment with status Pending
    /// * `Err(String)` - Validation error
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        owner_id: Uuid,
        expense_id: Option<Uuid>,
        amount_cents: i64,
        payment_method_type: PaymentMethodType,
        idempotency_key: String,
        description: Option<String>,
    ) -> Result<Self, String> {
        // Validate amount
        if amount_cents <= 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        // Validate idempotency key (min 16 chars for uniqueness)
        if idempotency_key.trim().is_empty() || idempotency_key.len() < 16 {
            return Err(
                "Idempotency key must be at least 16 characters for uniqueness".to_string(),
            );
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            owner_id,
            expense_id,
            amount_cents,
            currency: "EUR".to_string(), // Always EUR for Belgian context
            status: TransactionStatus::Pending,
            payment_method_type,
            stripe_payment_intent_id: None,
            stripe_customer_id: None,
            payment_method_id: None,
            idempotency_key,
            description,
            metadata: None,
            failure_reason: None,
            refunded_amount_cents: 0,
            succeeded_at: None,
            failed_at: None,
            cancelled_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Mark payment as processing
    pub fn mark_processing(&mut self) -> Result<(), String> {
        match self.status {
            TransactionStatus::Pending => {
                self.status = TransactionStatus::Processing;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as processing from status: {:?}",
                self.status
            )),
        }
    }

    /// Mark payment as requiring action (e.g., 3D Secure authentication)
    pub fn mark_requires_action(&mut self) -> Result<(), String> {
        match self.status {
            TransactionStatus::Pending | TransactionStatus::Processing => {
                self.status = TransactionStatus::RequiresAction;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as requires_action from status: {:?}",
                self.status
            )),
        }
    }

    /// Mark payment as succeeded
    pub fn mark_succeeded(&mut self) -> Result<(), String> {
        match self.status {
            TransactionStatus::Pending | TransactionStatus::Processing | TransactionStatus::RequiresAction => {
                self.status = TransactionStatus::Succeeded;
                self.succeeded_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as succeeded from status: {:?}",
                self.status
            )),
        }
    }

    /// Mark payment as failed
    pub fn mark_failed(&mut self, reason: String) -> Result<(), String> {
        match self.status {
            TransactionStatus::Pending
            | TransactionStatus::Processing
            | TransactionStatus::RequiresAction => {
                self.status = TransactionStatus::Failed;
                self.failure_reason = Some(reason);
                self.failed_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as failed from status: {:?}",
                self.status
            )),
        }
    }

    /// Mark payment as cancelled
    pub fn mark_cancelled(&mut self) -> Result<(), String> {
        match self.status {
            TransactionStatus::Pending | TransactionStatus::Processing | TransactionStatus::RequiresAction => {
                self.status = TransactionStatus::Cancelled;
                self.cancelled_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as cancelled from status: {:?}",
                self.status
            )),
        }
    }

    /// Refund payment (partial or full)
    pub fn refund(&mut self, refund_amount_cents: i64) -> Result<(), String> {
        // Can only refund succeeded payments
        if self.status != TransactionStatus::Succeeded {
            return Err(format!(
                "Can only refund succeeded payments, current status: {:?}",
                self.status
            ));
        }

        // Validate refund amount
        if refund_amount_cents <= 0 {
            return Err("Refund amount must be greater than 0".to_string());
        }

        // Check total refunds don't exceed original amount
        let total_refunded = self.refunded_amount_cents + refund_amount_cents;
        if total_refunded > self.amount_cents {
            return Err(format!(
                "Total refund ({} cents) would exceed original payment ({} cents)",
                total_refunded, self.amount_cents
            ));
        }

        self.refunded_amount_cents += refund_amount_cents;

        // If fully refunded, update status
        if self.refunded_amount_cents == self.amount_cents {
            self.status = TransactionStatus::Refunded;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set Stripe payment intent ID
    pub fn set_stripe_payment_intent_id(&mut self, payment_intent_id: String) {
        self.stripe_payment_intent_id = Some(payment_intent_id);
        self.updated_at = Utc::now();
    }

    /// Set Stripe customer ID
    pub fn set_stripe_customer_id(&mut self, customer_id: String) {
        self.stripe_customer_id = Some(customer_id);
        self.updated_at = Utc::now();
    }

    /// Set payment method ID (for saved payment methods)
    pub fn set_payment_method_id(&mut self, payment_method_id: Uuid) {
        self.payment_method_id = Some(payment_method_id);
        self.updated_at = Utc::now();
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: String) {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
    }

    /// Get net amount after refunds (in cents)
    pub fn get_net_amount_cents(&self) -> i64 {
        self.amount_cents - self.refunded_amount_cents
    }

    /// Check if payment is in final state (cannot be modified)
    pub fn is_final(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Succeeded
                | TransactionStatus::Failed
                | TransactionStatus::Cancelled
                | TransactionStatus::Refunded
        )
    }

    /// Check if payment can be refunded
    pub fn can_refund(&self) -> bool {
        self.status == TransactionStatus::Succeeded
            && self.refunded_amount_cents < self.amount_cents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_payment() -> Payment {
        Payment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            10000, // 100.00 EUR
            PaymentMethodType::Card,
            "test_idempotency_key_123456789".to_string(),
            Some("Test payment".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_create_payment_success() {
        let payment = create_test_payment();
        assert_eq!(payment.amount_cents, 10000);
        assert_eq!(payment.currency, "EUR");
        assert_eq!(payment.status, TransactionStatus::Pending);
        assert_eq!(payment.refunded_amount_cents, 0);
    }

    #[test]
    fn test_create_payment_invalid_amount() {
        let result = Payment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            0, // Invalid: must be > 0
            PaymentMethodType::Card,
            "test_idempotency_key_123456789".to_string(),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be greater than 0");
    }

    #[test]
    fn test_create_payment_invalid_idempotency_key() {
        let result = Payment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            10000,
            PaymentMethodType::Card,
            "short".to_string(), // Too short
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Idempotency key"));
    }

    #[test]
    fn test_payment_lifecycle_success() {
        let mut payment = create_test_payment();

        // Pending → Processing
        assert!(payment.mark_processing().is_ok());
        assert_eq!(payment.status, TransactionStatus::Processing);

        // Processing → Succeeded
        assert!(payment.mark_succeeded().is_ok());
        assert_eq!(payment.status, TransactionStatus::Succeeded);
        assert!(payment.succeeded_at.is_some());
        assert!(payment.is_final());
    }

    #[test]
    fn test_payment_lifecycle_failure() {
        let mut payment = create_test_payment();

        payment.mark_processing().unwrap();

        // Processing → Failed
        assert!(payment.mark_failed("Card declined".to_string()).is_ok());
        assert_eq!(payment.status, TransactionStatus::Failed);
        assert_eq!(payment.failure_reason, Some("Card declined".to_string()));
        assert!(payment.failed_at.is_some());
        assert!(payment.is_final());
    }

    #[test]
    fn test_payment_lifecycle_cancelled() {
        let mut payment = create_test_payment();

        // Pending → Cancelled
        assert!(payment.mark_cancelled().is_ok());
        assert_eq!(payment.status, TransactionStatus::Cancelled);
        assert!(payment.cancelled_at.is_some());
        assert!(payment.is_final());
    }

    #[test]
    fn test_payment_requires_action() {
        let mut payment = create_test_payment();

        payment.mark_processing().unwrap();

        // Processing → RequiresAction (e.g., 3D Secure)
        assert!(payment.mark_requires_action().is_ok());
        assert_eq!(payment.status, TransactionStatus::RequiresAction);

        // RequiresAction → Succeeded (after user completes 3DS)
        assert!(payment.mark_succeeded().is_ok());
        assert_eq!(payment.status, TransactionStatus::Succeeded);
    }

    #[test]
    fn test_payment_invalid_status_transition() {
        let mut payment = create_test_payment();
        payment.mark_succeeded().unwrap();

        // Cannot go from Succeeded to Processing
        assert!(payment.mark_processing().is_err());
    }

    #[test]
    fn test_refund_full() {
        let mut payment = create_test_payment();
        payment.mark_succeeded().unwrap();

        assert!(payment.can_refund());

        // Full refund
        assert!(payment.refund(10000).is_ok());
        assert_eq!(payment.refunded_amount_cents, 10000);
        assert_eq!(payment.status, TransactionStatus::Refunded);
        assert_eq!(payment.get_net_amount_cents(), 0);
        assert!(!payment.can_refund());
    }

    #[test]
    fn test_refund_partial() {
        let mut payment = create_test_payment();
        payment.mark_succeeded().unwrap();

        // Partial refund (50%)
        assert!(payment.refund(5000).is_ok());
        assert_eq!(payment.refunded_amount_cents, 5000);
        assert_eq!(payment.status, TransactionStatus::Succeeded); // Still succeeded
        assert_eq!(payment.get_net_amount_cents(), 5000);
        assert!(payment.can_refund());

        // Refund remaining 50%
        assert!(payment.refund(5000).is_ok());
        assert_eq!(payment.refunded_amount_cents, 10000);
        assert_eq!(payment.status, TransactionStatus::Refunded);
    }

    #[test]
    fn test_refund_exceeds_amount() {
        let mut payment = create_test_payment();
        payment.mark_succeeded().unwrap();

        // Try to refund more than original amount
        let result = payment.refund(15000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceed original payment"));
    }

    #[test]
    fn test_refund_before_success() {
        let mut payment = create_test_payment();

        // Cannot refund pending payment
        let result = payment.refund(5000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("succeeded payments"));
    }

    #[test]
    fn test_set_stripe_data() {
        let mut payment = create_test_payment();

        payment.set_stripe_payment_intent_id("pi_123456789".to_string());
        assert_eq!(
            payment.stripe_payment_intent_id,
            Some("pi_123456789".to_string())
        );

        payment.set_stripe_customer_id("cus_123456789".to_string());
        assert_eq!(
            payment.stripe_customer_id,
            Some("cus_123456789".to_string())
        );

        let method_id = Uuid::new_v4();
        payment.set_payment_method_id(method_id);
        assert_eq!(payment.payment_method_id, Some(method_id));
    }
}
