use crate::application::dto::{
    Disable2FADto, Enable2FADto, RegenerateBackupCodesDto, Verify2FADto,
};
use crate::application::use_cases::TwoFactorUseCases;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

/// Setup 2FA for a user (returns QR code + backup codes)
///
/// This endpoint initiates 2FA setup by generating a TOTP secret, QR code, and backup codes.
/// The user must then verify a TOTP code via `POST /2fa/enable` to activate 2FA.
///
/// # Security
/// - User must be authenticated
/// - Secret is only returned once during setup
/// - Backup codes are only shown once (user must save them)
///
/// # Returns
/// - 200 OK: Setup successful with QR code and backup codes
/// - 400 Bad Request: 2FA already enabled
/// - 401 Unauthorized: Not authenticated
/// - 500 Internal Server Error: Setup failed
///
/// # Example Response
/// ```json
/// {
///   "secret": "JBSWY3DPEHPK3PXP...",
///   "qr_code_data_url": "data:image/png;base64,...",
///   "backup_codes": ["ABCD-EFGH", "IJKL-MNOP", ...],
///   "issuer": "KoproGo",
///   "account_name": "user@example.com"
/// }
/// ```
pub async fn setup_2fa(
    auth: AuthenticatedUser,
    use_cases: web::Data<Arc<TwoFactorUseCases>>,
) -> HttpResponse {
    let organization_id = match auth.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Organization ID is required"
        })),
    };

    match use_cases
        .setup_2fa(auth.user_id, organization_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) if e.contains("already enabled") => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }))
        }
        Err(e) => {
            log::error!("Failed to setup 2FA for user {}: {}", auth.user_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to setup 2FA"
            }))
        }
    }
}

/// Enable 2FA after verifying TOTP code
///
/// After setup, the user must verify their TOTP code from their authenticator app
/// to enable 2FA. This confirms they have successfully saved the secret.
///
/// # Security
/// - User must be authenticated
/// - Requires valid 6-digit TOTP code
/// - Failed attempts are logged for security monitoring
///
/// # Request Body
/// ```json
/// {
///   "totp_code": "123456"
/// }
/// ```
///
/// # Returns
/// - 200 OK: 2FA successfully enabled
/// - 400 Bad Request: Invalid TOTP code or already enabled
/// - 401 Unauthorized: Not authenticated
/// - 500 Internal Server Error: Enable failed
pub async fn enable_2fa(
    auth: AuthenticatedUser,
    dto: web::Json<Enable2FADto>,
    use_cases: web::Data<Arc<TwoFactorUseCases>>,
) -> HttpResponse {
    let organization_id = match auth.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Organization ID is required"
        })),
    };

    match use_cases
        .enable_2fa(auth.user_id, organization_id, dto.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) if e.contains("Invalid TOTP") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Invalid TOTP code. Please check your authenticator app and try again."
            }),
        ),
        Err(e) if e.contains("already enabled") => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }))
        }
        Err(e) if e.contains("not found") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "2FA setup not found. Please run setup first."
            }),
        ),
        Err(e) => {
            log::error!("Failed to enable 2FA for user {}: {}", auth.user_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to enable 2FA"
            }))
        }
    }
}

/// Verify 2FA code during login
///
/// Validates a TOTP code or backup code during login. This endpoint is called after
/// successful password authentication when 2FA is enabled for the user.
///
/// # Security
/// - User must be authenticated (pre-2FA session)
/// - Accepts 6-digit TOTP code OR 8-character backup code
/// - Backup codes are one-time use (removed after verification)
/// - Failed attempts are logged and rate-limited
///
/// # Request Body
/// ```json
/// {
///   "totp_code": "123456"  // Or backup code like "ABCD-EFGH"
/// }
/// ```
///
/// # Returns
/// - 200 OK: Verification successful
/// - 400 Bad Request: Invalid code
/// - 401 Unauthorized: Not authenticated
/// - 429 Too Many Requests: Rate limit exceeded (3 attempts per 5 min)
/// - 500 Internal Server Error: Verification failed
pub async fn verify_2fa(
    auth: AuthenticatedUser,
    dto: web::Json<Verify2FADto>,
    use_cases: web::Data<Arc<TwoFactorUseCases>>,
) -> HttpResponse {
    let organization_id = match auth.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Organization ID is required"
        })),
    };

    match use_cases
        .verify_2fa(auth.user_id, organization_id, dto.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) if e.contains("Invalid TOTP") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Invalid code. Please try again or use a backup code."
            }),
        ),
        Err(e) if e.contains("not enabled") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "2FA is not enabled for this account"
            }),
        ),
        Err(e) => {
            log::error!("Failed to verify 2FA for user {}: {}", auth.user_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to verify 2FA"
            }))
        }
    }
}

/// Disable 2FA (requires current password)
///
/// Disables 2FA for the authenticated user. Requires password verification for security.
///
/// # Security
/// - User must be authenticated
/// - Requires current password verification
/// - All 2FA configuration is deleted (secret + backup codes)
/// - Action is logged for audit trail
///
/// # Request Body
/// ```json
/// {
///   "current_password": "user_password"
/// }
/// ```
///
/// # Returns
/// - 200 OK: 2FA successfully disabled
/// - 400 Bad Request: Invalid password
/// - 401 Unauthorized: Not authenticated
/// - 500 Internal Server Error: Disable failed
pub async fn disable_2fa(
    auth: AuthenticatedUser,
    dto: web::Json<Disable2FADto>,
    use_cases: web::Data<Arc<TwoFactorUseCases>>,
) -> HttpResponse {
    let organization_id = match auth.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Organization ID is required"
        })),
    };

    match use_cases
        .disable_2fa(auth.user_id, organization_id, dto.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "2FA successfully disabled"
        })),
        Err(e) if e.contains("Invalid password") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Invalid password. Please verify your password and try again."
            }),
        ),
        Err(e) => {
            log::error!("Failed to disable 2FA for user {}: {}", auth.user_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to disable 2FA"
            }))
        }
    }
}

