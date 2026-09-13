//! `Portfolio` — portefeuille immeubles d'un utilisateur (favoris/équipe).
//!
//! Story 2.1 — Slice 2 Refonte UX multi-rôle ACP.
//! Source : `docs/maury/refonte-ux-multi-role-acp/architecture.md` §2.5 + ADR-0011.
//!
//! Un portefeuille (`Portfolio`) regroupe N immeubles `(building_id, is_favorite)`
//! pour un propriétaire `owner_user_id`. Il peut être partagé en lecture
//! (option `can_edit`) avec d'autres `User` (typiquement équipe d'un cabinet
//! syndic).
//!
//! # Invariants
//!
//! - `name` non vide après trim, longueur ∈ [2, 120] caractères.
//! - `description` optionnel (longueur ≤ 1000 si présent, post-trim).
//!
//! # Hexagonal
//!
//! Aucune dépendance `sqlx` / `actix_web`. Les erreurs domaine retournent
//! `PortfolioError` (enum), mappé vers `AppError::Validation` côté
//! application (`application/error.rs`, pattern WP-A* #433).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Erreurs métier produites par le domaine `Portfolio`.
///
/// Mappées vers `AppError::Validation` (HTTP 400/422).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PortfolioError {
    #[error("Portfolio name cannot be empty")]
    NameEmpty,
    #[error("Portfolio name must be at least 2 characters long, got {0}")]
    NameTooShort(usize),
    #[error("Portfolio name must be at most 120 characters long, got {0}")]
    NameTooLong(usize),
    #[error("Portfolio description must be at most 1000 characters long, got {0}")]
    DescriptionTooLong(usize),
}

/// Représente un portefeuille (Aggregate Root).
///
/// Cf. ADR-0011 (`docs/maury/refonte-ux-multi-role-acp/architecture.md`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct Portfolio {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Portfolio {
    /// Constructeur validé.
    ///
    /// Invariants vérifiés :
    /// 1. `name.trim()` ∈ [2, 120] caractères.
    /// 2. `description.trim()` ≤ 1000 caractères si présent.
    pub fn new(
        owner_user_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Self, PortfolioError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(PortfolioError::NameEmpty);
        }
        let name_len = name.chars().count();
        if name_len < 2 {
            return Err(PortfolioError::NameTooShort(name_len));
        }
        if name_len > 120 {
            return Err(PortfolioError::NameTooLong(name_len));
        }

        let description = match description {
            Some(s) => {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    let d_len = trimmed.chars().count();
                    if d_len > 1000 {
                        return Err(PortfolioError::DescriptionTooLong(d_len));
                    }
                    Some(trimmed)
                }
            }
            None => None,
        };

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            owner_user_id,
            name,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    /// Mise à jour `name` + `description` avec re-validation des invariants.
    pub fn update_info(
        &mut self,
        name: String,
        description: Option<String>,
    ) -> Result<(), PortfolioError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(PortfolioError::NameEmpty);
        }
        let name_len = name.chars().count();
        if name_len < 2 {
            return Err(PortfolioError::NameTooShort(name_len));
        }
        if name_len > 120 {
            return Err(PortfolioError::NameTooLong(name_len));
        }
        let description = match description {
            Some(s) => {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    let d_len = trimmed.chars().count();
                    if d_len > 1000 {
                        return Err(PortfolioError::DescriptionTooLong(d_len));
                    }
                    Some(trimmed)
                }
            }
            None => None,
        };
        self.name = name;
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Entité de liaison M:N — un building dans un portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct PortfolioBuilding {
    pub portfolio_id: Uuid,
    pub building_id: Uuid,
    pub is_favorite: bool,
    pub added_at: DateTime<Utc>,
}

