use crate::application::dto::{LoginRequest, RefreshTokenRequest, RegisterRequest};
use crate::infrastructure::web::AppState;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use validator::Validate;

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

#[post("/auth/refresh")]
pub async fn refresh_token(
    data: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    match data.auth_use_cases.refresh_token(request.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e
        })),
    }
}
