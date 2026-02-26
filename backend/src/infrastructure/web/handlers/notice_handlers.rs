use crate::application::dto::{CreateNoticeDto, SetExpirationDto, UpdateNoticeDto};
use crate::domain::entities::{NoticeCategory, NoticeStatus, NoticeType};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// Create a new notice (Draft status)
///
/// POST /notices
#[post("/notices")]
pub async fn create_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateNoticeDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .notice_use_cases
        .create_notice(auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(notice) => HttpResponse::Created().json(notice),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Get notice by ID with author name enrichment
///
/// GET /notices/:id
#[get("/notices/{id}")]
pub async fn get_notice(data: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match data.notice_use_cases.get_notice(id.into_inner()).await {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// List all notices for a building (all statuses)
///
/// GET /buildings/:building_id/notices
#[get("/buildings/{building_id}/notices")]
pub async fn list_building_notices(
    data: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .list_building_notices(building_id.into_inner())
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List published notices for a building (visible to members)
///
/// GET /buildings/:building_id/notices/published
#[get("/buildings/{building_id}/notices/published")]
pub async fn list_published_notices(
    data: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .list_published_notices(building_id.into_inner())
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List pinned notices for a building (important announcements)
///
/// GET /buildings/:building_id/notices/pinned
#[get("/buildings/{building_id}/notices/pinned")]
pub async fn list_pinned_notices(
    data: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .list_pinned_notices(building_id.into_inner())
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List notices by type (Announcement, Event, LostAndFound, ClassifiedAd)
///
/// GET /buildings/:building_id/notices/type/:notice_type
#[get("/buildings/{building_id}/notices/type/{notice_type}")]
pub async fn list_notices_by_type(
    data: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, notice_type_str) = path.into_inner();

    // Parse notice type
    let notice_type = match serde_json::from_str::<NoticeType>(&format!("\"{}\"", notice_type_str))
    {
        Ok(nt) => nt,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid notice type: {}. Valid types: Announcement, Event, LostAndFound, ClassifiedAd", notice_type_str)
            }))
        }
    };

    match data
        .notice_use_cases
        .list_notices_by_type(building_id, notice_type)
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List notices by category (General, Maintenance, Social, etc.)
///
/// GET /buildings/:building_id/notices/category/:category
#[get("/buildings/{building_id}/notices/category/{category}")]
pub async fn list_notices_by_category(
    data: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, category_str) = path.into_inner();

    // Parse category
    let category = match serde_json::from_str::<NoticeCategory>(&format!("\"{}\"", category_str)) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid category: {}. Valid categories: General, Maintenance, Social, Security, Environment, Parking, Other", category_str)
            }))
        }
    };

    match data
        .notice_use_cases
        .list_notices_by_category(building_id, category)
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List notices by status (Draft, Published, Archived, Expired)
///
/// GET /buildings/:building_id/notices/status/:status
#[get("/buildings/{building_id}/notices/status/{status}")]
pub async fn list_notices_by_status(
    data: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, status_str) = path.into_inner();

    // Parse status
    let status = match serde_json::from_str::<NoticeStatus>(&format!("\"{}\"", status_str)) {
        Ok(s) => s,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid status: {}. Valid statuses: Draft, Published, Archived, Expired", status_str)
            }))
        }
    };

    match data
        .notice_use_cases
        .list_notices_by_status(building_id, status)
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List all notices created by an author
///
/// GET /owners/:author_id/notices
#[get("/owners/{author_id}/notices")]
pub async fn list_author_notices(
    data: web::Data<AppState>,
    author_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .list_author_notices(author_id.into_inner())
        .await
    {
        Ok(notices) => HttpResponse::Ok().json(notices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Update a notice (Draft only)
///
/// PUT /notices/:id
#[put("/notices/{id}")]
pub async fn update_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateNoticeDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .notice_use_cases
        .update_notice(id.into_inner(), auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Publish a notice (Draft → Published)
///
/// POST /notices/:id/publish
#[post("/notices/{id}/publish")]
pub async fn publish_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .notice_use_cases
        .publish_notice(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Archive a notice (Published/Expired → Archived)
///
/// POST /notices/:id/archive
#[post("/notices/{id}/archive")]
pub async fn archive_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .notice_use_cases
        .archive_notice(id.into_inner(), auth.user_id, org_id, &auth.role)
        .await
    {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Pin a notice to top of board (Published only)
///
/// POST /notices/:id/pin
#[post("/notices/{id}/pin")]
pub async fn pin_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .pin_notice(id.into_inner(), &auth.role)
        .await
    {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Unpin a notice
///
/// POST /notices/:id/unpin
#[post("/notices/{id}/unpin")]
pub async fn unpin_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .unpin_notice(id.into_inner(), &auth.role)
        .await
    {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Set expiration date for a notice
///
/// PUT /notices/:id/expiration
#[put("/notices/{id}/expiration")]
pub async fn set_expiration(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<SetExpirationDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .notice_use_cases
        .set_expiration(id.into_inner(), auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(notice) => HttpResponse::Ok().json(notice),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Delete a notice
///
/// DELETE /notices/:id
#[delete("/notices/{id}")]
pub async fn delete_notice(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .notice_use_cases
        .delete_notice(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Get notice statistics for a building
///
/// GET /buildings/:building_id/notices/statistics
#[get("/buildings/{building_id}/notices/statistics")]
pub async fn get_notice_statistics(
    data: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .notice_use_cases
        .get_statistics(building_id.into_inner())
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
