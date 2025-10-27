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

        let owner = Owner::new(
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

        let created = self.repository.create(&owner).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_owner(&self, id: Uuid) -> Result<Option<OwnerResponseDto>, String> {
        let owner = self.repository.find_by_id(id).await?;
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

    fn to_response_dto(&self, owner: &Owner) -> OwnerResponseDto {
        OwnerResponseDto {
            id: owner.id.to_string(),
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
