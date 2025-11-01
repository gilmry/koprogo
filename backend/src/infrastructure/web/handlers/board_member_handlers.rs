use crate::application::dto::{CreateBoardMemberDto, RenewMandateDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// Élire un nouveau membre du conseil de copropriété
#[post("/board-members")]
pub async fn elect_board_member(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateBoardMemberDto>,
) -> impl Responder {
    // SuperAdmin can elect members for any organization, others need to belong to an organization
    let organization_id = if user.role == "superadmin" {
        // For superadmin, get organization_id from the building
        None // Will be determined by the use case
    } else {
        match user.require_organization() {
            Ok(org_id) => Some(org_id),
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    };

    match state
        .board_member_use_cases
        .elect_board_member(request.into_inner())
        .await
    {
        Ok(member) => {
            // Audit log: successful board member election
            if let Ok(member_uuid) = Uuid::parse_str(&member.id) {
                AuditLogEntry::new(
                    AuditEventType::BoardMemberElected,
                    Some(user.user_id),
                    organization_id,
                )
                .with_resource("BoardMember", member_uuid)
                .log();
            }

            HttpResponse::Created().json(member)
        }
        Err(err) => {
            // Audit log: failed board member election
            AuditLogEntry::new(
                AuditEventType::BoardMemberElected,
                Some(user.user_id),
                organization_id,
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Récupérer un membre du conseil par ID
#[get("/board-members/{id}")]
pub async fn get_board_member(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.board_member_use_cases.get_board_member(*id).await {
        Ok(Some(member)) => HttpResponse::Ok().json(member),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Board member not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Lister tous les membres actifs du conseil pour un immeuble
#[get("/buildings/{building_id}/board-members/active")]
pub async fn list_active_board_members(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    // SuperAdmin can access all buildings, others need to belong to an organization
    if user.role != "superadmin" {
        if let Err(e) = user.require_organization() {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }

    match state
        .board_member_use_cases
        .list_active_board_members(*building_id)
        .await
    {
        Ok(members) => HttpResponse::Ok().json(members),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Lister tous les membres du conseil (actifs et historique) pour un immeuble
#[get("/buildings/{building_id}/board-members")]
pub async fn list_all_board_members(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    // SuperAdmin can access all buildings, others need to belong to an organization
    if user.role != "superadmin" {
        if let Err(e) = user.require_organization() {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }

    match state
        .board_member_use_cases
        .list_all_board_members(*building_id)
        .await
    {
        Ok(members) => HttpResponse::Ok().json(members),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Renouveler le mandat d'un membre du conseil
#[put("/board-members/{id}/renew")]
pub async fn renew_mandate(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<RenewMandateDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_member_use_cases
        .renew_mandate(*id, request.into_inner())
        .await
    {
        Ok(member) => {
            // Audit log: successful mandate renewal
            if let Ok(member_uuid) = Uuid::parse_str(&member.id) {
                AuditLogEntry::new(
                    AuditEventType::BoardMemberMandateRenewed,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("BoardMember", member_uuid)
                .log();
            }

            HttpResponse::Ok().json(member)
        }
        Err(err) => {
            // Audit log: failed mandate renewal
            AuditLogEntry::new(
                AuditEventType::BoardMemberMandateRenewed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Retirer un membre du conseil (fin de mandat anticipée)
#[delete("/board-members/{id}")]
pub async fn remove_board_member(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.board_member_use_cases.remove_board_member(*id).await {
        Ok(true) => {
            // Audit log: successful board member removal
            AuditLogEntry::new(
                AuditEventType::BoardMemberRemoved,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("BoardMember", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Board member not found"
        })),
        Err(err) => {
            // Audit log: failed board member removal
            AuditLogEntry::new(
                AuditEventType::BoardMemberRemoved,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Obtenir des statistiques sur le conseil d'un immeuble
#[get("/buildings/{building_id}/board-members/stats")]
pub async fn get_board_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_member_use_cases
        .get_board_stats(*building_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Obtenir le tableau de bord d'un membre du conseil
/// Accessible uniquement aux membres du conseil et superadmins
#[get("/board-members/dashboard")]
pub async fn get_board_dashboard(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // SuperAdmin can access dashboard for any building, others need to belong to an organization
    let _organization_id = if user.role != "superadmin" {
        match user.require_organization() {
            Ok(org_id) => Some(org_id),
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    } else {
        None
    };

    // Get building_id from query parameters
    let building_id = match query.get("building_id") {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid building_id format"
                }))
            }
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "building_id query parameter is required"
            }))
        }
    };

    // Get owner_id from user->owner link
    let owner_id_result =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM owners WHERE user_id = $1 LIMIT 1")
            .bind(user.user_id)
            .fetch_optional(&state.pool)
            .await;

    let owner_id = match owner_id_result {
        Ok(Some(oid)) => oid,
        Ok(None) => {
            // User is not linked to an owner
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "User is not linked to an owner. Board dashboard is only accessible to board members."
            }));
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", err)
            }));
        }
    };

    // Authorization: Verify user is an active board member for this building (unless superadmin)
    let is_superadmin = user.role == "superadmin";
    if !is_superadmin {
        match state
            .board_member_use_cases
            .has_active_board_mandate(owner_id, building_id)
            .await
        {
            Ok(true) => {
                // User is an active board member, proceed
            }
            Ok(false) => {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Access denied. You are not an active board member for this building."
                }));
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Authorization check failed: {}", err)
                }));
            }
        }
    }

    match state
        .board_dashboard_use_cases
        .get_dashboard(building_id, owner_id)
        .await
    {
        Ok(dashboard) => HttpResponse::Ok().json(dashboard),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// GET /board-members/my-mandates - Get all active board mandates for the authenticated user
#[get("/board-members/my-mandates")]
pub async fn get_my_mandates(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    // Get the owner ID for this user
    let owner_id = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM owners WHERE user_id = $1 AND organization_id = $2 AND is_anonymized = false"
    )
    .bind(user.user_id)
    .bind(organization_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            // User is not linked to an owner - return empty list
            return HttpResponse::Ok().json(serde_json::json!({
                "mandates": []
            }));
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch owner: {}", err)
            }));
        }
    };

    // Get all active board member mandates for this owner
    match sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            String,
        ),
    >(
        r#"
        SELECT
            bm.id,
            bm.building_id,
            bm.position::TEXT,
            b.name as building_name,
            bm.mandate_start,
            bm.mandate_end,
            b.address
        FROM board_members bm
        JOIN buildings b ON b.id = bm.building_id
        WHERE bm.owner_id = $1
          AND bm.organization_id = $2
          AND bm.is_active = true
        ORDER BY bm.mandate_end DESC
        "#,
    )
    .bind(owner_id)
    .bind(organization_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => {
            let mandates: Vec<serde_json::Value> = rows
                .into_iter()
                .map(
                    |(
                        id,
                        building_id,
                        position,
                        building_name,
                        mandate_start,
                        mandate_end,
                        address,
                    )| {
                        let now = chrono::Utc::now();
                        let days_remaining = (mandate_end - now).num_days();
                        let expires_soon = days_remaining > 0 && days_remaining <= 90;

                        serde_json::json!({
                            "id": id,
                            "building_id": building_id,
                            "building_name": building_name,
                            "building_address": address,
                            "position": position,
                            "mandate_start": mandate_start.format("%Y-%m-%d").to_string(),
                            "mandate_end": mandate_end.format("%Y-%m-%d").to_string(),
                            "days_remaining": days_remaining,
                            "expires_soon": expires_soon,
                        })
                    },
                )
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "mandates": mandates
            }))
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch mandates: {}", err)
        })),
    }
}
