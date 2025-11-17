use crate::application::dto::{
    AddAgendaItemRequest, CompleteMeetingRequest, CreateMeetingRequest, PageRequest, PageResponse,
    RescheduleMeetingRequest, UpdateMeetingRequest,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

#[post("/meetings")]
pub async fn create_meeting(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut request: web::Json<CreateMeetingRequest>,
) -> impl Responder {
    // Override the organization_id from request with the one from JWT token
    // This prevents users from creating meetings in other organizations
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };
    request.organization_id = organization_id;

    match state
        .meeting_use_cases
        .create_meeting(request.into_inner())
        .await
    {
        Ok(meeting) => {
            // Audit log: successful meeting creation
            AuditLogEntry::new(
                AuditEventType::MeetingCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Meeting", meeting.id)
            .log();

            HttpResponse::Created().json(meeting)
        }
        Err(err) => {
            // Audit log: failed meeting creation
            AuditLogEntry::new(
                AuditEventType::MeetingCreated,
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

#[get("/meetings/{id}")]
pub async fn get_meeting(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.meeting_use_cases.get_meeting(*id).await {
        Ok(Some(meeting)) => HttpResponse::Ok().json(meeting),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Meeting not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/meetings")]
pub async fn list_meetings(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    let organization_id = user.organization_id;

    match state
        .meeting_use_cases
        .list_meetings_paginated(&page_request, organization_id)
        .await
    {
        Ok((meetings, total)) => {
            let response =
                PageResponse::new(meetings, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{building_id}/meetings")]
pub async fn list_meetings_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .list_meetings_by_building(*building_id)
        .await
    {
        Ok(meetings) => HttpResponse::Ok().json(meetings),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/meetings/{id}")]
pub async fn update_meeting(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    request: web::Json<UpdateMeetingRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .update_meeting(*id, request.into_inner())
        .await
    {
        Ok(meeting) => HttpResponse::Ok().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/meetings/{id}/agenda")]
pub async fn add_agenda_item(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    request: web::Json<AddAgendaItemRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .add_agenda_item(*id, request.into_inner())
        .await
    {
        Ok(meeting) => HttpResponse::Ok().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/meetings/{id}/complete")]
pub async fn complete_meeting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<CompleteMeetingRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .complete_meeting(*id, request.into_inner())
        .await
    {
        Ok(meeting) => {
            // Audit log: successful meeting completion
            AuditLogEntry::new(
                AuditEventType::MeetingCompleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Meeting", *id)
            .log();

            HttpResponse::Ok().json(meeting)
        }
        Err(err) => {
            // Audit log: failed meeting completion
            AuditLogEntry::new(
                AuditEventType::MeetingCompleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Meeting", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

#[post("/meetings/{id}/cancel")]
pub async fn cancel_meeting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.meeting_use_cases.cancel_meeting(*id).await {
        Ok(meeting) => {
            AuditLogEntry::new(
                AuditEventType::MeetingCompleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Meeting", *id)
            .log();

            HttpResponse::Ok().json(meeting)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/meetings/{id}/reschedule")]
pub async fn reschedule_meeting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<RescheduleMeetingRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .reschedule_meeting(*id, request.scheduled_date)
        .await
    {
        Ok(meeting) => {
            AuditLogEntry::new(
                AuditEventType::MeetingCompleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Meeting", *id)
            .log();

            HttpResponse::Ok().json(meeting)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[delete("/meetings/{id}")]
pub async fn delete_meeting(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.meeting_use_cases.delete_meeting(*id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Meeting not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/meetings/{id}/export-minutes-pdf")]
pub async fn export_meeting_minutes_pdf(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    use crate::domain::services::{AttendeeInfo, MeetingMinutesExporter, ResolutionWithVotes};

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    let meeting_id = *id;

    // 1. Get meeting
    let meeting = match state.meeting_use_cases.get_meeting(meeting_id).await {
        Ok(Some(meeting_dto)) => meeting_dto,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Meeting not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // 2. Get building
    let building = match state
        .building_use_cases
        .get_building(meeting.building_id)
        .await
    {
        Ok(Some(building_dto)) => building_dto,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Building not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // 3. Get resolutions for this meeting
    let resolutions = match state
        .resolution_use_cases
        .get_meeting_resolutions(meeting_id)
        .await
    {
        Ok(resolutions) => resolutions,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get resolutions: {}", err)
            }))
        }
    };

    // 4. Get votes for each resolution and collect attendees
    let mut attendees_map = std::collections::HashMap::new();
    let mut resolutions_with_votes = Vec::new();

    for resolution_dto in resolutions {
        // Get votes for this resolution
        let votes_dto = match state
            .resolution_use_cases
            .get_resolution_votes(resolution_dto.id)
            .await
        {
            Ok(votes) => votes,
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to get votes: {}", err)
                }))
            }
        };

        // Collect attendees from votes
        for vote_dto in &votes_dto {
            if !attendees_map.contains_key(&vote_dto.owner_id) {
                // Get owner info
                if let Ok(Some(owner_dto)) = state.owner_use_cases.get_owner(vote_dto.owner_id).await
                {
                    let proxy_for_name = if let Some(proxy_id) = vote_dto.proxy_owner_id {
                        state
                            .owner_use_cases
                            .get_owner(proxy_id)
                            .await
                            .ok()
                            .flatten()
                            .map(|o| format!("{} {}", o.first_name, o.last_name))
                    } else {
                        None
                    };

                    let full_name = format!("{} {}", owner_dto.first_name, owner_dto.last_name);

                    attendees_map.insert(
                        vote_dto.owner_id,
                        AttendeeInfo {
                            owner_id: vote_dto.owner_id,
                            name: full_name,
                            email: owner_dto.email.clone(),
                            voting_power: vote_dto.voting_power,
                            is_proxy: vote_dto.proxy_owner_id.is_some(),
                            proxy_for: proxy_for_name,
                        },
                    );
                }
            }
        }

        // Convert DTOs to domain entities for PDF generation
        use crate::domain::entities::{Resolution, Vote, MajorityType, ResolutionStatus, ResolutionType, VoteChoice};

        let resolution_entity = Resolution {
            id: resolution_dto.id,
            meeting_id: resolution_dto.meeting_id,
            title: resolution_dto.title,
            description: resolution_dto.description,
            resolution_type: resolution_dto.resolution_type,
            majority_required: resolution_dto.majority_required,
            vote_count_pour: resolution_dto.vote_count_pour,
            vote_count_contre: resolution_dto.vote_count_contre,
            vote_count_abstention: resolution_dto.vote_count_abstention,
            total_voting_power_pour: resolution_dto.total_voting_power_pour,
            total_voting_power_contre: resolution_dto.total_voting_power_contre,
            total_voting_power_abstention: resolution_dto.total_voting_power_abstention,
            status: resolution_dto.status,
            voted_at: resolution_dto.voted_at,
            created_at: resolution_dto.created_at,
        };

        let votes: Vec<Vote> = votes_dto.iter().map(|v| Vote {
            id: v.id,
            resolution_id: v.resolution_id,
            owner_id: v.owner_id,
            unit_id: v.unit_id,
            vote_choice: v.vote_choice.clone(),
            voting_power: v.voting_power,
            proxy_owner_id: v.proxy_owner_id,
            voted_at: v.voted_at,
        }).collect();

        resolutions_with_votes.push(ResolutionWithVotes {
            resolution: resolution_entity,
            votes,
        });
    }

    let attendees: Vec<AttendeeInfo> = attendees_map.into_values().collect();

    // Convert DTOs to domain entities
    use crate::domain::entities::{Building, Meeting};

    // Parse building organization_id from string
    let building_org_id = match Uuid::parse_str(&building.organization_id) {
        Ok(id) => id,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Invalid organization_id: {}", err)
            }))
        }
    };

    // Parse building dates
    use chrono::DateTime;
    let building_created_at = match DateTime::parse_from_rfc3339(&building.created_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => chrono::Utc::now(),
    };
    let building_updated_at = match DateTime::parse_from_rfc3339(&building.updated_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => chrono::Utc::now(),
    };

    let building_entity = Building {
        id: match Uuid::parse_str(&building.id) {
            Ok(id) => id,
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Invalid building id: {}", err)
                }))
            }
        },
        name: building.name,
        address: building.address,
        city: building.city,
        postal_code: building.postal_code,
        country: building.country,
        total_units: building.total_units,
        total_tantiemes: building.total_tantiemes,
        construction_year: building.construction_year,
        organization_id: building_org_id,
        created_at: building_created_at,
        updated_at: building_updated_at,
    };

    let meeting_entity = Meeting {
        id: meeting.id,
        organization_id,
        building_id: meeting.building_id,
        meeting_type: meeting.meeting_type,
        title: meeting.title,
        description: meeting.description,
        scheduled_date: meeting.scheduled_date,
        location: meeting.location,
        status: meeting.status,
        agenda: meeting.agenda,
        attendees_count: meeting.attendees_count,
        created_at: meeting.created_at,
        updated_at: meeting.updated_at,
    };

    // 5. Generate PDF
    match MeetingMinutesExporter::export_to_pdf(
        &building_entity,
        &meeting_entity,
        &attendees,
        &resolutions_with_votes,
    ) {
        Ok(pdf_bytes) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Meeting", meeting_id)
            .with_metadata(serde_json::json!({
                "report_type": "meeting_minutes_pdf",
                "building_name": building_entity.name,
                "meeting_date": meeting_entity.scheduled_date.to_rfc3339()
            }))
            .log();

            HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header((
                    "Content-Disposition",
                    format!(
                        "attachment; filename=\"PV_{}_{}_{}.pdf\"",
                        building_entity.name.replace(' ', "_"),
                        meeting_entity.scheduled_date.format("%Y%m%d"),
                        meeting_entity.id
                    ),
                ))
                .body(pdf_bytes)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate PDF: {}", err)
        })),
    }
}
