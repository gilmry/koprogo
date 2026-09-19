// DTOs for Account API
//
// CREDITS: Structure inspired by Noalyss API patterns (GPL-2.0+)
// https://gitlab.com/noalyss/noalyss

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request DTO for creating a new account
///
/// `deny_unknown_fields` n'est pas décoratif : sans lui, un champ mal nommé
/// est accepté puis JETÉ en silence — la requête rend 201 et la donnée
/// n'existe pas. Sur un compte du plan comptable, ce serait une écriture
/// qu'on croit passée.
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct CreateAccountDto {
    #[validate(length(min = 1, max = 40, message = "Account code must be 1-40 characters"))]
    pub code: String,

    #[validate(length(min = 1, max = 255, message = "Account label must be 1-255 characters"))]
    pub label: String,

    pub parent_code: Option<String>,

    pub account_type: String, // "ASSET", "LIABILITY", "EXPENSE", "REVENUE", "OFF_BALANCE"

    pub direct_use: bool,

    pub organization_id: String,
}

/// Request DTO for updating an existing account
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct UpdateAccountDto {
    #[validate(length(min = 1, max = 255))]
    pub label: Option<String>,

    pub parent_code: Option<Option<String>>,

    pub account_type: Option<String>, // "ASSET", "LIABILITY", "EXPENSE", "REVENUE", "OFF_BALANCE"

    pub direct_use: Option<bool>,
}

/// Response DTO for account data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountResponseDto {
    pub id: String,
    pub code: String,
    pub label: String,
    pub parent_code: Option<String>,
    pub account_type: String, // "ASSET", "LIABILITY", "EXPENSE", "REVENUE", "OFF_BALANCE"
    pub direct_use: bool,
    pub organization_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request DTO for seeding Belgian PCMN
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SeedBelgianPcmnDto {
    pub organization_id: String,
}

/// Response DTO for seed operation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeedPcmnResponseDto {
    pub accounts_created: i64,
    pub message: String,
}

/// Query parameters for searching accounts
#[derive(Debug, Deserialize, Clone)]
pub struct AccountSearchQuery {
    pub code_pattern: Option<String>,
    pub account_type: Option<String>,
    pub direct_use_only: Option<bool>,
    pub parent_code: Option<String>,
}
