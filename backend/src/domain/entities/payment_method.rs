use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment method entity - Represents a stored payment method
///
/// Belgian property management context:
/// - Store payment methods for recurring charges
/// - Support cards (Stripe) and SEPA mandates (Belgian bank accounts)
/// - PCI-DSS compliant: Never store raw card data, only Stripe tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: Uuid,
    /// Organization (multi-tenant isolation)
    pub organization_id: Uuid,
    /// Owner who owns this payment method
    pub owner_id: Uuid,
    /// Payment method type
    pub method_type: PaymentMethodType,
    /// Stripe payment method ID (pm_xxx for cards, sepa_debit_xxx for SEPA)
    pub stripe_payment_method_id: String,
    /// Stripe customer ID (links payment method to customer)
    pub stripe_customer_id: String,
    /// Display label for UI (e.g., "Visa •••• 4242", "SEPA BE68 5390 0754")
    pub display_label: String,
    /// Is this the default payment method for the owner?
    pub is_default: bool,
    /// Is this payment method active? (can be deactivated)
    pub is_active: bool,
    /// Card/SEPA specific metadata (JSON) - stores last4, brand, expiry, etc.
    pub metadata: Option<String>,
    /// Expiry date for cards (not applicable for SEPA)
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment method type (aligned with Payment entity)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    /// Credit/debit card via Stripe
    Card,
    /// SEPA Direct Debit (Belgian bank transfer)
    SepaDebit,
}

