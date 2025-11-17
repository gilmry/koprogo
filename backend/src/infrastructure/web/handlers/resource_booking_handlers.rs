use crate::application::dto::{CreateResourceBookingDto, UpdateResourceBookingDto};
use crate::domain::entities::{BookingStatus, ResourceType};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

/// Create a new resource booking
///
/// POST /resource-bookings
///
/// # Request Body
/// - building_id: UUID
/// - resource_type: ResourceType (MeetingRoom, LaundryRoom, Gym, etc.)
/// - resource_name: String (e.g., "Meeting Room A")
/// - start_time: DateTime<Utc>
/// - end_time: DateTime<Utc>
/// - notes: Option<String>
/// - recurring_pattern: RecurringPattern (default: None)
/// - recurrence_end_date: Option<DateTime<Utc>>
/// - max_duration_hours: Option<i64> (default: 4)
/// - max_advance_days: Option<i64> (default: 30)
///
/// # Responses
/// - 201 Created: Booking created successfully
/// - 400 Bad Request: Validation error or conflict
/// - 404 Not Found: Building not found
#[post("/resource-bookings")]
pub async fn create_booking(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateResourceBookingDto>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .create_booking(auth.user_id, request.into_inner())
        .await
    {
        Ok(booking) => HttpResponse::Created().json(booking),
        Err(e) => {
            if e.contains("conflicts with") {
                HttpResponse::Conflict().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Get booking by ID
///
/// GET /resource-bookings/:id
///
/// # Responses
/// - 200 OK: Booking details
/// - 404 Not Found: Booking not found
#[get("/resource-bookings/{id}")]
pub async fn get_booking(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .get_booking(id.into_inner())
        .await
    {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

/// List all bookings for a building
///
/// GET /buildings/:building_id/resource-bookings
///
/// # Responses
/// - 200 OK: List of bookings
#[get("/buildings/{building_id}/resource-bookings")]
pub async fn list_building_bookings(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .list_building_bookings(building_id.into_inner())
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List bookings by resource type
///
/// GET /buildings/:building_id/resource-bookings/type/:resource_type
///
/// # Responses
/// - 200 OK: List of bookings for resource type
#[get("/buildings/{building_id}/resource-bookings/type/{resource_type}")]
pub async fn list_by_resource_type(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, resource_type_str) = path.into_inner();

    // Parse resource_type from string
    let resource_type: ResourceType =
        match serde_json::from_str(&format!("\"{}\"", resource_type_str)) {
            Ok(rt) => rt,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid resource type: {}", resource_type_str)
                }))
            }
        };

    match data
        .resource_booking_use_cases
        .list_by_resource_type(building_id, resource_type)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List bookings for a specific resource
///
/// GET /buildings/:building_id/resource-bookings/resource/:resource_type/:resource_name
///
/// # Responses
/// - 200 OK: List of bookings for specific resource
#[get("/buildings/{building_id}/resource-bookings/resource/{resource_type}/{resource_name}")]
pub async fn list_by_resource(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String, String)>,
) -> impl Responder {
    let (building_id, resource_type_str, resource_name) = path.into_inner();

    // Parse resource_type from string
    let resource_type: ResourceType =
        match serde_json::from_str(&format!("\"{}\"", resource_type_str)) {
            Ok(rt) => rt,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid resource type: {}", resource_type_str)
                }))
            }
        };

    match data
        .resource_booking_use_cases
        .list_by_resource(building_id, resource_type, resource_name)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List user's bookings
///
/// GET /resource-bookings/my
///
/// # Responses
/// - 200 OK: List of user's bookings
#[get("/resource-bookings/my")]
pub async fn list_my_bookings(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .list_user_bookings(auth.user_id)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List user's bookings by status
///
/// GET /resource-bookings/my/status/:status
///
/// # Responses
/// - 200 OK: List of user's bookings with status
#[get("/resource-bookings/my/status/{status}")]
pub async fn list_my_bookings_by_status(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    status: web::Path<String>,
) -> impl Responder {
    // Parse status from string
    let booking_status: BookingStatus =
        match serde_json::from_str(&format!("\"{}\"", status.into_inner())) {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Invalid status"}))
            }
        };

    match data
        .resource_booking_use_cases
        .list_user_bookings_by_status(auth.user_id, booking_status)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List building bookings by status
///
/// GET /buildings/:building_id/resource-bookings/status/:status
///
/// # Responses
/// - 200 OK: List of bookings with status
#[get("/buildings/{building_id}/resource-bookings/status/{status}")]
pub async fn list_building_bookings_by_status(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, status_str) = path.into_inner();

    // Parse status from string
    let booking_status: BookingStatus = match serde_json::from_str(&format!("\"{}\"", status_str))
    {
        Ok(s) => s,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid status: {}", status_str)
            }))
        }
    };

    match data
        .resource_booking_use_cases
        .list_building_bookings_by_status(building_id, booking_status)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

#[derive(Deserialize)]
pub struct UpcomingQuery {
    limit: Option<i64>,
}

