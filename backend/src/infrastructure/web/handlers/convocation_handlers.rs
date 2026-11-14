use crate::application::dto::{
    CreateConvocationRequest, ScheduleConvocationRequest, ScheduleSecondConvocationRequest,
    SendConvocationRequest, SetProxyRequest, UpdateAttendanceRequest,
};
use crate::application::error::AppError;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::middleware::scope_guard::{
    verify_building_org_access, verify_convocation_org_access,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::ResponseError;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Convocation CRUD Endpoints ====================

/// Envoyer une convocation fait courir le délai légal de l'Art. 3.87 §3 CC
/// pour tous les copropriétaires : c'est un acte du syndic, jamais celui d'un
/// copropriétaire membre de la même organisation. Relevé en écrivant le
/// parcours multi-rôle complet du cycle d'AG (#780).
fn require_syndic_or_superadmin(user: &AuthenticatedUser) -> Result<(), AppError> {
    match user.role.as_str() {
        "syndic" | "superadmin" => Ok(()),
        _ => Err(AppError::Forbidden(
            "Seul le syndic peut envoyer une convocation".to_string(),
        )),
    }
}

/// Le pixel d'ouverture d'une convocation : un GIF transparent de 1 x 1.
const PIXEL_GIF: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];

/// Finalité du jeton signé du pixel d'ouverture.
const FINALITE_OUVERTURE: &str = "ouverture-convocation";

/// Chemin du pixel d'ouverture à embarquer dans l'email de convocation d'un
/// destinataire, sous `/api/v1`.
pub fn url_pixel_ouverture(state: &AppState, recipient_id: Uuid) -> Option<String> {
    let jeton = state
        .auth_use_cases
        .signer_identifiant(FINALITE_OUVERTURE, recipient_id)?;
    Some(format!("/convocation-recipients/ouverture/{jeton}"))
}

/// Le jour de l'AG, le syndic contrôle les présences et les procurations
/// jusqu'au quorum : lui seul (et le superadmin) les consigne, et sur une
/// convocation de son organisation. Sans cette garde, n'importe quel compte
/// se désignait mandataire d'autrui, puis votait en son nom.
async fn verifier_controle_de_seance(
    state: &web::Data<AppState>,
    user: &AuthenticatedUser,
    recipient_id: Uuid,
) -> Result<(), Box<HttpResponse>> {
    if !(user.is_superadmin() || user.role == "syndic") {
        return Err(Box::new(HttpResponse::Forbidden().json(
            serde_json::json!({
                "error": "Le contrôle des présences et des procurations revient au syndic."
            }),
        )));
    }
    let convocation_id = match state
        .convocation_use_cases
        .convocation_du_destinataire(recipient_id)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return Err(Box::new(HttpResponse::NotFound().json(
                serde_json::json!({"error": format!("Recipient not found: {recipient_id}")}),
            )))
        }
        Err(e) => {
            return Err(Box::new(
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
            ))
        }
    };
    verify_convocation_org_access(
        user,
        convocation_id,
        &state.convocation_use_cases,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    .map_err(|err| Box::new(err.error_response()))
}

