use crate::application::dto::{
    LoginRequest, RefreshTokenRequest, RegisterRequest, SwitchRoleRequest,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use validator::Validate;

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    summary = "Login",
    request_body = LoginRequest,
    responses(
        (status = 201, description = "Resource created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[post("/auth/login")]
pub async fn login(data: web::Data<AppState>, request: web::Json<LoginRequest>) -> impl Responder {
    // Validate request
    if let Err(errors) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match data.auth_use_cases.login(request.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e
        })),
    }
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    summary = "Register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Resource created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[post("/auth/register")]
pub async fn register(
    data: web::Data<AppState>,
    request: web::Json<RegisterRequest>,
) -> impl Responder {
    // Validate request
    if let Err(errors) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match data.auth_use_cases.register(request.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "Auth",
    summary = "Get Current User",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[get("/auth/me")]
pub async fn get_current_user(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    // Extract Authorization header
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid authorization header"
                }))
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Missing authorization header"
            }))
        }
    };

    let token = auth_header.trim_start_matches("Bearer ").trim();

    match data.auth_use_cases.verify_token(token) {
        Ok(claims) => match uuid::Uuid::parse_str(&claims.sub) {
            Ok(user_id) => match data.auth_use_cases.get_user_by_id(user_id).await {
                Ok(user) => HttpResponse::Ok().json(user),
                Err(e) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                })),
            },
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid user ID: {}", e)
            })),
        },
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e
        })),
    }
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Auth",
    summary = "Refresh Token",
    request_body = RefreshTokenRequest,
    responses(
        (status = 201, description = "Resource created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[post("/auth/refresh")]
pub async fn refresh_token(
    data: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    match data
        .auth_use_cases
        .refresh_token(request.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e
        })),
    }
}

#[utoipa::path(
    post,
    path = "/auth/switch-role",
    tag = "Auth",
    summary = "Switch Role",
    request_body = SwitchRoleRequest,
    responses(
        (status = 201, description = "Resource created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[post("/auth/switch-role")]
pub async fn switch_role(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<SwitchRoleRequest>,
) -> impl Responder {
    let payload = request.into_inner();

    match data
        .auth_use_cases
        .switch_active_role(user.user_id, payload.role_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}
