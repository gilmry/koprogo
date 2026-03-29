use crate::application::dto::marketplace_dto::{
    CreateServiceProviderDto, SearchServiceProvidersQuery, ServiceProviderResponseDto,
};
use crate::application::ports::service_provider_repository::ServiceProviderRepository;
use crate::domain::entities::service_provider::{ServiceProvider, TradeCategory};
use std::sync::Arc;
use uuid::Uuid;

pub struct ServiceProviderUseCases {
    pub repo: Arc<dyn ServiceProviderRepository>,
}

impl ServiceProviderUseCases {
    pub fn new(repo: Arc<dyn ServiceProviderRepository>) -> Self {
        Self { repo }
    }

    /// Create a new service provider
    pub async fn create(
        &self,
        organization_id: Uuid,
        dto: CreateServiceProviderDto,
    ) -> Result<ServiceProviderResponseDto, String> {
        let trade_category =
            TradeCategory::from_sql(&dto.trade_category).map_err(|e| e.to_string())?;

        let mut provider = ServiceProvider::new(
            organization_id,
            dto.company_name,
            trade_category,
            dto.bce_number,
        )?;

        if let Some(specs) = dto.specializations {
            provider.specializations = specs;
        }
        if let Some(zones) = dto.service_zone_postal_codes {
            provider.service_zone_postal_codes = zones;
        }
        if let Some(certs) = dto.certifications {
            provider.certifications = certs;
        }
        if let Some(ipi) = dto.ipi_registration {
            provider.ipi_registration = Some(ipi);
        }

        let created = self.repo.create(&provider).await?;
        Ok(ServiceProviderResponseDto::from(created))
    }

    /// Search providers with filters
    pub async fn search(
        &self,
        query: &SearchServiceProvidersQuery,
    ) -> Result<Vec<ServiceProviderResponseDto>, String> {
        let providers = if let Some(ref q) = query.trade_category {
            let category = TradeCategory::from_sql(q).map_err(|e| e.to_string())?;
            self.repo.find_by_trade_category(category, 1, 50).await?
        } else if let Some(ref postal_code) = query.postal_code {
            self.repo.search("", Some(postal_code), 1, 50).await?
        } else {
            self.repo.find_all(None, 1, 50).await?
        };

        let mut results: Vec<ServiceProviderResponseDto> = providers
            .into_iter()
            .map(ServiceProviderResponseDto::from)
            .collect();

        // Apply client-side filters
        if let Some(min_rating) = query.min_rating {
            results.retain(|p| p.rating_avg.unwrap_or(0.0) >= min_rating);
        }
        if query.is_verified_only == Some(true) {
            results.retain(|p| p.is_verified);
        }

        Ok(results)
    }

    /// Find provider by slug (public profile)
    pub async fn find_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ServiceProviderResponseDto>, String> {
        let provider = self.repo.find_by_slug(slug).await?;
        Ok(provider.map(ServiceProviderResponseDto::from))
    }
}
