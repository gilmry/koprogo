use crate::application::dto::{CreateOwnerDto, OwnerResponseDto};
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
        // Vérifier si l'email existe déjà
        if let Some(_) = self.repository.find_by_email(&dto.email).await? {
            return Err("Email already exists".to_string());
        }

        let owner = Owner::new(
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
