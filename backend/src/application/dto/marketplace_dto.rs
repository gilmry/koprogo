// DTOs for Marketplace API (Issue #276)

use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::domain::entities::ServiceProvider;

/// Request DTO for creating a service provider
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateServiceProviderDto {
    #[validate(length(min = 1, max = 200, message = "Company name must be 1-200 characters"))]
    pub company_name: String,

    pub trade_category: String, // e.g., "Plombier", "Electricien"

    pub specializations: Option<Vec<String>>,

    pub service_zone_postal_codes: Option<Vec<String>>,

    pub certifications: Option<Vec<String>>,

    pub ipi_registration: Option<String>,

    pub bce_number: Option<String>,
}

/// Response DTO for service provider (public marketplace view)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceProviderResponseDto {
    pub id: String,
    pub organization_id: String,
    pub company_name: String,
    pub trade_category: String,
    pub specializations: Vec<String>,
    pub service_zone_postal_codes: Vec<String>,
    pub certifications: Vec<String>,
    pub ipi_registration: Option<String>,
    pub bce_number: Option<String>,
    pub rating_avg: Option<f64>,
    pub reviews_count: i32,
    pub is_verified: bool,
    pub public_profile_slug: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Query parameters for searching service providers (public marketplace)
#[derive(Debug, Deserialize, Clone)]
pub struct SearchServiceProvidersQuery {
    pub trade_category: Option<String>,
    pub postal_code: Option<String>,
    pub min_rating: Option<f64>,
    pub is_verified_only: Option<bool>,
}

/// Request DTO for contract evaluation
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateContractEvaluationDto {
    pub service_provider_id: String,
    pub quote_id: Option<String>,
    pub ticket_id: Option<String>,
    #[validate(length(min = 1))]
    pub criteria: std::collections::HashMap<String, u8>,
    pub would_recommend: bool,
    pub comments: Option<String>,
    pub is_anonymous: Option<bool>,
}

/// Response DTO for contract evaluation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractEvaluationResponseDto {
    pub id: String,
    pub organization_id: String,
    pub service_provider_id: String,
    pub evaluator_id: String,
    pub building_id: String,
    pub quote_id: Option<String>,
    pub ticket_id: Option<String>,
    pub criteria: std::collections::HashMap<String, u8>,
    pub global_score: f64,
    pub comments: Option<String>,
    pub would_recommend: bool,
    pub is_legal_evaluation: bool,
    pub is_anonymous: bool,
    pub created_at: String,
}

/// Response DTO for L13 annual report (contract evaluations)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractEvaluationsAnnualReportDto {
    pub building_id: String,
    pub report_year: i32,
    pub total_evaluations: i32,
    pub total_providers_evaluated: i32,
    pub average_global_score: f64,
    pub recommendation_rate: f64, // % who would recommend
    pub evaluations: Vec<ContractEvaluationResponseDto>,
}

impl From<ServiceProvider> for ServiceProviderResponseDto {
    fn from(provider: ServiceProvider) -> Self {
        ServiceProviderResponseDto {
            id: provider.id.to_string(),
            organization_id: provider.organization_id.to_string(),
            company_name: provider.company_name,
            trade_category: provider.trade_category.to_sql().to_string(),
            specializations: provider.specializations,
            service_zone_postal_codes: provider.service_zone_postal_codes,
            certifications: provider.certifications,
            ipi_registration: provider.ipi_registration,
            bce_number: provider.bce_number,
            rating_avg: provider.rating_avg,
            reviews_count: provider.reviews_count,
            is_verified: provider.is_verified,
            public_profile_slug: provider.public_profile_slug,
            created_at: provider.created_at.to_rfc3339(),
            updated_at: provider.updated_at.to_rfc3339(),
        }
    }
}