/// List upcoming bookings (future, confirmed or pending)
///
/// GET /buildings/:building_id/resource-bookings/upcoming?limit=50
///
/// # Responses
/// - 200 OK: List of upcoming bookings
#[get("/buildings/{building_id}/resource-bookings/upcoming")]
pub async fn list_upcoming_bookings(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    query: web::Query<UpcomingQuery>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .list_upcoming_bookings(building_id.into_inner(), query.limit)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List active bookings (currently in progress)
///
/// GET /buildings/:building_id/resource-bookings/active
///
/// # Responses
/// - 200 OK: List of active bookings
#[get("/buildings/{building_id}/resource-bookings/active")]
pub async fn list_active_bookings(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .list_active_bookings(building_id.into_inner())
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

#[derive(Deserialize)]
pub struct PastQuery {
    limit: Option<i64>,
}

/// List past bookings
///
/// GET /buildings/:building_id/resource-bookings/past?limit=50
///
/// # Responses
/// - 200 OK: List of past bookings
#[get("/buildings/{building_id}/resource-bookings/past")]
pub async fn list_past_bookings(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    query: web::Query<PastQuery>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .list_past_bookings(building_id.into_inner(), query.limit)
        .await
    {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Update booking details (resource_name, notes)
///
/// PUT /resource-bookings/:id
///
/// # Request Body
/// - resource_name: Option<String>
/// - notes: Option<String>
///
/// # Responses
/// - 200 OK: Booking updated
/// - 400 Bad Request: Validation error
/// - 403 Forbidden: Not booking owner
/// - 404 Not Found: Booking not found
#[put("/resource-bookings/{id}")]
pub async fn update_booking(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateResourceBookingDto>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .update_booking(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(e) => {
            if e.contains("Only the booking owner") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Cancel a booking
///
/// POST /resource-bookings/:id/cancel
///
/// # Responses
/// - 200 OK: Booking cancelled
/// - 400 Bad Request: Cannot cancel (invalid state)
/// - 403 Forbidden: Not booking owner
/// - 404 Not Found: Booking not found
#[post("/resource-bookings/{id}/cancel")]
pub async fn cancel_booking(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .cancel_booking(id.into_inner(), auth.user_id)
        .await
    {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(e) => {
            if e.contains("Only the booking owner") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Complete a booking (admin only)
///
/// POST /resource-bookings/:id/complete
///
/// # Responses
/// - 200 OK: Booking completed
/// - 400 Bad Request: Cannot complete
/// - 404 Not Found: Booking not found
#[post("/resource-bookings/{id}/complete")]
pub async fn complete_booking(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .complete_booking(id.into_inner())
        .await
    {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Mark booking as no-show (admin only)
///
/// POST /resource-bookings/:id/no-show
///
/// # Responses
/// - 200 OK: Booking marked as no-show
/// - 400 Bad Request: Cannot mark as no-show
/// - 404 Not Found: Booking not found
#[post("/resource-bookings/{id}/no-show")]
pub async fn mark_no_show(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .mark_no_show(id.into_inner())
        .await
    {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Confirm a pending booking (admin only)
///
/// POST /resource-bookings/:id/confirm
///
/// # Responses
/// - 200 OK: Booking confirmed
/// - 400 Bad Request: Cannot confirm
/// - 404 Not Found: Booking not found
#[post("/resource-bookings/{id}/confirm")]
pub async fn confirm_booking(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .confirm_booking(id.into_inner())
        .await
    {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Delete a booking
///
/// DELETE /resource-bookings/:id
///
/// # Responses
/// - 204 No Content: Booking deleted
/// - 403 Forbidden: Not booking owner
/// - 404 Not Found: Booking not found
#[delete("/resource-bookings/{id}")]
pub async fn delete_booking(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .delete_booking(id.into_inner(), auth.user_id)
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => {
            if e.contains("Only the booking owner") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct CheckConflictsQuery {
    pub building_id: Uuid,
    pub resource_type: String,
    pub resource_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub exclude_booking_id: Option<Uuid>,
}

/// Check for booking conflicts (preview before creating)
///
/// GET /resource-bookings/check-conflicts?building_id=...&resource_type=...&resource_name=...&start_time=...&end_time=...
///
/// # Query Parameters
/// - building_id: UUID
/// - resource_type: String
/// - resource_name: String
/// - start_time: ISO 8601 DateTime
/// - end_time: ISO 8601 DateTime
/// - exclude_booking_id: Option<UUID>
///
/// # Responses
/// - 200 OK: List of conflicting bookings (empty if no conflicts)
#[get("/resource-bookings/check-conflicts")]
pub async fn check_conflicts(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    query: web::Query<CheckConflictsQuery>,
) -> impl Responder {
    // Parse resource_type
    let resource_type: ResourceType =
        match serde_json::from_str(&format!("\"{}\"", query.resource_type)) {
            Ok(rt) => rt,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid resource type: {}", query.resource_type)
                }))
            }
        };

    match data
        .resource_booking_use_cases
        .check_conflicts(
            query.building_id,
            resource_type,
            query.resource_name.clone(),
            query.start_time,
            query.end_time,
            query.exclude_booking_id,
        )
        .await
    {
        Ok(conflicts) => HttpResponse::Ok().json(conflicts),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Get booking statistics for a building
///
/// GET /buildings/:building_id/resource-bookings/statistics
///
/// # Responses
/// - 200 OK: Booking statistics
#[get("/buildings/{building_id}/resource-bookings/statistics")]
pub async fn get_statistics(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .resource_booking_use_cases
        .get_statistics(building_id.into_inner())
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
