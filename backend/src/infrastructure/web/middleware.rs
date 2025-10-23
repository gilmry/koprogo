use crate::infrastructure::web::app_state::AppState;
use actix_web::{
    dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest,
};
use std::future::{ready, Ready};
use uuid::Uuid;

/// Authenticated user claims extracted from JWT token
///
/// This struct automatically extracts and validates JWT tokens from the Authorization header.
/// Use it as a parameter in your handler functions to require authentication:
///
/// ```rust
/// async fn protected_handler(claims: AuthenticatedUser) -> impl Responder {
///     // claims.user_id and claims.organization_id are now available
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub organization_id: Option<Uuid>,
}

impl AuthenticatedUser {
    /// Get the organization_id or return an error if not present
    pub fn require_organization(&self) -> Result<Uuid, Error> {
        self.organization_id
            .ok_or_else(|| ErrorUnauthorized("User does not belong to an organization"))
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Get AppState from request
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(state) => state,
            None => return ready(Err(ErrorUnauthorized("Internal server error"))),
        };

        // Extract Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => match header.to_str() {
                Ok(s) => s,
                Err(_) => return ready(Err(ErrorUnauthorized("Invalid authorization header"))),
            },
            None => return ready(Err(ErrorUnauthorized("Missing authorization header"))),
        };

        // Extract token from "Bearer <token>"
        let token = auth_header.trim_start_matches("Bearer ").trim();

        // Verify token and extract claims
        match app_state.auth_use_cases.verify_token(token) {
            Ok(claims) => {
                // Parse user_id from claims.sub
                match Uuid::parse_str(&claims.sub) {
                    Ok(user_id) => ready(Ok(AuthenticatedUser {
                        user_id,
                        email: claims.email,
                        role: claims.role,
                        organization_id: claims.organization_id,
                    })),
                    Err(_) => ready(Err(ErrorUnauthorized("Invalid user ID in token"))),
                }
            }
            Err(e) => ready(Err(ErrorUnauthorized(e))),
        }
    }
}

/// Organization ID extracted from authenticated user's JWT token
///
/// This extractor requires that the user belongs to an organization.
/// Use it when you need to enforce organization-scoped operations:
///
/// ```rust
/// async fn create_building(
///     organization: OrganizationId,
///     dto: web::Json<CreateBuildingDto>
/// ) -> impl Responder {
///     // organization.0 contains the Uuid
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct OrganizationId(pub Uuid);

impl FromRequest for OrganizationId {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // First extract AuthenticatedUser
        let user_future = AuthenticatedUser::from_request(req, payload);

        // Get the result
        match user_future.into_inner() {
            Ok(user) => match user.organization_id {
                Some(org_id) => ready(Ok(OrganizationId(org_id))),
                None => ready(Err(ErrorUnauthorized(
                    "User does not belong to an organization",
                ))),
            },
            Err(e) => ready(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user_require_organization() {
        let user_with_org = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "admin".to_string(),
            organization_id: Some(Uuid::new_v4()),
        };

        assert!(user_with_org.require_organization().is_ok());

        let user_without_org = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "admin".to_string(),
            organization_id: None,
        };

        assert!(user_without_org.require_organization().is_err());
    }
}
