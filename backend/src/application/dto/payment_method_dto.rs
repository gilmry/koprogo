use crate::domain::entities::payment_method::{PaymentMethod, PaymentMethodType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment method response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub method_type: PaymentMethodType,
    pub stripe_payment_method_id: String,
    pub stripe_customer_id: String,
    pub display_label: String,
    pub is_default: bool,
    pub is_active: bool,
    pub metadata: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_expired: bool, // Computed from expires_at
    pub is_usable: bool,  // Computed: is_active && !is_expired
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PaymentMethod> for PaymentMethodResponse {
    fn from(method: PaymentMethod) -> Self {
        let is_expired = method.is_expired();
        let is_usable = method.is_usable();

        Self {
            id: method.id,
            organization_id: method.organization_id,
            owner_id: method.owner_id,
            method_type: method.method_type,
            stripe_payment_method_id: method.stripe_payment_method_id,
            stripe_customer_id: method.stripe_customer_id,
            display_label: method.display_label,
            is_default: method.is_default,
            is_active: method.is_active,
            metadata: method.metadata,
            expires_at: method.expires_at,
            is_expired,
            is_usable,
            created_at: method.created_at,
            updated_at: method.updated_at,
        }
    }
}

/// Create payment method request DTO (from Stripe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentMethodRequest {
    pub owner_id: Uuid,
    pub method_type: PaymentMethodType,
    pub stripe_payment_method_id: String,
    pub stripe_customer_id: String,
    pub display_label: String,
    pub is_default: bool,
    pub metadata: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Update payment method request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentMethodRequest {
    pub display_label: Option<String>,
    pub is_default: Option<bool>,
    pub metadata: Option<String>,
}
