use crate::application::dto::{
    LoginRequest, RefreshTokenRequest, RegisterRequest, SwitchRoleRequest,
};
use crate::application::error::AppError;
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
pub async fn login(
    data: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    request
        .validate()
        .map_err(|errors| AppError::Validation(errors.to_string()))?;

    let response = data.auth_use_cases.login(request.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
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
pub async fn get_current_user(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Validation("invalid authorization header".to_string()))?;

    let token = auth_header.trim_start_matches("Bearer ").trim();

    let claims = data.auth_use_cases.verify_token(token)?;

    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|e| AppError::Validation(format!("invalid user id in token: {}", e)))?;

    let user = data.auth_use_cases.get_user_by_id(user_id).await?;
    Ok(HttpResponse::Ok().json(user))
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
) -> Result<HttpResponse, AppError> {
    let response = data
        .auth_use_cases
        .refresh_token(request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
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
