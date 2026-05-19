use crate::application::dto::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RegisterRequest, SwitchRoleRequest,
    UserResponse,
};
use crate::application::error::AppError;
use crate::infrastructure::web::{
    build_clearing_cookie, build_refresh_cookie, AppState, AuthenticatedUser, REFRESH_COOKIE_NAME,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use validator::Validate;

/// Corps de réponse d'authentification **sans** `refresh_token` (WP-FE1).
///
/// Le refresh token n'est plus exposé au JavaScript : il part exclusivement
/// dans le cookie `HttpOnly` (cf. `auth_cookie`). On ne modifie pas le DTO
/// interne `LoginResponse` (contrat use-case) — on projette ici la vue HTTP.
#[derive(Serialize)]
struct AuthBody {
    token: String,
    user: UserResponse,
}

impl From<LoginResponse> for AuthBody {
    fn from(r: LoginResponse) -> Self {
        Self {
            token: r.token,
            user: r.user,
        }
    }
}

/// Réponse 200 : pose le cookie refresh `HttpOnly` + corps sans refresh.
fn auth_response_with_cookie(resp: LoginResponse) -> HttpResponse {
    let cookie = build_refresh_cookie(&resp.refresh_token);
    HttpResponse::Ok().cookie(cookie).json(AuthBody::from(resp))
}

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
    Ok(auth_response_with_cookie(response))
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
        Ok(response) => {
            let cookie = build_refresh_cookie(&response.refresh_token);
            HttpResponse::Created()
                .cookie(cookie)
                .json(AuthBody::from(response))
        }
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
    description = "Le refresh token est lu depuis le cookie HttpOnly \
                   `koprogo_refresh` (WP-FE1) — aucun corps de requête. \
                   La réponse rote le cookie et ne contient pas de \
                   refresh_token.",
    responses(
        (status = 200, description = "Access token rafraîchi"),
        (status = 401, description = "Cookie refresh absent, expiré ou révoqué"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[post("/auth/refresh")]
pub async fn refresh_token(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Le refresh token vient EXCLUSIVEMENT du cookie HttpOnly : illisible
    // par JS, donc non exfiltrable par XSS (WP-FE1). Absence = 401.
    let refresh_token = req
        .cookie(REFRESH_COOKIE_NAME)
        .map(|c| c.value().to_owned())
        .filter(|v| !v.is_empty())
        .ok_or(AppError::Unauthorized)?;

    let response = data
        .auth_use_cases
        .refresh_token(RefreshTokenRequest { refresh_token })
        .await?;
    // Rotation : le use-case révoque l'ancien refresh et en émet un neuf.
    Ok(auth_response_with_cookie(response))
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
        Ok(response) => auth_response_with_cookie(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    summary = "Logout",
    description = "Révoque tous les refresh tokens de l'utilisateur \
                   (déconnexion serveur) et expire le cookie HttpOnly \
                   `koprogo_refresh` (WP-FE1).",
    responses(
        (status = 200, description = "Déconnecté, cookie expiré"),
        (status = 401, description = "Access token absent ou invalide"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[post("/auth/logout")]
pub async fn logout(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    // Révocation serveur : tous les refresh de l'utilisateur sont
    // invalidés (le cookie volé éventuel devient inutilisable), puis le
    // cookie est expiré côté navigateur.
    data.auth_use_cases
        .revoke_all_refresh_tokens(user.user_id)
        .await
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok()
        .cookie(build_clearing_cookie())
        .json(serde_json::json!({ "message": "logged out" })))
}
