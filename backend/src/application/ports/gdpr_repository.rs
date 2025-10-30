use crate::domain::entities::gdpr_export::GdprExport;
use async_trait::async_trait;
use uuid::Uuid;

/// GDPR Repository port for data export and anonymization operations
/// Implements GDPR Article 15 (Right to Access) and Article 17 (Right to Erasure)
#[async_trait]
pub trait GdprRepository: Send + Sync {
    /// Aggregate all personal data for a user (GDPR Article 15)
    ///
    /// Collects data from:
    /// - Users table
    /// - Owners table
    /// - Unit ownership relationships
    /// - Expenses
    /// - Documents
    /// - Meetings attendance
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user requesting data export
    /// * `organization_id` - Optional organization scope (None for SuperAdmin)
    ///
    /// # Returns
    /// * `Ok(GdprExport)` - Complete data export
    /// * `Err(String)` - If user not found or database error
    async fn aggregate_user_data(
        &self,
        user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<GdprExport, String>;

    /// Anonymize user account (GDPR Article 17)
    ///
    /// Replaces personal identifiable information with anonymized placeholders:
    /// - email → anonymized-{uuid}@deleted.local
    /// - first_name → "Anonymized"
    /// - last_name → "User"
    /// - Sets is_anonymized = true
    /// - Sets anonymized_at = NOW()
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user to anonymize
    ///
    /// # Returns
    /// * `Ok(())` - Anonymization successful
    /// * `Err(String)` - If user not found, already anonymized, or database error
    async fn anonymize_user(&self, user_id: Uuid) -> Result<(), String>;

    /// Anonymize owner profile (GDPR Article 17)
    ///
    /// Replaces personal identifiable information:
    /// - email → None
    /// - phone → None
    /// - address, city, postal_code, country → None
    /// - first_name → "Anonymized"
    /// - last_name → "User"
    /// - Sets is_anonymized = true
    /// - Sets anonymized_at = NOW()
    ///
    /// # Arguments
    /// * `owner_id` - UUID of the owner to anonymize
    ///
    /// # Returns
    /// * `Ok(())` - Anonymization successful
    /// * `Err(String)` - If owner not found, already anonymized, or database error
    async fn anonymize_owner(&self, owner_id: Uuid) -> Result<(), String>;

    /// Find all owner IDs linked to a user
    ///
    /// Used to identify which owner profiles need anonymization when a user requests erasure.
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `organization_id` - Optional organization scope
    ///
    /// # Returns
    /// * `Ok(Vec<Uuid>)` - List of owner UUIDs
    /// * `Err(String)` - Database error
    async fn find_owner_ids_by_user(
        &self,
        user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<Uuid>, String>;

    /// Check if user has legal holds preventing deletion
    ///
    /// Verifies if user has outstanding financial obligations or legal requirements
    /// that prevent complete anonymization (e.g., unpaid expenses, ongoing legal proceedings).
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of hold reasons (empty if no holds)
    /// * `Err(String)` - Database error
    async fn check_legal_holds(&self, user_id: Uuid) -> Result<Vec<String>, String>;

    /// Check if user is already anonymized
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    ///
    /// # Returns
    /// * `Ok(true)` - User is anonymized
    /// * `Ok(false)` - User is not anonymized
    /// * `Err(String)` - User not found or database error
    async fn is_user_anonymized(&self, user_id: Uuid) -> Result<bool, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    // Mock implementation for testing
    mock! {
        pub GdprRepo {}

        #[async_trait]
        impl GdprRepository for GdprRepo {
            async fn aggregate_user_data(
                &self,
                user_id: Uuid,
                organization_id: Option<Uuid>,
            ) -> Result<GdprExport, String>;

            async fn anonymize_user(&self, user_id: Uuid) -> Result<(), String>;

            async fn anonymize_owner(&self, owner_id: Uuid) -> Result<(), String>;

            async fn find_owner_ids_by_user(
                &self,
                user_id: Uuid,
                organization_id: Option<Uuid>,
            ) -> Result<Vec<Uuid>, String>;

            async fn check_legal_holds(&self, user_id: Uuid) -> Result<Vec<String>, String>;

            async fn is_user_anonymized(&self, user_id: Uuid) -> Result<bool, String>;
        }
    }

    #[tokio::test]
    async fn test_mock_gdpr_repository() {
        let mut mock_repo = MockGdprRepo::new();

        // Test aggregate_user_data mock
        let user_id = Uuid::new_v4();
        mock_repo
            .expect_aggregate_user_data()
            .times(1)
            .returning(|_, _| Err("Not implemented".to_string()));

        let result = mock_repo.aggregate_user_data(user_id, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_anonymize_user() {
        let mut mock_repo = MockGdprRepo::new();

        let user_id = Uuid::new_v4();
        mock_repo
            .expect_anonymize_user()
            .times(1)
            .returning(|_| Ok(()));

        let result = mock_repo.anonymize_user(user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_check_legal_holds() {
        let mut mock_repo = MockGdprRepo::new();

        let user_id = Uuid::new_v4();
        mock_repo
            .expect_check_legal_holds()
            .times(1)
            .returning(|_| Ok(vec!["Unpaid expenses".to_string()]));

        let result = mock_repo.check_legal_holds(user_id).await;
        assert_eq!(result.unwrap(), vec!["Unpaid expenses".to_string()]);
    }

    #[tokio::test]
    async fn test_mock_is_user_anonymized() {
        let mut mock_repo = MockGdprRepo::new();

        let user_id = Uuid::new_v4();
        mock_repo
            .expect_is_user_anonymized()
            .times(1)
            .returning(|_| Ok(false));

        let result = mock_repo.is_user_anonymized(user_id).await;
        assert!(!result.unwrap());
    }
}