#[post("/convocations")]
pub async fn create_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Isolation multi-tenant à l'ÉCRITURE (ADR-0045) : la convocation est un
    // acte de l'ACP, dont elle supporte les frais (Art. 3.87 § 3). L'immeuble
    // visé doit relever d'une ACP confiée à ce syndic.
    if let Err(err) = verify_building_org_access(
        &user,
        request.building_id,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    let created_by = user.user_id;

    match state
        .convocation_use_cases
        .create_convocation(organization_id, request.into_inner(), created_by)
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .log();

            HttpResponse::Created().json(convocation)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/convocations/{id}")]
pub async fn get_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.convocation_use_cases.get_convocation(*id).await {
        Ok(convocation) => {
            // Verify organization access
            if let Err(err) = user.verify_org_access(convocation.organization_id) {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": err}));
            }
            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::NotFound().json(serde_json::json!({"error": err})),
    }
}

#[get("/meetings/{meeting_id}/convocation")]
pub async fn get_convocation_by_meeting(
    state: web::Data<AppState>,
    meeting_id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772). C'est
    // par ce genre de route qu'un cabinet a lu les bulletins NOMINATIFS d'une
    // autre copropriete (RN-2).
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_meeting_org_access(
            &user,
            *meeting_id,
            &state.meeting_use_cases,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state
        .convocation_use_cases
        .get_convocation_by_meeting(*meeting_id)
        .await
    {
        Ok(Some(convocation)) => HttpResponse::Ok().json(convocation),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Convocation not found for this meeting"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/buildings/{building_id}/convocations")]
pub async fn list_building_convocations(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // servait une sous-collection d'un dossier d'ACP a quiconque connaissait
    // un identifiant, sans demander d'identite.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            *building_id,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state
        .convocation_use_cases
        .list_building_convocations(*building_id)
        .await
    {
        Ok(convocations) => HttpResponse::Ok().json(convocations),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/organizations/{organization_id}/convocations")]
pub async fn list_organization_convocations(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = user.verify_org_access(*organization_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e}));
    }
    match state
        .convocation_use_cases
        .list_organization_convocations(*organization_id)
        .await
    {
        Ok(convocations) => HttpResponse::Ok().json(convocations),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[delete("/convocations/{id}")]
pub async fn delete_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state.convocation_use_cases.delete_convocation(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Convocation not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Convocation Actions ====================

#[put("/convocations/{id}/schedule")]
pub async fn schedule_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<ScheduleConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .convocation_use_cases
        .schedule_convocation(*id, request.into_inner())
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationScheduled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .log();

            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[post("/convocations/{id}/send")]
pub async fn send_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<SendConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    if let Err(err) = require_syndic_or_superadmin(&user) {
        return err.error_response();
    }

    // PDF generation now happens in the use case layer
    match state
        .convocation_use_cases
        .send_convocation(*id, request.into_inner())
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationSent,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .with_details(format!("recipients: {}", convocation.total_recipients))
            .log();

            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/convocations/{id}/cancel")]
pub async fn cancel_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state.convocation_use_cases.cancel_convocation(*id).await {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationCancelled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .log();

            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

// ==================== Recipient Endpoints ====================

#[get("/convocations/{id}/recipients")]
pub async fn list_convocation_recipients(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // sert la liste NOMINATIVE des copropriétaires convoqués.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_convocation_org_access(
            &user,
            *id,
            &state.convocation_use_cases,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state
        .convocation_use_cases
        .list_convocation_recipients(*id)
        .await
    {
        Ok(recipients) => HttpResponse::Ok().json(recipients),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Les copropriétaires qu'une convocation pour cet immeuble toucherait.
///
/// Sert l'écran de sélection des destinataires (#780 verrou 1, #784) : le
/// syndic doit pouvoir voir et choisir AVANT d'envoyer, pas seulement
/// constater après coup que « 0 destinataire » était resté un libellé.
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/eligible-convocation-recipients",
    tag = "Convocations",
    summary = "Copropriétaires éligibles à une convocation",
    params(("building_id" = Uuid, Path, description = "UUID de l'immeuble")),
    responses(
        (status = 200, description = "Destinataires éligibles", body = Vec<crate::application::dto::convocation_dto::EligibleRecipientResponse>),
        (status = 403, description = "Hors portée"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{building_id}/eligible-convocation-recipients")]
pub async fn list_eligible_convocation_recipients(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(err) = verify_building_org_access(
        &user,
        *building_id,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match state
        .convocation_use_cases
        .list_eligible_recipients(*building_id)
        .await
    {
        Ok(destinataires) => HttpResponse::Ok().json(destinataires),
        Err(err) => err.error_response(),
    }
}

#[get("/convocations/{id}/tracking-summary")]
pub async fn get_convocation_tracking_summary(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // sert la liste NOMINATIVE des copropriétaires convoqués.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_convocation_org_access(
            &user,
            *id,
            &state.convocation_use_cases,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state.convocation_use_cases.get_tracking_summary(*id).await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Ouverture de l'email de convocation, constatée par le pixel qu'il embarque.
/// Route publique : le jeton est signé par le serveur
/// (`url_pixel_ouverture`), un identifiant brut ne marque rien. La réponse
/// est la même dans tous les cas, pour ne rien apprendre à qui sonde.
#[get("/convocation-recipients/ouverture/{jeton}")]
pub async fn pixel_ouverture_convocation(
    state: web::Data<AppState>,
    jeton: web::Path<String>,
) -> impl Responder {
    if let Some(recipient_id) = state
        .auth_use_cases
        .verifier_identifiant(FINALITE_OUVERTURE, &jeton)
    {
        if let Err(err) = state
            .convocation_use_cases
            .mark_recipient_email_opened(recipient_id)
            .await
        {
            log::warn!("pixel d'ouverture {recipient_id} : {err}");
        }
    }
    HttpResponse::Ok()
        .content_type("image/gif")
        .insert_header(("Cache-Control", "no-store"))
        .body(PIXEL_GIF)
}

#[put("/convocation-recipients/{id}/attendance")]
pub async fn update_recipient_attendance(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateAttendanceRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    if let Err(refus) = verifier_controle_de_seance(&state, &user, *id).await {
        return *refus;
    }

    match state
        .convocation_use_cases
        .update_recipient_attendance(*id, request.attendance_status.clone())
        .await
    {
        Ok(recipient) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationAttendanceUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("ConvocationRecipient", recipient.id)
            .with_details(format!("status: {:?}", recipient.attendance_status))
            .log();

            HttpResponse::Ok().json(recipient)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/convocation-recipients/{id}/proxy")]
pub async fn set_recipient_proxy(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<SetProxyRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    if let Err(refus) = verifier_controle_de_seance(&state, &user, *id).await {
        return *refus;
    }

    match state
        .convocation_use_cases
        .set_recipient_proxy(*id, request.proxy_owner_id)
        .await
    {
        Ok(recipient) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationProxySet,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("ConvocationRecipient", recipient.id)
            .with_details(format!("proxy_owner_id: {:?}", recipient.proxy_owner_id))
            .log();

            HttpResponse::Ok().json(recipient)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[post("/convocations/{id}/reminders")]
pub async fn send_convocation_reminders(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state.convocation_use_cases.send_reminders(*id).await {
        Ok(recipients) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationReminderSent,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", *id)
            .with_details(format!("recipients: {}", recipients.len()))
            .log();

            HttpResponse::Ok().json(recipients)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

// ==================== Second Convocation Endpoint (Art. 3.87 §5 CC) ====================

#[post("/convocations/second")]
pub async fn schedule_second_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<ScheduleSecondConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let req = request.into_inner();

    // Isolation multi-tenant à l'ÉCRITURE (ADR-0045) : la seconde convocation
    // crée une assemblée ET une convocation dans le dossier de l'ACP. Sans
    // garde, un cabinet tiers convoquerait les copropriétaires d'un autre.
    if let Err(err) = verify_building_org_access(
        &user,
        req.building_id,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    // Create a new meeting for the second convocation
    let new_meeting_req = crate::application::dto::CreateMeetingRequest {
        organization_id,
        building_id: req.building_id,
        meeting_type: crate::domain::entities::MeetingType::Ordinary,
        title: format!("Second Convocation (Art. 3.87 §5 CC)"),
        description: Some("Second convocation after quorum not reached".to_string()),
        scheduled_date: req.new_meeting_date,
        location: "Same as first meeting".to_string(),
        is_second_convocation: true,
    };

    // Create the new meeting
    let new_meeting = match state
        .meeting_use_cases
        .create_meeting(new_meeting_req)
        .await
    {
        Ok(m) => m,
        Err(err) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": format!("Failed to create meeting: {}", err)}))
        }
    };

    match state
        .convocation_use_cases
        .schedule_second_convocation(
            organization_id,
            req.building_id,
            req.first_meeting_id,
            new_meeting.id,
            req.new_meeting_date,
            req.language,
            user.user_id,
        )
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::SecondConvocationScheduled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .with_details(format!(
                "first_meeting_id: {}, new_meeting_id: {}",
                req.first_meeting_id, new_meeting.id
            ))
            .log();

            HttpResponse::Created().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}