impl PaymentMethod {
    /// Create a new payment method
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID (multi-tenant)
    /// * `owner_id` - Owner who owns this payment method
    /// * `method_type` - Payment method type (Card or SepaDebit)
    /// * `stripe_payment_method_id` - Stripe payment method ID
    /// * `stripe_customer_id` - Stripe customer ID
    /// * `display_label` - Display label for UI
    /// * `is_default` - Is this the default payment method?
    ///
    /// # Returns
    /// * `Ok(PaymentMethod)` - New payment method
    /// * `Err(String)` - Validation error
    pub fn new(
        organization_id: Uuid,
        owner_id: Uuid,
        method_type: PaymentMethodType,
        stripe_payment_method_id: String,
        stripe_customer_id: String,
        display_label: String,
        is_default: bool,
    ) -> Result<Self, String> {
        // Validate Stripe IDs
        if stripe_payment_method_id.trim().is_empty() {
            return Err("Stripe payment method ID cannot be empty".to_string());
        }
        if stripe_customer_id.trim().is_empty() {
            return Err("Stripe customer ID cannot be empty".to_string());
        }

        // Validate display label
        if display_label.trim().is_empty() {
            return Err("Display label cannot be empty".to_string());
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            owner_id,
            method_type,
            stripe_payment_method_id,
            stripe_customer_id,
            display_label,
            is_default,
            is_active: true, // Active by default
            metadata: None,
            expires_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Set as default payment method
    pub fn set_default(&mut self) {
        self.is_default = true;
        self.updated_at = Utc::now();
    }

    /// Unset as default payment method
    pub fn unset_default(&mut self) {
        self.is_default = false;
        self.updated_at = Utc::now();
    }

    /// Deactivate payment method (soft delete)
    pub fn deactivate(&mut self) -> Result<(), String> {
        if !self.is_active {
            return Err("Payment method is already inactive".to_string());
        }

        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Reactivate payment method
    pub fn reactivate(&mut self) -> Result<(), String> {
        if self.is_active {
            return Err("Payment method is already active".to_string());
        }

        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set metadata (JSON)
    pub fn set_metadata(&mut self, metadata: String) {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
    }

    /// Set expiry date (for cards only)
    pub fn set_expiry(&mut self, expires_at: DateTime<Utc>) -> Result<(), String> {
        if self.method_type != PaymentMethodType::Card {
            return Err("Only cards have expiry dates".to_string());
        }

        self.expires_at = Some(expires_at);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if payment method is expired (cards only)
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    /// Check if payment method is usable (active and not expired)
    pub fn is_usable(&self) -> bool {
        self.is_active && !self.is_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_card() -> PaymentMethod {
        PaymentMethod::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            PaymentMethodType::Card,
            "pm_test_card_123456789".to_string(),
            "cus_test_123456789".to_string(),
            "Visa •••• 4242".to_string(),
            true,
        )
        .unwrap()
    }

    fn create_test_sepa() -> PaymentMethod {
        PaymentMethod::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            PaymentMethodType::SepaDebit,
            "sepa_debit_test_123456789".to_string(),
            "cus_test_123456789".to_string(),
            "SEPA BE68 5390 0754".to_string(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn test_create_card_success() {
        let card = create_test_card();
        assert_eq!(card.method_type, PaymentMethodType::Card);
        assert_eq!(card.display_label, "Visa •••• 4242");
        assert!(card.is_default);
        assert!(card.is_active);
        assert!(card.is_usable());
    }

    #[test]
    fn test_create_sepa_success() {
        let sepa = create_test_sepa();
        assert_eq!(sepa.method_type, PaymentMethodType::SepaDebit);
        assert_eq!(sepa.display_label, "SEPA BE68 5390 0754");
        assert!(!sepa.is_default);
        assert!(sepa.is_active);
        assert!(sepa.is_usable());
    }

    #[test]
    fn test_create_invalid_stripe_id() {
        let result = PaymentMethod::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            PaymentMethodType::Card,
            "".to_string(), // Empty Stripe ID
            "cus_123".to_string(),
            "Visa 4242".to_string(),
            false,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("payment method ID"));
    }

    #[test]
    fn test_create_invalid_display_label() {
        let result = PaymentMethod::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            PaymentMethodType::Card,
            "pm_123".to_string(),
            "cus_123".to_string(),
            "".to_string(), // Empty label
            false,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Display label"));
    }

    #[test]
    fn test_set_unset_default() {
        let mut card = create_test_card();
        assert!(card.is_default);

        card.unset_default();
        assert!(!card.is_default);

        card.set_default();
        assert!(card.is_default);
    }

    #[test]
    fn test_deactivate_reactivate() {
        let mut card = create_test_card();
        assert!(card.is_active);
        assert!(card.is_usable());

        // Deactivate
        assert!(card.deactivate().is_ok());
        assert!(!card.is_active);
        assert!(!card.is_usable());

        // Try deactivating again (should fail)
        assert!(card.deactivate().is_err());

        // Reactivate
        assert!(card.reactivate().is_ok());
        assert!(card.is_active);
        assert!(card.is_usable());
    }

    #[test]
    fn test_card_expiry() {
        let mut card = create_test_card();
        assert!(!card.is_expired());

        // Set expiry in the past
        let past = Utc::now() - chrono::Duration::days(30);
        assert!(card.set_expiry(past).is_ok());
        assert!(card.is_expired());
        assert!(!card.is_usable()); // Not usable because expired

        // Set expiry in the future
        let future = Utc::now() + chrono::Duration::days(365);
        assert!(card.set_expiry(future).is_ok());
        assert!(!card.is_expired());
        assert!(card.is_usable());
    }

    #[test]
    fn test_sepa_no_expiry() {
        let mut sepa = create_test_sepa();

        // SEPA should not have expiry
        let future = Utc::now() + chrono::Duration::days(365);
        let result = sepa.set_expiry(future);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only cards"));
    }

    #[test]
    fn test_set_metadata() {
        let mut card = create_test_card();
        assert!(card.metadata.is_none());

        let metadata =
            r#"{"brand": "visa", "last4": "4242", "exp_month": 12, "exp_year": 2025}"#.to_string();
        card.set_metadata(metadata.clone());
        assert_eq!(card.metadata, Some(metadata));
    }
}
