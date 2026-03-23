//! API Key Management Handlers — Public API v2 (Issues #111, #232)
//!
//! Enables third-party integrations with KoproGo via API keys.
//! Supports: PropTech, notaries, energy providers, accounting software.

use actix_web::{web, HttpResponse, get, post, delete, put};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::infrastructure::web::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;

/// Available API v2 permissions
const VALID_PERMISSIONS: &[&str] = &[
    "read:buildings",
    "read:expenses",
    "read:owners",
    "read:meetings",
    "read:etats-dates",
    "write:etats-dates",
    "read:energy-campaigns",
    "read:documents",
    "read:financial-reports",
    "webhooks:subscribe",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub key: String,  // Full key — only shown ONCE at creation
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub warning: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyDto {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub data: Vec<ApiKeyDto>,
    pub total: i64,
}

/// Generate a random API key with secure hashing
fn generate_api_key() -> (String, String, String) {
    use sha2::{Sha256, Digest};

    // Generate 32 random bytes for the key body
    let random_bytes: Vec<u8> = (0..32)
        .map(|_| rand::random::<u8>())
        .collect();

    let key_body = hex::encode(&random_bytes);
    let full_key = format!("kpg_live_{}", key_body);
    let prefix = "kpg_live_".to_string();

    // Hash the key for secure storage
    let mut hasher = Sha256::new();
    hasher.update(full_key.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    (full_key, prefix, hash)
}

/// Create a new API key (Syndic or SuperAdmin only)
#[post("/api-keys")]
pub async fn create_api_key(
    claims: AuthenticatedUser,
    state: web::Data<Arc<AppState>>,
    body: web::Json<CreateApiKeyRequest>,
) -> HttpResponse {
    // Verify permissions
    if claims.role != "SYNDIC" && claims.role != "SUPERADMIN" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only syndics and admins can create API keys"
        }));
    }

    // Validate permissions
    for perm in &body.permissions {
        if !VALID_PERMISSIONS.contains(&perm.as_str()) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid permission: {}. Valid permissions: {:?}", perm, VALID_PERMISSIONS)
            }));
        }
    }

    let org_id = match claims.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "organization_id required"
        })),
    };

    // Validate name
    if body.name.is_empty() || body.name.len() > 255 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "API key name must be between 1 and 255 characters"
        }));
    }

    // Generate key
    let (full_key, prefix, hash) = generate_api_key();
    let key_id = Uuid::new_v4();
    let rate_limit = body.rate_limit.unwrap_or(100);

    // Ensure rate_limit is reasonable
    if rate_limit < 1 || rate_limit > 10000 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Rate limit must be between 1 and 10,000 requests per minute"
        }));
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO api_keys (id, organization_id, created_by, key_prefix, key_hash, name, description, permissions, rate_limit, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, created_at
        "#,
        key_id,
        org_id,
        claims.user_id,
        prefix,
        hash,
        body.name,
        body.description,
        &body.permissions,
        rate_limit,
        body.expires_at,
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => {
            // Log audit event
            let _ = sqlx::query!(
                r#"
                INSERT INTO api_key_audit (api_key_id, action, actor_id, reason)
                VALUES ($1, $2, $3, $4)
                "#,
                key_id,
                "created",
                claims.user_id,
                Some(format!("API key created by {}", claims.user_id))
            )
            .execute(&state.pool)
            .await;

            HttpResponse::Created().json(ApiKeyCreatedResponse {
                id: row.id,
                name: body.name.clone(),
                key: full_key,
                key_prefix: prefix,
                permissions: body.permissions.clone(),
                rate_limit,
                expires_at: body.expires_at,
                created_at: row.created_at,
                warning: "This key will never be displayed again. Store it securely.",
            })
        }
        Err(e) => {
            eprintln!("Database error creating API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create API key"
            }))
        }
    }
}

/// List API keys for organization (key bodies hidden)
#[get("/api-keys")]
pub async fn list_api_keys(
    claims: AuthenticatedUser,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let org_id = match claims.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "organization_id required"
        })),
    };

    let rows = sqlx::query!(
        r#"
        SELECT id, name, key_prefix, permissions, rate_limit, last_used_at, expires_at, is_active, created_at
        FROM api_keys
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
        org_id,
    )
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(keys) => {
            let dtos: Vec<ApiKeyDto> = keys
                .into_iter()
                .map(|row| ApiKeyDto {
                    id: row.id,
                    name: row.name,
                    key_prefix: row.key_prefix,
                    permissions: row.permissions,
                    rate_limit: row.rate_limit,
                    last_used_at: row.last_used_at,
                    expires_at: row.expires_at,
                    is_active: row.is_active,
                    created_at: row.created_at,
                })
                .collect();

            HttpResponse::Ok().json(ApiKeyListResponse {
                total: dtos.len() as i64,
                data: dtos,
            })
        }
        Err(e) => {
            eprintln!("Database error listing API keys: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list API keys"
            }))
        }
    }
}

