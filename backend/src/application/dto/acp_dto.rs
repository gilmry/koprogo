//! ACP DTOs — Story 1.1 (refonte UX multi-rôle ACP).
//!
//! Request/Response DTOs pour les endpoints `/acps`. Validation via `validator`.
//! Bornes alignées avec les invariants `Acp::new` (cf. `domain/entities/acp.rs`).

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Création d'une ACP.
///
/// Champs :
/// - `organization_id` : `Option<String>` — UUID du cabinet syndic, `None` si
///   ACP auto-gérée (ADR-0010).
/// - `name` : 2..=160 chars (post-trim côté domain).
/// - `address_street`, `address_postal_code`, `address_city` : obligatoires.
/// - `bce_number` : optionnel (toutes les ACPs ne sont pas immatriculées BCE).
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateAcpDto {
    pub organization_id: Option<String>,

    #[validate(length(
        min = 2,
        max = 160,
        message = "Name must be between 2 and 160 characters"
    ))]
    pub name: String,

    #[validate(length(min = 1, message = "address_street cannot be empty"))]
    pub address_street: String,

    #[validate(length(min = 1, message = "address_postal_code cannot be empty"))]
    pub address_postal_code: String,

    #[validate(length(min = 1, message = "address_city cannot be empty"))]
    pub address_city: String,

    #[validate(length(max = 20, message = "bce_number too long"))]
    pub bce_number: Option<String>,
}

/// Mise à jour d'une ACP (PATCH-like : tous les champs identitaires sont requis,
/// par défaut on ré-envoie l'état complet via PUT — pattern Building).
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateAcpDto {
    /// Permet à un admin de rattacher / détacher l'ACP d'un cabinet.
    /// `Some(None)` (JSON `"organization_id": null`) = détache,
    /// `None` (clé absente) = conserve l'existant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<Option<String>>,

    #[validate(length(
        min = 2,
        max = 160,
        message = "Name must be between 2 and 160 characters"
    ))]
    pub name: String,

    #[validate(length(min = 1, message = "address_street cannot be empty"))]
    pub address_street: String,

    #[validate(length(min = 1, message = "address_postal_code cannot be empty"))]
    pub address_postal_code: String,

    #[validate(length(min = 1, message = "address_city cannot be empty"))]
    pub address_city: String,

    #[validate(length(max = 20, message = "bce_number too long"))]
    pub bce_number: Option<String>,
}

/// Réponse JSON pour une ACP.
///
/// IDs et timestamps sérialisés en `String` (UUID + RFC3339) pour
/// cohérence avec les autres DTOs (cf. `BuildingResponseDto`).
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AcpResponseDto {
    pub id: String,
    pub organization_id: Option<String>,
    pub name: String,
    pub slug: String,
    pub legal_status: String,
    pub bce_number: Option<String>,
    pub address_street: String,
    pub address_postal_code: String,
    pub address_city: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn happy_valid_create_dto_passes_validation() {
        let dto = CreateAcpDto {
            organization_id: Some(uuid::Uuid::new_v4().to_string()),
            name: "Acp Test".to_string(),
            address_street: "Rue X 1".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Bruxelles".to_string(),
            bce_number: None,
        };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn negative_too_short_name_fails_validation() {
        let dto = CreateAcpDto {
            organization_id: None,
            name: "A".to_string(),
            address_street: "Rue X 1".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Bruxelles".to_string(),
            bce_number: None,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn negative_empty_street_fails_validation() {
        let dto = CreateAcpDto {
            organization_id: None,
            name: "Acp Test".to_string(),
            address_street: "".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Bruxelles".to_string(),
            bce_number: None,
        };
        assert!(dto.validate().is_err());
    }
}
