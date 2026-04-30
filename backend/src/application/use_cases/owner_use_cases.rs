use crate::application::dto::{CreateOwnerDto, OwnerFilters, OwnerResponseDto, PageRequest};
use crate::application::ports::OwnerRepository;
use crate::domain::entities::Owner;
use std::sync::Arc;
use uuid::Uuid;

pub struct OwnerUseCases {
    repository: Arc<dyn OwnerRepository>,
}

impl OwnerUseCases {
    pub fn new(repository: Arc<dyn OwnerRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_owner(&self, dto: CreateOwnerDto) -> Result<OwnerResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;

        // Vérifier si l'email existe déjà
        if (self.repository.find_by_email(&dto.email).await?).is_some() {
            return Err("Email already exists".to_string());
        }

        let mut owner = Owner::new(
            organization_id,
            dto.first_name,
            dto.last_name,
            dto.email,
            dto.phone,
            dto.address,
            dto.city,
            dto.postal_code,
            dto.country,
        )?;

        // Link to user account if provided
        if let Some(user_id_str) = dto.user_id {
            if !user_id_str.is_empty() {
                owner.user_id = Some(
                    Uuid::parse_str(&user_id_str)
                        .map_err(|_| "Invalid user_id format".to_string())?,
                );
            }
        }

        let created = self.repository.create(&owner).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_owner(&self, id: Uuid) -> Result<Option<OwnerResponseDto>, String> {
        let owner = self.repository.find_by_id(id).await?;
        Ok(owner.map(|o| self.to_response_dto(&o)))
    }

    pub async fn find_owner_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<OwnerResponseDto>, String> {
        let owner = self.repository.find_by_user_id(user_id).await?;
        Ok(owner.map(|o| self.to_response_dto(&o)))
    }

    pub async fn find_owner_by_user_id_and_organization(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Option<OwnerResponseDto>, String> {
        let owner = self
            .repository
            .find_by_user_id_and_organization(user_id, organization_id)
            .await?;
        Ok(owner.map(|o| self.to_response_dto(&o)))
    }

    pub async fn list_owners(&self) -> Result<Vec<OwnerResponseDto>, String> {
        let owners = self.repository.find_all().await?;
        Ok(owners.iter().map(|o| self.to_response_dto(o)).collect())
    }

    pub async fn list_owners_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<OwnerResponseDto>, i64), String> {
        let filters = OwnerFilters {
            organization_id,
            ..Default::default()
        };

        let (owners, total) = self
            .repository
            .find_all_paginated(page_request, &filters)
            .await?;

        let dtos = owners.iter().map(|o| self.to_response_dto(o)).collect();
        Ok((dtos, total))
    }

    pub async fn update_owner(
        &self,
        id: Uuid,
        first_name: String,
        last_name: String,
        email: String,
        phone: Option<String>,
    ) -> Result<OwnerResponseDto, String> {
        // Get existing owner
        let mut owner = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or("Owner not found".to_string())?;

        // Check if email is being changed and if the new email already exists
        if owner.email != email {
            if let Some(existing) = self.repository.find_by_email(&email).await? {
                if existing.id != id {
                    return Err("Email already exists".to_string());
                }
            }
        }

        // Update owner fields
        owner.first_name = first_name;
        owner.last_name = last_name;
        owner.email = email;
        owner.phone = phone;

        // Save updated owner
        let updated = self.repository.update(&owner).await?;
        Ok(self.to_response_dto(&updated))
    }

    /// Link or unlink a user account to an owner.
    /// Returns `Err` with a human-readable message if another owner is already linked
    /// to the same user, or if the owner was not found.
    pub async fn link_user_to_owner(
        &self,
        owner_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<(), String> {
        if let Some(uid) = user_id {
            // Conflict check: is this user already linked to a different owner?
            if let Some(existing) = self.repository.find_by_user_id(uid).await? {
                if existing.id != owner_id {
                    return Err(format!(
                        "User is already linked to owner {} {} (ID: {})",
                        existing.first_name, existing.last_name, existing.id
                    ));
                }
            }
        }

        let updated = self.repository.set_user_link(owner_id, user_id).await?;
        if !updated {
            return Err("Owner not found".to_string());
        }
        Ok(())
    }

    fn to_response_dto(&self, owner: &Owner) -> OwnerResponseDto {
        OwnerResponseDto {
            id: owner.id.to_string(),
            organization_id: owner.organization_id.to_string(),
            user_id: owner.user_id.map(|id| id.to_string()),
            first_name: owner.first_name.clone(),
            last_name: owner.last_name.clone(),
            email: owner.email.clone(),
            phone: owner.phone.clone(),
            address: owner.address.clone(),
            city: owner.city.clone(),
            postal_code: owner.postal_code.clone(),
            country: owner.country.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockOwnerRepository {
        items: Mutex<HashMap<Uuid, Owner>>,
    }

    impl MockOwnerRepository {
        fn new() -> Self {
            Self {
                items: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerRepository for MockOwnerRepository {
        async fn create(&self, owner: &Owner) -> Result<Owner, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(owner.id, owner.clone());
            Ok(owner.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.get(&id).cloned())
        }

        async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.values().find(|o| o.user_id == Some(user_id)).cloned())
        }

        async fn find_by_user_id_and_organization(
            &self,
            user_id: Uuid,
            organization_id: Uuid,
        ) -> Result<Option<Owner>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .find(|o| o.user_id == Some(user_id) && o.organization_id == organization_id)
                .cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.values().find(|o| o.email == email).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Owner>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.values().cloned().collect())
        }

        async fn find_all_paginated(
            &self,
            page_request: &PageRequest,
            _filters: &OwnerFilters,
        ) -> Result<(Vec<Owner>, i64), String> {
            let items = self.items.lock().unwrap();
            let all: Vec<Owner> = items.values().cloned().collect();
            let total = all.len() as i64;
            let offset = page_request.offset() as usize;
            let limit = page_request.limit() as usize;
            let page = all.into_iter().skip(offset).take(limit).collect();
            Ok((page, total))
        }

        async fn update(&self, owner: &Owner) -> Result<Owner, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(owner.id, owner.clone());
            Ok(owner.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut items = self.items.lock().unwrap();
            Ok(items.remove(&id).is_some())
        }

        async fn set_user_link(
            &self,
            owner_id: Uuid,
            user_id: Option<Uuid>,
        ) -> Result<bool, String> {
            let mut items = self.items.lock().unwrap();
            if let Some(owner) = items.get_mut(&owner_id) {
                owner.user_id = user_id;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    fn make_use_cases(repo: MockOwnerRepository) -> OwnerUseCases {
        OwnerUseCases::new(Arc::new(repo))
    }

    fn make_create_dto(org_id: Uuid) -> CreateOwnerDto {
        CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Jean".to_string(),
            last_name: "Dupont".to_string(),
            email: "jean.dupont@example.com".to_string(),
            phone: Some("+32470123456".to_string()),
            address: "Rue de la Loi 16".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgique".to_string(),
            user_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_owner_success() {
        let repo = MockOwnerRepository::new();
        let use_cases = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let result = use_cases.create_owner(make_create_dto(org_id)).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.first_name, "Jean");
        assert_eq!(dto.last_name, "Dupont");
        assert_eq!(dto.email, "jean.dupont@example.com");
        assert_eq!(dto.organization_id, org_id.to_string());
    }

    #[tokio::test]
    async fn test_create_owner_duplicate_email() {
        let repo = MockOwnerRepository::new();
        let org_id = Uuid::new_v4();

        // Pre-populate with an existing owner having the same email
        let existing = Owner::new(
            org_id,
            "Marie".to_string(),
            "Martin".to_string(),
            "jean.dupont@example.com".to_string(),
            None,
            "Av. Louise 100".to_string(),
            "Bruxelles".to_string(),
            "1050".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();
        repo.items.lock().unwrap().insert(existing.id, existing);

        let use_cases = make_use_cases(repo);
        let result = use_cases.create_owner(make_create_dto(org_id)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email already exists");
    }

    #[tokio::test]
    async fn test_get_owner_found() {
        let repo = MockOwnerRepository::new();
        let org_id = Uuid::new_v4();
        let owner = Owner::new(
            org_id,
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean@example.com".to_string(),
            None,
            "Rue Test 1".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();
        let owner_id = owner.id;
        repo.items.lock().unwrap().insert(owner.id, owner);

        let use_cases = make_use_cases(repo);
        let result = use_cases.get_owner(owner_id).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert!(dto.is_some());
        assert_eq!(dto.unwrap().first_name, "Jean");
    }

    #[tokio::test]
    async fn test_get_owner_not_found() {
        let repo = MockOwnerRepository::new();
        let use_cases = make_use_cases(repo);

        let result = use_cases.get_owner(Uuid::new_v4()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_owners() {
        let repo = MockOwnerRepository::new();
        let org_id = Uuid::new_v4();

        let owner1 = Owner::new(
            org_id,
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean@example.com".to_string(),
            None,
            "Rue A".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();
        let owner2 = Owner::new(
            org_id,
            "Marie".to_string(),
            "Martin".to_string(),
            "marie@example.com".to_string(),
            None,
            "Rue B".to_string(),
            "Liege".to_string(),
            "4000".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(owner1.id, owner1);
            items.insert(owner2.id, owner2);
        }

        let use_cases = make_use_cases(repo);
        let result = use_cases.list_owners().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_update_owner_success() {
        let repo = MockOwnerRepository::new();
        let org_id = Uuid::new_v4();
        let owner = Owner::new(
            org_id,
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean@example.com".to_string(),
            None,
            "Rue Test".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();
        let owner_id = owner.id;
        repo.items.lock().unwrap().insert(owner.id, owner);

        let use_cases = make_use_cases(repo);
        let result = use_cases
            .update_owner(
                owner_id,
                "Pierre".to_string(),
                "Durand".to_string(),
                "pierre@example.com".to_string(),
                Some("+32470999999".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.first_name, "Pierre");
        assert_eq!(dto.last_name, "Durand");
        assert_eq!(dto.email, "pierre@example.com");
        assert_eq!(dto.phone, Some("+32470999999".to_string()));
    }

    #[tokio::test]
    async fn test_update_owner_email_conflict() {
        let repo = MockOwnerRepository::new();
        let org_id = Uuid::new_v4();

        let owner1 = Owner::new(
            org_id,
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean@example.com".to_string(),
            None,
            "Rue A".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();
        let owner1_id = owner1.id;

        let owner2 = Owner::new(
            org_id,
            "Marie".to_string(),
            "Martin".to_string(),
            "marie@example.com".to_string(),
            None,
            "Rue B".to_string(),
            "Liege".to_string(),
            "4000".to_string(),
            "Belgique".to_string(),
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(owner1.id, owner1);
            items.insert(owner2.id, owner2);
        }

        let use_cases = make_use_cases(repo);

        // Try to update owner1's email to owner2's email
        let result = use_cases
            .update_owner(
                owner1_id,
                "Jean".to_string(),
                "Dupont".to_string(),
                "marie@example.com".to_string(), // Already taken by owner2
                None,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email already exists");
    }
}
