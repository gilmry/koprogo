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

        let created = self
            .payment_method_repository
            .create(&payment_method)
            .await?;

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
        let methods = self
            .payment_method_repository
            .find_by_owner(owner_id)
            .await?;
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

        let updated = self
            .payment_method_repository
            .update(&payment_method)
            .await?;
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

        let updated = self
            .payment_method_repository
            .update(&payment_method)
            .await?;
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

        let updated = self
            .payment_method_repository
            .update(&payment_method)
            .await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // --- Mock PaymentMethodRepository ---

    struct MockPaymentMethodRepository {
        methods: Mutex<HashMap<Uuid, PaymentMethod>>,
    }

    impl MockPaymentMethodRepository {
        fn new() -> Self {
            Self {
                methods: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl PaymentMethodRepository for MockPaymentMethodRepository {
        async fn create(&self, payment_method: &PaymentMethod) -> Result<PaymentMethod, String> {
            self.methods
                .lock()
                .unwrap()
                .insert(payment_method.id, payment_method.clone());
            Ok(payment_method.clone())
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
            method_type: PaymentMethodType,
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

        async fn update(&self, payment_method: &PaymentMethod) -> Result<PaymentMethod, String> {
            self.methods
                .lock()
                .unwrap()
                .insert(payment_method.id, payment_method.clone());
            Ok(payment_method.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.methods.lock().unwrap().remove(&id).is_some())
        }

        async fn set_as_default(&self, id: Uuid, owner_id: Uuid) -> Result<PaymentMethod, String> {
            let mut store = self.methods.lock().unwrap();
            // Unset all other defaults for this owner
            for method in store.values_mut() {
                if method.owner_id == owner_id && method.id != id {
                    method.is_default = false;
                    method.updated_at = chrono::Utc::now();
                }
            }
            // Set the target method as default
            if let Some(method) = store.get_mut(&id) {
                method.is_default = true;
                method.updated_at = chrono::Utc::now();
                Ok(method.clone())
            } else {
                Err("Payment method not found".to_string())
            }
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
            Ok(self
                .methods
                .lock()
                .unwrap()
                .values()
                .any(|m| m.owner_id == owner_id && m.is_active))
        }
    }

    // --- Helper functions ---

    fn make_use_cases(repo: Arc<MockPaymentMethodRepository>) -> PaymentMethodUseCases {
        PaymentMethodUseCases::new(repo)
    }

    fn make_create_request(
        owner_id: Uuid,
        method_type: PaymentMethodType,
        label: &str,
        is_default: bool,
    ) -> CreatePaymentMethodRequest {
        CreatePaymentMethodRequest {
            owner_id,
            method_type,
            stripe_payment_method_id: format!("pm_test_{}", Uuid::new_v4()),
            stripe_customer_id: format!("cus_test_{}", Uuid::new_v4()),
            display_label: label.to_string(),
            is_default,
            metadata: None,
            expires_at: None,
        }
    }

    // --- Tests ---

    #[tokio::test]
    async fn test_create_payment_method_success() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let request =
            make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 4242", false);

        let result = use_cases.create_payment_method(org_id, request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.owner_id, owner_id);
        assert_eq!(response.organization_id, org_id);
        assert_eq!(response.display_label, "Visa **** 4242");
        assert!(!response.is_default);
        assert!(response.is_active);

        // Verify persisted in repository
        assert_eq!(repo.methods.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_create_payment_method_with_default_flag() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let request = make_create_request(
            owner_id,
            PaymentMethodType::SepaDebit,
            "SEPA BE68 5390 0754",
            true,
        );

        let result = use_cases.create_payment_method(org_id, request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.is_default);
        assert_eq!(response.display_label, "SEPA BE68 5390 0754");
    }

    #[tokio::test]
    async fn test_set_as_default_unsets_previous_default() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Create first method as default
        let req1 = make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 1111", true);
        let method1 = use_cases.create_payment_method(org_id, req1).await.unwrap();
        assert!(method1.is_default);

        // Create second method (not default)
        let req2 = make_create_request(
            owner_id,
            PaymentMethodType::SepaDebit,
            "SEPA BE68 1234",
            false,
        );
        let method2 = use_cases.create_payment_method(org_id, req2).await.unwrap();
        assert!(!method2.is_default);

        // Set second method as default
        let result = use_cases.set_as_default(method2.id, owner_id).await;
        assert!(result.is_ok());
        let updated_method2 = result.unwrap();
        assert!(updated_method2.is_default);

        // Verify first method is no longer default
        let first = use_cases
            .get_payment_method(method1.id)
            .await
            .unwrap()
            .unwrap();
        assert!(!first.is_default);
    }

    #[tokio::test]
    async fn test_deactivate_payment_method_success() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let request =
            make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 4242", false);
        let created = use_cases
            .create_payment_method(org_id, request)
            .await
            .unwrap();
        assert!(created.is_active);

        let result = use_cases.deactivate_payment_method(created.id).await;
        assert!(result.is_ok());

        let deactivated = result.unwrap();
        assert!(!deactivated.is_active);
        assert!(!deactivated.is_usable);
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive_fails() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let request =
            make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 4242", false);
        let created = use_cases
            .create_payment_method(org_id, request)
            .await
            .unwrap();

        // Deactivate once (success)
        use_cases
            .deactivate_payment_method(created.id)
            .await
            .unwrap();

        // Deactivate again (should fail)
        let result = use_cases.deactivate_payment_method(created.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already inactive"));
    }

    #[tokio::test]
    async fn test_reactivate_payment_method_success() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let request =
            make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 4242", false);
        let created = use_cases
            .create_payment_method(org_id, request)
            .await
            .unwrap();

        // Deactivate first
        use_cases
            .deactivate_payment_method(created.id)
            .await
            .unwrap();

        // Reactivate
        let result = use_cases.reactivate_payment_method(created.id).await;
        assert!(result.is_ok());

        let reactivated = result.unwrap();
        assert!(reactivated.is_active);
        assert!(reactivated.is_usable);
    }

    #[tokio::test]
    async fn test_list_owner_payment_methods() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let other_owner_id = Uuid::new_v4();

        // Create 2 methods for owner_id
        let req1 = make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 1111", true);
        let req2 = make_create_request(
            owner_id,
            PaymentMethodType::SepaDebit,
            "SEPA BE68 1234",
            false,
        );
        // Create 1 method for another owner (should not appear)
        let req3 = make_create_request(
            other_owner_id,
            PaymentMethodType::Card,
            "MC **** 5555",
            false,
        );

        use_cases.create_payment_method(org_id, req1).await.unwrap();
        use_cases.create_payment_method(org_id, req2).await.unwrap();
        use_cases.create_payment_method(org_id, req3).await.unwrap();

        let result = use_cases.list_owner_payment_methods(owner_id).await;
        assert!(result.is_ok());
        let methods = result.unwrap();
        assert_eq!(methods.len(), 2);
        assert!(methods.iter().all(|m| m.owner_id == owner_id));
    }

    #[tokio::test]
    async fn test_deactivate_nonexistent_method_fails() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let fake_id = Uuid::new_v4();

        let result = use_cases.deactivate_payment_method(fake_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_reactivate_nonexistent_method_fails() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let fake_id = Uuid::new_v4();

        let result = use_cases.reactivate_payment_method(fake_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_count_and_has_active_payment_methods() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Initially no methods
        let count = use_cases
            .count_active_payment_methods(owner_id)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let has = use_cases
            .has_active_payment_methods(owner_id)
            .await
            .unwrap();
        assert!(!has);

        // Create two methods
        let req1 = make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 1111", false);
        let req2 = make_create_request(
            owner_id,
            PaymentMethodType::SepaDebit,
            "SEPA BE68 9999",
            false,
        );
        let m1 = use_cases.create_payment_method(org_id, req1).await.unwrap();
        use_cases.create_payment_method(org_id, req2).await.unwrap();

        let count = use_cases
            .count_active_payment_methods(owner_id)
            .await
            .unwrap();
        assert_eq!(count, 2);
        let has = use_cases
            .has_active_payment_methods(owner_id)
            .await
            .unwrap();
        assert!(has);

        // Deactivate one, count should decrease
        use_cases.deactivate_payment_method(m1.id).await.unwrap();
        let count = use_cases
            .count_active_payment_methods(owner_id)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_default_payment_method() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // No default initially
        let result = use_cases
            .get_default_payment_method(owner_id)
            .await
            .unwrap();
        assert!(result.is_none());

        // Create a default method
        let req = make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 4242", true);
        let created = use_cases.create_payment_method(org_id, req).await.unwrap();

        let result = use_cases
            .get_default_payment_method(owner_id)
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_delete_payment_method() {
        let repo = Arc::new(MockPaymentMethodRepository::new());
        let use_cases = make_use_cases(repo.clone());
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let req = make_create_request(owner_id, PaymentMethodType::Card, "Visa **** 4242", false);
        let created = use_cases.create_payment_method(org_id, req).await.unwrap();

        // Delete
        let deleted = use_cases.delete_payment_method(created.id).await.unwrap();
        assert!(deleted);

        // Verify gone
        let found = use_cases.get_payment_method(created.id).await.unwrap();
        assert!(found.is_none());

        // Delete again returns false
        let deleted_again = use_cases.delete_payment_method(created.id).await.unwrap();
        assert!(!deleted_again);
    }
}
