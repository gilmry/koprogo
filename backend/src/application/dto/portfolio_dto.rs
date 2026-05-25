//! Portfolio DTOs — Story 2.1.
//!
//! Request/Response DTOs pour les endpoints `/portfolios`. Validation via
//! `validator`. Bornes alignées avec les invariants `Portfolio::new`
//! (cf. `domain/entities/portfolio.rs`).

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Création d'un portfolio.
///
/// `owner_user_id` est inféré côté handler depuis `AuthenticatedUser`
/// — pas exposé dans le body pour éviter toute escalade.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreatePortfolioDto {
    #[validate(length(
        min = 2,
        max = 120,
        message = "Name must be between 2 and 120 characters"
    ))]
    pub name: String,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
}

/// Mise à jour d'un portfolio (PUT — état complet).
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdatePortfolioDto {
    #[validate(length(
        min = 2,
        max = 120,
        message = "Name must be between 2 and 120 characters"
    ))]
    pub name: String,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
}

/// Ajout d'un building au portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct AddBuildingDto {
    pub building_id: String,
    #[serde(default)]
    pub is_favorite: bool,
}

/// Partage du portfolio avec un autre utilisateur.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct SharePortfolioDto {
    pub shared_with_user_id: String,
    #[serde(default)]
    pub can_edit: bool,
}

/// Réponse JSON pour un portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PortfolioResponseDto {
    pub id: String,
    pub owner_user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Building du portfolio (élément du listing trié favoris d'abord).
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PortfolioBuildingResponseDto {
    pub portfolio_id: String,
    pub building_id: String,
    pub is_favorite: bool,
}

/// Partage du portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PortfolioShareResponseDto {
    pub portfolio_id: String,
    pub shared_with_user_id: String,
    pub can_edit: bool,
    pub shared_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_valid_create_dto_passes_validation() {
        let dto = CreatePortfolioDto {
            name: "Mes favoris".to_string(),
            description: None,
        };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn negative_too_short_name_fails_validation() {
        let dto = CreatePortfolioDto {
            name: "A".to_string(),
            description: None,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn negative_too_long_description_fails_validation() {
        let dto = CreatePortfolioDto {
            name: "Name".to_string(),
            description: Some("x".repeat(1001)),
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn negative_empty_name_fails_validation() {
        let dto = CreatePortfolioDto {
            name: "".to_string(),
            description: None,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn happy_share_dto_default_can_edit_false() {
        let json = r#"{"shared_with_user_id": "00000000-0000-0000-0000-000000000001"}"#;
        let dto: SharePortfolioDto = serde_json::from_str(json).unwrap();
        assert!(!dto.can_edit);
    }
}