/// Get a specific API key (hidden body)
#[get("/api-keys/{id}")]
pub async fn get_api_key(
    claims: AuthenticatedUser,
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let key_id = path.into_inner();
    let org_id = match claims.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "organization_id required"
        })),
    };

    let row = sqlx::query!(
        r#"
        SELECT id, name, key_prefix, permissions, rate_limit, last_used_at, expires_at, is_active, created_at
        FROM api_keys
        WHERE id = $1 AND organization_id = $2
        "#,
        key_id,
        org_id,
    )
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(key)) => HttpResponse::Ok().json(ApiKeyDto {
            id: key.id,
            name: key.name,
            key_prefix: key.key_prefix,
            permissions: key.permissions,
            rate_limit: key.rate_limit,
            last_used_at: key.last_used_at,
            expires_at: key.expires_at,
            is_active: key.is_active,
            created_at: key.created_at,
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "API key not found"
        })),
        Err(e) => {
            eprintln!("Database error fetching API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch API key"
            }))
        }
    }
}

/// Update an API key (name, description, rate limit, expiration)
#[put("/api-keys/{id}")]
pub async fn update_api_key(
    claims: AuthenticatedUser,
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateApiKeyRequest>,
) -> HttpResponse {
    let key_id = path.into_inner();
    let org_id = match claims.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "organization_id required"
        })),
    };

    // Verify authorization (only the creator or superadmin can update)
    let existing = sqlx::query!(
        "SELECT created_by FROM api_keys WHERE id = $1 AND organization_id = $2",
        key_id,
        org_id,
    )
    .fetch_optional(&state.pool)
    .await;

    match existing {
        Ok(Some(key)) => {
            if key.created_by != claims.user_id && claims.role != "SUPERADMIN" {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Only the API key creator can update it"
                }));
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "API key not found"
            }))
        }
        Err(e) => {
            eprintln!("Database error checking API key: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update API key"
            }));
        }
    }

    let result = sqlx::query!(
        r#"
        UPDATE api_keys
        SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            rate_limit = COALESCE($3, rate_limit),
            expires_at = COALESCE($4, expires_at),
            updated_at = NOW()
        WHERE id = $5 AND organization_id = $6
        RETURNING id, name, key_prefix, permissions, rate_limit, last_used_at, expires_at, is_active, created_at
        "#,
        body.name,
        body.description,
        body.rate_limit,
        body.expires_at,
        key_id,
        org_id,
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => {
            // Log audit event
            let _ = sqlx::query!(
                r#"
                INSERT INTO api_key_audit (api_key_id, action, actor_id, reason)
                VALUES ($1, $2, $3, $4)
                "#,
                key_id,
                "updated",
                claims.user_id,
                Some(format!("API key updated by {}", claims.user_id))
            )
            .execute(&state.pool)
            .await;

            HttpResponse::Ok().json(ApiKeyDto {
                id: row.id,
                name: row.name,
                key_prefix: row.key_prefix,
                permissions: row.permissions,
                rate_limit: row.rate_limit,
                last_used_at: row.last_used_at,
                expires_at: row.expires_at,
                is_active: row.is_active,
                created_at: row.created_at,
            })
        }
        Err(e) => {
            eprintln!("Database error updating API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update API key"
            }))
        }
    }
}

/// Revoke an API key (disable it)
#[delete("/api-keys/{id}")]
pub async fn revoke_api_key(
    claims: AuthenticatedUser,
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let key_id = path.into_inner();
    let org_id = match claims.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "organization_id required"
        })),
    };

    let result = sqlx::query!(
        "UPDATE api_keys SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND organization_id = $2",
        key_id,
        org_id,
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            // Log audit event
            let _ = sqlx::query!(
                r#"
                INSERT INTO api_key_audit (api_key_id, action, actor_id, reason)
                VALUES ($1, $2, $3, $4)
                "#,
                key_id,
                "revoked",
                claims.user_id,
                Some(format!("API key revoked by {}", claims.user_id))
            )
            .execute(&state.pool)
            .await;

            HttpResponse::Ok().json(ApiKeyResponse {
                success: true,
                message: "API key revoked successfully".to_string(),
            })
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "API key not found"
        })),
        Err(e) => {
            eprintln!("Database error revoking API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to revoke API key"
            }))
        }
    }
}

/// Rotate an API key (generate a new one, disable old one)
/// Note: This is a placeholder for future implementation
#[post("/api-keys/{id}/rotate")]
pub async fn rotate_api_key(
    claims: AuthenticatedUser,
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _key_id = path.into_inner();
    let _org_id = match claims.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "organization_id required"
        })),
    };

    // TODO: Implement key rotation
    // 1. Generate new key
    // 2. Mark old key as rotated
    // 3. Return new key (only shown once)

    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Key rotation not yet implemented"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let (full_key, prefix, hash) = generate_api_key();

        assert!(full_key.starts_with("kpg_live_"));
        assert_eq!(prefix, "kpg_live_");
        assert_eq!(hash.len(), 64); // SHA-256 is 64 hex chars
        assert!(full_key != generate_api_key().0); // Keys should be unique
    }

    #[test]
    fn test_validate_permissions() {
        let valid_perms = vec!["read:buildings".to_string(), "write:etats-dates".to_string()];
        for perm in valid_perms {
            assert!(VALID_PERMISSIONS.contains(&perm.as_str()));
        }

        let invalid_perm = "invalid:permission".to_string();
        assert!(!VALID_PERMISSIONS.contains(&invalid_perm.as_str()));
    }
}
