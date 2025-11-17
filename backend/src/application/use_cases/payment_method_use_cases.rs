use crate::application::dto::{
    CreatePaymentMethodRequest, PaymentMethodResponse, UpdatePaymentMethodRequest,
};
use crate::application::ports::PaymentMethodRepository;
use crate::domain::entities::payment_method::{PaymentMethod, PaymentMethodType};
use std::sync::Arc;
use uuid::Uuid;

pub struct PaymentMethodUseCases {
    payment_method_repository: Arc<dyn PaymentMethodRepository>,
}

impl PaymentMethodUseCases {
    pub fn new(payment_method_repository: Arc<dyn PaymentMethodRepository>) -> Self {
        Self {
            payment_method_repository,
        }
    }

    /// Create a new payment method
    ///
    /// If is_default is true, automatically unsets other default payment methods for the owner.
    pub async fn create_payment_method(
        &self,
        organization_id: Uuid,
        request: CreatePaymentMethodRequest,
    ) -> Result<PaymentMethodResponse, String> {
        let payment_method = PaymentMethod::new(
            organization_id,
            request.owner_id,
            request.method_type,
            request.stripe_payment_method_id,
            request.stripe_customer_id,
            request.display_label,
            request.is_default,
        )?;

        // Set metadata if provided
        let mut payment_method = payment_method;
        if let Some(metadata) = request.metadata {
            payment_method.set_metadata(metadata);
        }

        // Set expiry if provided (cards only)
        if let Some(expires_at) = request.expires_at {
            payment_method.set_expiry(expires_at)?;
        }

        let created = self.payment_method_repository.create(&payment_method).await?;

        // If set as default, ensure it's the only default for this owner
        if created.is_default {
            let _ = self
                .payment_method_repository
                .set_as_default(created.id, created.owner_id)
                .await?;
        }

        Ok(PaymentMethodResponse::from(created))
    }

    /// Get payment method by ID
    pub async fn get_payment_method(
        &self,
        id: Uuid,
    ) -> Result<Option<PaymentMethodResponse>, String> {
        match self.payment_method_repository.find_by_id(id).await? {
            Some(method) => Ok(Some(PaymentMethodResponse::from(method))),
            None => Ok(None),
        }
    }

    /// Get payment method by Stripe payment method ID
    pub async fn get_payment_method_by_stripe_id(
        &self,
        stripe_payment_method_id: &str,
    ) -> Result<Option<PaymentMethodResponse>, String> {
        match self
            .payment_method_repository
            .find_by_stripe_payment_method_id(stripe_payment_method_id)
            .await?
        {
            Some(method) => Ok(Some(PaymentMethodResponse::from(method))),
            None => Ok(None),
        }
    }

    /// List payment methods for an owner
    pub async fn list_owner_payment_methods(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentMethodResponse>, String> {
        let methods = self.payment_method_repository.find_by_owner(owner_id).await?;
        Ok(methods
            .into_iter()
            .map(PaymentMethodResponse::from)
            .collect())
    }

    /// List active payment methods for an owner
    pub async fn list_active_owner_payment_methods(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentMethodResponse>, String> {
        let methods = self
            .payment_method_repository
            .find_active_by_owner(owner_id)
            .await?;
        Ok(methods
            .into_iter()
            .map(PaymentMethodResponse::from)
            .collect())
    }

    /// Get default payment method for an owner
    pub async fn get_default_payment_method(
        &self,
        owner_id: Uuid,
    ) -> Result<Option<PaymentMethodResponse>, String> {
        match self
            .payment_method_repository
            .find_default_by_owner(owner_id)
            .await?
        {
            Some(method) => Ok(Some(PaymentMethodResponse::from(method))),
            None => Ok(None),
        }
    }

    /// List payment methods for an organization
    pub async fn list_organization_payment_methods(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentMethodResponse>, String> {
        let methods = self
            .payment_method_repository
            .find_by_organization(organization_id)
            .await?;
        Ok(methods
            .into_iter()
            .map(PaymentMethodResponse::from)
            .collect())
    }

    /// List payment methods by owner and type
    pub async fn list_payment_methods_by_type(
        &self,
        owner_id: Uuid,
        method_type: PaymentMethodType,
    ) -> Result<Vec<PaymentMethodResponse>, String> {
        let methods = self
            .payment_method_repository
            .find_by_owner_and_type(owner_id, method_type)
            .await?;
        Ok(methods
            .into_iter()
            .map(PaymentMethodResponse::from)
            .collect())
    }

    /// Update payment method
    pub async fn update_payment_method(
        &self,
        id: Uuid,
        request: UpdatePaymentMethodRequest,
    ) -> Result<PaymentMethodResponse, String> {
        let mut payment_method = self
            .payment_method_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment method not found".to_string())?;

        // Update display label if provided
        if let Some(display_label) = request.display_label {
            if display_label.trim().is_empty() {
                return Err("Display label cannot be empty".to_string());
            }
            payment_method.display_label = display_label;
            payment_method.updated_at = chrono::Utc::now();
        }

        // Update metadata if provided
        if let Some(metadata) = request.metadata {
            payment_method.set_metadata(metadata);
        }

        // Update default status if provided
        if let Some(is_default) = request.is_default {
            if is_default && !payment_method.is_default {
                // Set as default (will unset other defaults)
                return Ok(PaymentMethodResponse::from(
                    self.payment_method_repository
                        .set_as_default(id, payment_method.owner_id)
                        .await?,
                ));
            } else if !is_default && payment_method.is_default {
                // Unset default
                payment_method.unset_default();
            }
        }

        let updated = self.payment_method_repository.update(&payment_method).await?;
        Ok(PaymentMethodResponse::from(updated))
    }

    /// Set payment method as default
    pub async fn set_as_default(
        &self,
        id: Uuid,
        owner_id: Uuid,
    ) -> Result<PaymentMethodResponse, String> {
        let payment_method = self
            .payment_method_repository
            .set_as_default(id, owner_id)
            .await?;
        Ok(PaymentMethodResponse::from(payment_method))
    }

    /// Deactivate payment method
    pub async fn deactivate_payment_method(
        &self,
        id: Uuid,
    ) -> Result<PaymentMethodResponse, String> {
        let mut payment_method = self
            .payment_method_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment method not found".to_string())?;

        payment_method.deactivate()?;

        let updated = self.payment_method_repository.update(&payment_method).await?;
        Ok(PaymentMethodResponse::from(updated))
    }

    /// Reactivate payment method
    pub async fn reactivate_payment_method(
        &self,
        id: Uuid,
    ) -> Result<PaymentMethodResponse, String> {
        let mut payment_method = self
            .payment_method_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment method not found".to_string())?;

        payment_method.reactivate()?;

        let updated = self.payment_method_repository.update(&payment_method).await?;
        Ok(PaymentMethodResponse::from(updated))
    }

    /// Delete payment method
    pub async fn delete_payment_method(&self, id: Uuid) -> Result<bool, String> {
        self.payment_method_repository.delete(id).await
    }

    /// Count active payment methods for owner
    pub async fn count_active_payment_methods(&self, owner_id: Uuid) -> Result<i64, String> {
        self.payment_method_repository
            .count_active_by_owner(owner_id)
            .await
    }

    /// Check if owner has any active payment methods
    pub async fn has_active_payment_methods(&self, owner_id: Uuid) -> Result<bool, String> {
        self.payment_method_repository
            .has_active_payment_methods(owner_id)
            .await
    }
}