/// Regenerate backup codes (requires TOTP verification)
///
/// Generates a new set of 10 backup codes, replacing the old ones.
/// Requires TOTP verification for security.
///
/// # Security
/// - User must be authenticated
/// - Requires valid 6-digit TOTP code
/// - Old backup codes are invalidated
/// - New codes are only shown once (user must save them)
///
/// # Request Body
/// ```json
/// {
///   "totp_code": "123456"
/// }
/// ```
///
/// # Returns
/// - 200 OK: Backup codes regenerated
/// - 400 Bad Request: Invalid TOTP code or 2FA not enabled
/// - 401 Unauthorized: Not authenticated
/// - 500 Internal Server Error: Regeneration failed
///
/// # Example Response
/// ```json
/// {
///   "backup_codes": ["ABCD-EFGH", "IJKL-MNOP", ...],
///   "regenerated_at": "2024-12-02T12:00:00Z"
/// }
/// ```
pub async fn regenerate_backup_codes(
    auth: AuthenticatedUser,
    dto: web::Json<RegenerateBackupCodesDto>,
    use_cases: web::Data<Arc<TwoFactorUseCases>>,
) -> HttpResponse {
    let organization_id = match auth.organization_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Organization ID is required"
        })),
    };

    match use_cases
        .regenerate_backup_codes(auth.user_id, organization_id, dto.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) if e.contains("Invalid TOTP") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Invalid TOTP code. Please check your authenticator app and try again."
            }),
        ),
        Err(e) if e.contains("not enabled") => HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "2FA is not enabled for this account"
            }),
        ),
        Err(e) => {
            log::error!(
                "Failed to regenerate backup codes for user {}: {}",
                auth.user_id,
                e
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to regenerate backup codes"
            }))
        }
    }
}

/// Get 2FA status for the authenticated user
///
/// Returns the current 2FA configuration status, including:
/// - Whether 2FA is enabled
/// - Number of backup codes remaining
/// - Whether backup codes are low (< 3)
/// - Whether reverification is needed (not used in 90 days)
///
/// # Security
/// - User must be authenticated
/// - Only returns user's own 2FA status
///
/// # Returns
/// - 200 OK: Status retrieved successfully
/// - 401 Unauthorized: Not authenticated
/// - 500 Internal Server Error: Failed to retrieve status
///
/// # Example Response
/// ```json
/// {
///   "is_enabled": true,
///   "verified_at": "2024-11-01T10:00:00Z",
///   "last_used_at": "2024-12-01T08:30:00Z",
///   "backup_codes_remaining": 7,
///   "backup_codes_low": false,
///   "needs_reverification": false
/// }
/// ```
pub async fn get_2fa_status(
    auth: AuthenticatedUser,
    use_cases: web::Data<Arc<TwoFactorUseCases>>,
) -> HttpResponse {
    match use_cases.get_2fa_status(auth.user_id).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            log::error!("Failed to get 2FA status for user {}: {}", auth.user_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve 2FA status"
            }))
        }
    }
}

/// Configure 2FA routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/2fa")
            .route("/setup", web::post().to(setup_2fa))
            .route("/enable", web::post().to(enable_2fa))
            .route("/verify", web::post().to(verify_2fa))
            .route("/disable", web::post().to(disable_2fa))
            .route("/regenerate-backup-codes", web::post().to(regenerate_backup_codes))
            .route("/status", web::get().to(get_2fa_status)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{TwoFactorRepository, UserRepository};
    use crate::domain::entities::User;
    use actix_web::{test, web, App};
    use mockall::predicate::*;
    use mockall::mock;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock repositories
    mock! {
        TwoFactorRepo {}
        #[async_trait::async_trait]
        impl TwoFactorRepository for TwoFactorRepo {
            async fn create(&self, secret: &crate::domain::entities::TwoFactorSecret) -> Result<crate::domain::entities::TwoFactorSecret, String>;
            async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<crate::domain::entities::TwoFactorSecret>, String>;
            async fn update(&self, secret: &crate::domain::entities::TwoFactorSecret) -> Result<crate::domain::entities::TwoFactorSecret, String>;
            async fn delete(&self, user_id: Uuid) -> Result<(), String>;
            async fn find_needing_reverification(&self) -> Result<Vec<crate::domain::entities::TwoFactorSecret>, String>;
            async fn find_with_low_backup_codes(&self) -> Result<Vec<crate::domain::entities::TwoFactorSecret>, String>;
        }
    }

    mock! {
        UserRepo {}
        #[async_trait::async_trait]
        impl UserRepository for UserRepo {
            async fn create(&self, user: &User) -> Result<User, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
            async fn find_all(&self) -> Result<Vec<User>, String>;
            async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>;
            async fn update(&self, user: &User) -> Result<User, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String>;
        }
    }

    #[actix_web::test]
    async fn test_get_2fa_status_not_enabled() {
        let two_factor_repo = Arc::new(MockTwoFactorRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let encryption_key = [0u8; 32]; // Dummy encryption key for testing

        let use_cases = Arc::new(TwoFactorUseCases::new(two_factor_repo, user_repo, encryption_key));

        let _app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_cases))
                .configure(configure_routes),
        )
        .await;

        // TODO: Add authentication middleware mock
        // For now, this test is incomplete due to auth requirements
    }

    // Additional tests would require mocking the authentication middleware
    // and the repository responses. This is left as a TODO for integration tests.
}
