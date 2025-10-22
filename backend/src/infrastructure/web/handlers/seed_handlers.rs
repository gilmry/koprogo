use crate::infrastructure::database::DatabaseSeeder;
use crate::infrastructure::web::AppState;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

/// Seed demo data (SuperAdmin only)
#[post("/seed/demo")]
pub async fn seed_demo_data(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    // Extract and verify token
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

    // Verify token and check role
    match data.auth_use_cases.verify_token(token) {
        Ok(claims) => {
            // Only SuperAdmin can seed data
            if claims.role != "superadmin" {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Only SuperAdmin can seed demo data"
                }));
            }

            // Create seeder
            let seeder = DatabaseSeeder::new(data.pool.clone());

            // Seed demo data
            match seeder.seed_demo_data().await {
                Ok(message) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": message
                })),
                Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                })),
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Clear demo data (SuperAdmin only)
#[post("/seed/clear")]
pub async fn clear_demo_data(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
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
        Ok(claims) => {
            if claims.role != "superadmin" {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Only SuperAdmin can clear demo data"
                }));
            }

            let seeder = DatabaseSeeder::new(data.pool.clone());

            match seeder.clear_demo_data().await {
                Ok(message) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": message
                })),
                Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                })),
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e
        })),
    }
}
