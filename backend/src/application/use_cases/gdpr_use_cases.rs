use crate::application::dto::{GdprEraseResponseDto, GdprExportResponseDto};
use crate::application::ports::GdprRepository;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// GDPR Use Cases for data export and erasure operations
/// Implements business logic for GDPR Article 15 (Right to Access) and Article 17 (Right to Erasure)
pub struct GdprUseCases {
    gdpr_repository: Arc<dyn GdprRepository>,
}

impl GdprUseCases {
    pub fn new(gdpr_repository: Arc<dyn GdprRepository>) -> Self {
        Self { gdpr_repository }
    }

    /// Export all personal data for a user (GDPR Article 15 - Right to Access)
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user requesting data export
    /// * `requesting_user_id` - UUID of the user making the request (for authorization)
    /// * `organization_id` - Optional organization scope (None for SuperAdmin)
    ///
    /// # Authorization
    /// - Users can only export their own data
    /// - SuperAdmin can export any user's data
    ///
    /// # Returns
    /// * `Ok(GdprExportResponseDto)` - Complete data export in JSON format
    /// * `Err(String)` - If user not found, not authorized, or database error
    pub async fn export_user_data(
        &self,
        user_id: Uuid,
        requesting_user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<GdprExportResponseDto, String> {
        // Authorization check: user can only export their own data
        // SuperAdmin bypass is handled by passing organization_id = None
        if user_id != requesting_user_id && organization_id.is_some() {
            return Err("Unauthorized: You can only export your own data".to_string());
        }

        // Check if user is already anonymized
        let is_anonymized = self.gdpr_repository.is_user_anonymized(user_id).await?;
        if is_anonymized {
            return Err("User data has been anonymized and cannot be exported".to_string());
        }

        // Aggregate all user data from database
        let export = self
            .gdpr_repository
            .aggregate_user_data(user_id, organization_id)
            .await?;

        // Convert domain entity to DTO
        Ok(GdprExportResponseDto::from(export))
    }

    /// Erase user data by anonymization (GDPR Article 17 - Right to Erasure)
    ///
    /// Anonymizes user account and linked owner profiles. Does not delete data entirely
    /// to preserve referential integrity and comply with legal retention requirements
    /// (e.g., financial records must be kept for 7 years in Belgium).
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user to anonymize
    /// * `requesting_user_id` - UUID of the user making the request (for authorization)
    /// * `organization_id` - Optional organization scope (None for SuperAdmin)
    ///
    /// # Authorization
    /// - Users can only erase their own data
    /// - SuperAdmin can erase any user's data
    ///
    /// # Returns
    /// * `Ok(GdprEraseResponseDto)` - Anonymization confirmation
    /// * `Err(String)` - If user not found, not authorized, already anonymized, or legal holds exist
    pub async fn erase_user_data(
        &self,
        user_id: Uuid,
        requesting_user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<GdprEraseResponseDto, String> {
        // Authorization check
        if user_id != requesting_user_id && organization_id.is_some() {
            return Err("Unauthorized: You can only erase your own data".to_string());
        }

        // Check if already anonymized
        let is_anonymized = self.gdpr_repository.is_user_anonymized(user_id).await?;
        if is_anonymized {
            return Err("User data is already anonymized".to_string());
        }

        // Check for legal holds (e.g., unpaid expenses, ongoing legal proceedings)
        let holds = self.gdpr_repository.check_legal_holds(user_id).await?;
        if !holds.is_empty() {
            return Err(format!(
                "Cannot erase data due to legal holds: {}",
                holds.join(", ")
            ));
        }

        // Retrieve user data BEFORE anonymization (needed for email notification)
        let user_data = self
            .gdpr_repository
            .aggregate_user_data(user_id, organization_id)
            .await?;
        let user_email = user_data.user_data.email.clone();
        let user_first_name = user_data.user_data.first_name.clone();
        let user_last_name = user_data.user_data.last_name.clone();

        // Find all linked owner profiles
        let owner_ids = self
            .gdpr_repository
            .find_owner_ids_by_user(user_id, organization_id)
            .await?;

        // Anonymize user account
        self.gdpr_repository.anonymize_user(user_id).await?;

        // Anonymize all linked owner profiles
        let mut owners_anonymized = 0;
        for owner_id in &owner_ids {
            match self.gdpr_repository.anonymize_owner(*owner_id).await {
                Ok(_) => owners_anonymized += 1,
                Err(e) => {
                    // Log error but continue (partial anonymization is acceptable)
                    eprintln!("Warning: Failed to anonymize owner {}: {}", owner_id, e);
                }
            }
        }

        Ok(GdprEraseResponseDto {
            success: true,
            message: "Personal data has been successfully anonymized".to_string(),
            anonymized_at: Utc::now().to_rfc3339(),
            user_id: user_id.to_string(),
            user_email,
            user_first_name,
            user_last_name,
            owners_anonymized,
        })
    }

    /// Check if user data can be erased (no legal holds)
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user to check
    ///
    /// # Returns
    /// * `Ok(true)` - User can be erased
    /// * `Ok(false)` - User has legal holds preventing erasure
    /// * `Err(String)` - Database error
    pub async fn can_erase_user(&self, user_id: Uuid) -> Result<bool, String> {
        let holds = self.gdpr_repository.check_legal_holds(user_id).await?;
        Ok(holds.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::gdpr_repository::MockGdprRepo;
    use crate::domain::entities::gdpr_export::{GdprExport, UserData};
    use chrono::Utc;

    fn create_test_user_data(user_id: Uuid) -> UserData {
        UserData {
            id: user_id,
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            organization_id: Some(Uuid::new_v4()),
            is_active: true,
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_export_user_data_success() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_is_user_anonymized()
            .times(1)
            .returning(|_| Ok(false));
        mock_repo
            .expect_aggregate_user_data()
            .times(1)
            .returning(move |_, _| {
                let user_data = create_test_user_data(user_id);
                Ok(GdprExport::new(user_data))
            });

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases
            .export_user_data(user_id, user_id, Some(org_id))
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_export_user_data_unauthorized() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let mock_repo = MockGdprRepo::new();
        let use_cases = GdprUseCases::new(Arc::new(mock_repo));

        let result = use_cases
            .export_user_data(user_id, other_user_id, Some(org_id))
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unauthorized: You can only export your own data"));
    }

    #[tokio::test]
    async fn test_export_anonymized_user_fails() {
        let user_id = Uuid::new_v4();

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_is_user_anonymized()
            .times(1)
            .returning(|_| Ok(true));

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases
            .export_user_data(user_id, user_id, Some(Uuid::new_v4()))
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("User data has been anonymized"));
    }

    #[tokio::test]
    async fn test_erase_user_data_success() {
        let user_id = Uuid::new_v4();
        let owner_id1 = Uuid::new_v4();
        let owner_id2 = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        // Create test user data
        let user_data = crate::domain::entities::gdpr_export::UserData {
            id: user_id,
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            organization_id: Some(org_id),
            is_active: true,
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let gdpr_export = crate::domain::entities::gdpr_export::GdprExport::new(user_data);

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_is_user_anonymized()
            .times(1)
            .returning(|_| Ok(false));
        mock_repo
            .expect_check_legal_holds()
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_repo
            .expect_aggregate_user_data()
            .times(1)
            .returning(move |_, _| Ok(gdpr_export.clone()));
        mock_repo
            .expect_find_owner_ids_by_user()
            .times(1)
            .returning(move |_, _| Ok(vec![owner_id1, owner_id2]));
        mock_repo
            .expect_anonymize_user()
            .times(1)
            .returning(|_| Ok(()));
        mock_repo
            .expect_anonymize_owner()
            .times(2)
            .returning(|_| Ok(()));

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases
            .erase_user_data(user_id, user_id, Some(org_id))
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert!(dto.success);
        assert_eq!(dto.owners_anonymized, 2);
        assert_eq!(dto.user_email, "test@example.com");
        assert_eq!(dto.user_first_name, "Test");
        assert_eq!(dto.user_last_name, "User");
    }

    #[tokio::test]
    async fn test_erase_user_data_unauthorized() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let mock_repo = MockGdprRepo::new();
        let use_cases = GdprUseCases::new(Arc::new(mock_repo));

        let result = use_cases
            .erase_user_data(user_id, other_user_id, Some(org_id))
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unauthorized: You can only erase your own data"));
    }

    #[tokio::test]
    async fn test_erase_already_anonymized_user_fails() {
        let user_id = Uuid::new_v4();

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_is_user_anonymized()
            .times(1)
            .returning(|_| Ok(true));

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases
            .erase_user_data(user_id, user_id, Some(Uuid::new_v4()))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already anonymized"));
    }

    #[tokio::test]
    async fn test_erase_with_legal_holds_fails() {
        let user_id = Uuid::new_v4();

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_is_user_anonymized()
            .times(1)
            .returning(|_| Ok(false));
        mock_repo
            .expect_check_legal_holds()
            .times(1)
            .returning(|_| Ok(vec!["Unpaid expenses".to_string()]));

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases
            .erase_user_data(user_id, user_id, Some(Uuid::new_v4()))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("legal holds"));
    }

    #[tokio::test]
    async fn test_can_erase_user_no_holds() {
        let user_id = Uuid::new_v4();

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_check_legal_holds()
            .times(1)
            .returning(|_| Ok(vec![]));

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases.can_erase_user(user_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_erase_user_with_holds() {
        let user_id = Uuid::new_v4();

        let mut mock_repo = MockGdprRepo::new();
        mock_repo
            .expect_check_legal_holds()
            .times(1)
            .returning(|_| Ok(vec!["Unpaid expenses".to_string()]));

        let use_cases = GdprUseCases::new(Arc::new(mock_repo));
        let result = use_cases.can_erase_user(user_id).await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
