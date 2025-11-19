use crate::application::dto::{CreateAchievementDto, CreateChallengeDto, UpdateAchievementDto, UpdateChallengeDto};
use crate::domain::entities::{AchievementCategory, ChallengeStatus};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper function to check if user has admin role (for managing achievements/challenges)
///
/// Gamification management requires admin or superadmin privileges to prevent
/// abuse of the points/achievements system.
fn check_admin_role(user: &AuthenticatedUser) -> Option<HttpResponse> {
    if user.role != "admin" && user.role != "superadmin" {
        Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only admin or superadmin can manage achievements and challenges"
        })))
    } else {
        None
    }
}

// ============================================================================
// ACHIEVEMENT HANDLERS
// ============================================================================

/// Create a new achievement (admin only)
///
/// POST /achievements
///
/// # Request Body
/// - organization_id: UUID
/// - category: AchievementCategory
/// - tier: AchievementTier
/// - name: String (3-100 chars)
/// - description: String (10-500 chars)
/// - icon: String (emoji or URL)
/// - points_value: i32 (0-1000)
/// - requirements: String (JSON criteria)
/// - is_secret: bool
/// - is_repeatable: bool
/// - display_order: i32
///
/// # Responses
/// - 201 Created: Achievement created successfully
/// - 400 Bad Request: Validation error
#[post("/achievements")]
pub async fn create_achievement(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateAchievementDto>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.achievement_use_cases.create_achievement(request.into_inner()).await {
        Ok(achievement) => HttpResponse::Created().json(achievement),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Get achievement by ID
///
/// GET /achievements/:id
///
/// # Responses
/// - 200 OK: Achievement details
/// - 404 Not Found: Achievement not found
#[get("/achievements/{id}")]
pub async fn get_achievement(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.achievement_use_cases.get_achievement(id.into_inner()).await {
        Ok(achievement) => HttpResponse::Ok().json(achievement),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

/// List all achievements for an organization
///
/// GET /organizations/:organization_id/achievements
///
/// # Responses
/// - 200 OK: List of achievements
#[get("/organizations/{organization_id}/achievements")]
pub async fn list_achievements(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match data.achievement_use_cases.list_achievements(organization_id.into_inner()).await {
        Ok(achievements) => HttpResponse::Ok().json(achievements),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List achievements by category
///
/// GET /organizations/:organization_id/achievements/category/:category
///
/// # Responses
/// - 200 OK: List of achievements in category
#[get("/organizations/{organization_id}/achievements/category/{category}")]
pub async fn list_achievements_by_category(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (organization_id, category_str) = path.into_inner();

    // Parse category
    let category: AchievementCategory = match serde_json::from_str(&format!("\"{}\"", category_str)) {
        Ok(cat) => cat,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid category"})),
    };

    match data.achievement_use_cases.list_achievements_by_category(organization_id, category).await {
        Ok(achievements) => HttpResponse::Ok().json(achievements),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List visible achievements for current user
///
/// GET /organizations/:organization_id/achievements/visible
///
/// # Responses
/// - 200 OK: List of visible achievements (non-secret or already earned)
#[get("/organizations/{organization_id}/achievements/visible")]
pub async fn list_visible_achievements(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match data.achievement_use_cases.list_visible_achievements(organization_id.into_inner(), auth.user_id).await {
        Ok(achievements) => HttpResponse::Ok().json(achievements),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Update achievement (admin only)
///
/// PUT /achievements/:id
///
/// # Responses
/// - 200 OK: Achievement updated successfully
/// - 400 Bad Request: Validation error
/// - 404 Not Found: Achievement not found
#[put("/achievements/{id}")]
pub async fn update_achievement(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateAchievementDto>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.achievement_use_cases.update_achievement(id.into_inner(), request.into_inner()).await {
        Ok(achievement) => HttpResponse::Ok().json(achievement),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Delete achievement (admin only)
///
/// DELETE /achievements/:id
///
/// # Responses
/// - 204 No Content: Achievement deleted successfully
/// - 404 Not Found: Achievement not found
#[delete("/achievements/{id}")]
pub async fn delete_achievement(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.achievement_use_cases.delete_achievement(id.into_inner()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

// ============================================================================
// USER ACHIEVEMENT HANDLERS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AwardAchievementRequest {
    pub achievement_id: Uuid,
    pub progress_data: Option<String>,
}

/// Award achievement to user
///
/// POST /users/achievements
///
/// # Request Body
/// - achievement_id: UUID
/// - progress_data: Option<String> (JSON)
///
/// # Responses
/// - 201 Created: Achievement awarded successfully
/// - 400 Bad Request: Already earned (non-repeatable) or validation error
/// - 404 Not Found: Achievement not found
#[post("/users/achievements")]
pub async fn award_achievement(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<AwardAchievementRequest>,
) -> impl Responder {
    let req = request.into_inner();
    match data.achievement_use_cases.award_achievement(auth.user_id, req.achievement_id, req.progress_data).await {
        Ok(user_achievement) => HttpResponse::Created().json(user_achievement),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Get all achievements earned by current user
///
/// GET /users/achievements
///
/// # Responses
/// - 200 OK: List of user achievements with enriched achievement data
#[get("/users/achievements")]
pub async fn get_user_achievements(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    match data.achievement_use_cases.get_user_achievements(auth.user_id).await {
        Ok(achievements) => HttpResponse::Ok().json(achievements),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Get recent achievements for current user
///
/// GET /users/achievements/recent?limit=5
///
/// # Query Parameters
/// - limit: i64 (default: 5)
///
/// # Responses
/// - 200 OK: List of recent achievements
#[get("/users/achievements/recent")]
pub async fn get_recent_achievements(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let limit = query.get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(5);

    match data.achievement_use_cases.get_recent_achievements(auth.user_id, limit).await {
        Ok(achievements) => HttpResponse::Ok().json(achievements),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

// ============================================================================
// CHALLENGE HANDLERS
// ============================================================================

/// Create a new challenge (admin only)
///
/// POST /challenges
///
/// # Request Body
/// - organization_id: UUID
/// - building_id: Option<UUID> (null = organization-wide)
/// - challenge_type: ChallengeType (Individual, Team, Building)
/// - title: String (3-100 chars)
/// - description: String (10-1000 chars)
/// - icon: String
/// - start_date: DateTime<Utc>
/// - end_date: DateTime<Utc>
/// - target_metric: String (e.g., "bookings_created")
/// - target_value: i32
/// - reward_points: i32 (0-10000)
///
/// # Responses
/// - 201 Created: Challenge created successfully (Draft status)
/// - 400 Bad Request: Validation error
#[post("/challenges")]
pub async fn create_challenge(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateChallengeDto>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.challenge_use_cases.create_challenge(request.into_inner()).await {
        Ok(challenge) => HttpResponse::Created().json(challenge),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Get challenge by ID
///
/// GET /challenges/:id
///
/// # Responses
/// - 200 OK: Challenge details
/// - 404 Not Found: Challenge not found
#[get("/challenges/{id}")]
pub async fn get_challenge(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.challenge_use_cases.get_challenge(id.into_inner()).await {
        Ok(challenge) => HttpResponse::Ok().json(challenge),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

/// List all challenges for an organization
///
/// GET /organizations/:organization_id/challenges
///
/// # Responses
/// - 200 OK: List of challenges
#[get("/organizations/{organization_id}/challenges")]
pub async fn list_challenges(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match data.challenge_use_cases.list_challenges(organization_id.into_inner()).await {
        Ok(challenges) => HttpResponse::Ok().json(challenges),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List challenges by status
///
/// GET /organizations/:organization_id/challenges/status/:status
///
/// # Responses
/// - 200 OK: List of challenges with specified status
#[get("/organizations/{organization_id}/challenges/status/{status}")]
pub async fn list_challenges_by_status(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (organization_id, status_str) = path.into_inner();

    // Parse status
    let status: ChallengeStatus = match serde_json::from_str(&format!("\"{}\"", status_str)) {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid status"})),
    };

    match data.challenge_use_cases.list_challenges_by_status(organization_id, status).await {
        Ok(challenges) => HttpResponse::Ok().json(challenges),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List challenges for a building
///
/// GET /buildings/:building_id/challenges
///
/// # Responses
/// - 200 OK: List of building challenges
#[get("/buildings/{building_id}/challenges")]
pub async fn list_building_challenges(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data.challenge_use_cases.list_building_challenges(building_id.into_inner()).await {
        Ok(challenges) => HttpResponse::Ok().json(challenges),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List active challenges (Active status + date range)
///
/// GET /organizations/:organization_id/challenges/active
///
/// # Responses
/// - 200 OK: List of currently active challenges
#[get("/organizations/{organization_id}/challenges/active")]
pub async fn list_active_challenges(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match data.challenge_use_cases.list_active_challenges(organization_id.into_inner()).await {
        Ok(challenges) => HttpResponse::Ok().json(challenges),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Update challenge (Draft only, admin only)
///
/// PUT /challenges/:id
///
/// # Responses
/// - 200 OK: Challenge updated successfully
/// - 400 Bad Request: Validation error or not Draft status
/// - 404 Not Found: Challenge not found
#[put("/challenges/{id}")]
pub async fn update_challenge(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateChallengeDto>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.challenge_use_cases.update_challenge(id.into_inner(), request.into_inner()).await {
        Ok(challenge) => HttpResponse::Ok().json(challenge),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Activate challenge (Draft → Active, admin only)
///
/// PUT /challenges/:id/activate
///
/// # Responses
/// - 200 OK: Challenge activated successfully
/// - 400 Bad Request: Invalid state transition
/// - 404 Not Found: Challenge not found
#[put("/challenges/{id}/activate")]
pub async fn activate_challenge(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.challenge_use_cases.activate_challenge(id.into_inner()).await {
        Ok(challenge) => HttpResponse::Ok().json(challenge),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Complete challenge (Active → Completed, admin only)
///
/// PUT /challenges/:id/complete
///
/// # Responses
/// - 200 OK: Challenge completed successfully
/// - 400 Bad Request: Invalid state transition
/// - 404 Not Found: Challenge not found
#[put("/challenges/{id}/complete")]
pub async fn complete_challenge(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.challenge_use_cases.complete_challenge(id.into_inner()).await {
        Ok(challenge) => HttpResponse::Ok().json(challenge),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Cancel challenge (Draft/Active → Cancelled, admin only)
///
/// PUT /challenges/:id/cancel
///
/// # Responses
/// - 200 OK: Challenge cancelled successfully
/// - 400 Bad Request: Invalid state transition
/// - 404 Not Found: Challenge not found
#[put("/challenges/{id}/cancel")]
pub async fn cancel_challenge(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.challenge_use_cases.cancel_challenge(id.into_inner()).await {
        Ok(challenge) => HttpResponse::Ok().json(challenge),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Delete challenge (admin only)
///
/// DELETE /challenges/:id
///
/// # Responses
/// - 204 No Content: Challenge deleted successfully
/// - 404 Not Found: Challenge not found
#[delete("/challenges/{id}")]
pub async fn delete_challenge(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_admin_role(&auth) {
        return response;
    }

    match data.challenge_use_cases.delete_challenge(id.into_inner()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

// ============================================================================
// CHALLENGE PROGRESS HANDLERS
// ============================================================================

/// Get user progress for a challenge
///
/// GET /challenges/:challenge_id/progress
///
/// # Responses
/// - 200 OK: User progress details
/// - 404 Not Found: Progress or challenge not found
#[get("/challenges/{challenge_id}/progress")]
pub async fn get_challenge_progress(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    challenge_id: web::Path<Uuid>,
) -> impl Responder {
    match data.challenge_use_cases.get_challenge_progress(auth.user_id, challenge_id.into_inner()).await {
        Ok(progress) => HttpResponse::Ok().json(progress),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

/// List all progress for a challenge
///
/// GET /challenges/:challenge_id/all-progress
///
/// # Responses
/// - 200 OK: List of all user progress for challenge
#[get("/challenges/{challenge_id}/all-progress")]
pub async fn list_challenge_progress(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    challenge_id: web::Path<Uuid>,
) -> impl Responder {
    match data.challenge_use_cases.list_challenge_progress(challenge_id.into_inner()).await {
        Ok(progress_list) => HttpResponse::Ok().json(progress_list),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List active challenges for current user with progress
///
/// GET /users/challenges/active
///
/// # Responses
/// - 200 OK: List of active challenges with user progress
#[get("/users/challenges/active")]
pub async fn list_user_active_challenges(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    match data.challenge_use_cases.list_user_active_progress(auth.user_id).await {
        Ok(progress_list) => HttpResponse::Ok().json(progress_list),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

#[derive(Debug, Deserialize)]
pub struct IncrementProgressRequest {
    pub increment: i32,
}

/// Increment user progress for a challenge
///
/// POST /challenges/:challenge_id/progress/increment
///
/// # Request Body
/// - increment: i32 (value to add to current progress)
///
/// # Responses
/// - 200 OK: Progress incremented successfully (auto-completes if target reached)
/// - 400 Bad Request: Validation error
/// - 404 Not Found: Challenge not found
#[post("/challenges/{challenge_id}/progress/increment")]
pub async fn increment_progress(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    challenge_id: web::Path<Uuid>,
    request: web::Json<IncrementProgressRequest>,
) -> impl Responder {
    match data.challenge_use_cases.increment_progress(auth.user_id, challenge_id.into_inner(), request.increment).await {
        Ok(progress) => HttpResponse::Ok().json(progress),
        Err(e) if e.contains("not found") => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

// ============================================================================
// GAMIFICATION STATS HANDLERS
// ============================================================================

/// Get comprehensive gamification stats for current user
///
/// GET /organizations/:organization_id/gamification/stats
///
/// # Responses
/// - 200 OK: User gamification statistics
#[get("/organizations/{organization_id}/gamification/stats")]
pub async fn get_gamification_user_stats(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match data.gamification_stats_use_cases.get_user_stats(auth.user_id, organization_id.into_inner()).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Get leaderboard for organization or building
///
/// GET /organizations/:organization_id/gamification/leaderboard?building_id=<uuid>&limit=10
///
/// # Query Parameters
/// - building_id: Option<UUID> (filter by building)
/// - limit: i64 (default: 10)
///
/// # Responses
/// - 200 OK: Leaderboard with top users
#[get("/organizations/{organization_id}/gamification/leaderboard")]
pub async fn get_gamification_leaderboard(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let building_id = query.get("building_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let limit = query.get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(10);

    match data.gamification_stats_use_cases.get_leaderboard(organization_id.into_inner(), building_id, limit).await {
        Ok(leaderboard) => HttpResponse::Ok().json(leaderboard),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
