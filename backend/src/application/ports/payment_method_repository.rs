use crate::domain::entities::payment_method::{PaymentMethod, PaymentMethodType};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PaymentMethodRepository: Send + Sync {
    /// Create a new payment method
    async fn create(&self, payment_method: &PaymentMethod) -> Result<PaymentMethod, String>;

    /// Find payment method by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentMethod>, String>;

    /// Find payment method by Stripe payment method ID
    async fn find_by_stripe_payment_method_id(
        &self,
        stripe_payment_method_id: &str,
    ) -> Result<Option<PaymentMethod>, String>;

    /// Find all payment methods for an owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentMethod>, String>;

    /// Find active payment methods for an owner
    async fn find_active_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentMethod>, String>;

    /// Find default payment method for an owner
    async fn find_default_by_owner(&self, owner_id: Uuid) -> Result<Option<PaymentMethod>, String>;

    /// Find all payment methods for an organization
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentMethod>, String>;

    /// Find payment methods by type
    async fn find_by_owner_and_type(
        &self,
        owner_id: Uuid,
        method_type: PaymentMethodType,
    ) -> Result<Vec<PaymentMethod>, String>;

    /// Update payment method
    async fn update(&self, payment_method: &PaymentMethod) -> Result<PaymentMethod, String>;

    /// Delete payment method (soft delete recommended)
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Set payment method as default (unsets other defaults for owner)
    async fn set_as_default(&self, id: Uuid, owner_id: Uuid) -> Result<PaymentMethod, String>;

    /// Count active payment methods for owner
    async fn count_active_by_owner(&self, owner_id: Uuid) -> Result<i64, String>;

    /// Check if owner has any active payment methods
    async fn has_active_payment_methods(&self, owner_id: Uuid) -> Result<bool, String>;
}