/// Entité de liaison — partage portfolio ↔ user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct PortfolioShare {
    pub portfolio_id: Uuid,
    pub shared_with_user_id: Uuid,
    pub can_edit: bool,
    pub shared_at: DateTime<Utc>,
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md règle #3, #427).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- @happy -------------------------------------------------------------

    #[test]
    fn happy_new_portfolio_minimal_succeeds() {
        let user_id = Uuid::new_v4();
        let p = Portfolio::new(user_id, "Mes immeubles favoris".to_string(), None)
            .expect("valid portfolio");
        assert_eq!(p.owner_user_id, user_id);
        assert_eq!(p.name, "Mes immeubles favoris");
        assert!(p.description.is_none());
        assert_eq!(p.created_at, p.updated_at);
    }

    #[test]
    fn happy_new_portfolio_with_description() {
        let p = Portfolio::new(
            Uuid::new_v4(),
            "Cabinet B".to_string(),
            Some("Immeubles du portefeuille du gestionnaire B".to_string()),
        )
        .unwrap();
        assert_eq!(
            p.description.as_deref(),
            Some("Immeubles du portefeuille du gestionnaire B")
        );
    }

    #[test]
    fn happy_update_info_changes_name_and_touches_updated_at() {
        let mut p = Portfolio::new(Uuid::new_v4(), "Old".to_string(), None).unwrap();
        let original = p.updated_at;
        // Sleep avoided: use_cases tests cover real time deltas — here we
        // just check the value moved forward (>= original).
        p.update_info("New".to_string(), Some("desc".to_string()))
            .unwrap();
        assert_eq!(p.name, "New");
        assert_eq!(p.description.as_deref(), Some("desc"));
        assert!(p.updated_at >= original);
    }

    // ----- @edge --------------------------------------------------------------

    #[test]
    fn edge_minimum_name_length_2_accepted() {
        let p = Portfolio::new(Uuid::new_v4(), "Ab".to_string(), None);
        assert!(p.is_ok());
    }

    #[test]
    fn edge_max_name_length_120_accepted() {
        let name = "A".repeat(120);
        let p = Portfolio::new(Uuid::new_v4(), name.clone(), None);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().name.chars().count(), 120);
    }

    #[test]
    fn edge_name_trimmed_before_validation() {
        let p =
            Portfolio::new(Uuid::new_v4(), "   Trimmed Portfolio   ".to_string(), None).unwrap();
        assert_eq!(p.name, "Trimmed Portfolio");
    }

    #[test]
    fn edge_description_whitespace_only_becomes_none() {
        let p =
            Portfolio::new(Uuid::new_v4(), "Name".to_string(), Some("   ".to_string())).unwrap();
        assert!(p.description.is_none());
    }

    #[test]
    fn edge_description_max_1000_chars_accepted() {
        let d = "x".repeat(1000);
        let p = Portfolio::new(Uuid::new_v4(), "Name".to_string(), Some(d));
        assert!(p.is_ok());
    }

    // ----- @security ----------------------------------------------------------

    // L'agrégat lui-même ne porte pas la logique RBAC (qui vit dans les
    // use-cases — `portfolio_use_cases.rs`). On s'assure cependant que
    // l'invariant structurel : `owner_user_id` est REQUIS — pas de
    // fallback "current user" implicite.
    #[test]
    fn security_owner_user_id_is_required_to_be_explicit() {
        // Compile-time guarantee : la signature impose `Uuid`,
        // pas de fallback "current user" implicite.
        let _: fn(Uuid, String, Option<String>) -> Result<Portfolio, PortfolioError> =
            Portfolio::new;
    }

    // ----- @negative ----------------------------------------------------------

    #[test]
    fn negative_empty_name_is_rejected() {
        let err = Portfolio::new(Uuid::new_v4(), "".to_string(), None).unwrap_err();
        assert_eq!(err, PortfolioError::NameEmpty);
    }

    #[test]
    fn negative_whitespace_only_name_is_rejected_as_empty() {
        let err = Portfolio::new(Uuid::new_v4(), "     ".to_string(), None).unwrap_err();
        assert_eq!(err, PortfolioError::NameEmpty);
    }

    #[test]
    fn negative_single_char_name_is_too_short() {
        let err = Portfolio::new(Uuid::new_v4(), "A".to_string(), None).unwrap_err();
        assert_eq!(err, PortfolioError::NameTooShort(1));
    }

    #[test]
    fn negative_too_long_name_is_rejected() {
        let name = "B".repeat(121);
        let err = Portfolio::new(Uuid::new_v4(), name, None).unwrap_err();
        assert_eq!(err, PortfolioError::NameTooLong(121));
    }

    #[test]
    fn negative_too_long_description_is_rejected() {
        let d = "x".repeat(1001);
        let err = Portfolio::new(Uuid::new_v4(), "Name".to_string(), Some(d)).unwrap_err();
        assert_eq!(err, PortfolioError::DescriptionTooLong(1001));
    }

    #[test]
    fn negative_update_info_re_validates_invariants() {
        let mut p = Portfolio::new(Uuid::new_v4(), "Valid".to_string(), None).unwrap();
        let err = p.update_info("".to_string(), None).unwrap_err();
        assert_eq!(err, PortfolioError::NameEmpty);
        // Name unchanged because update failed.
        assert_eq!(p.name, "Valid");
    }
}
