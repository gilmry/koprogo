use crate::application::dto::{
    CastVoteRequest, ChangeVoteRequest, CloseVotingRequest, CreateResolutionRequest,
    ResolutionResponse, VoteResponse,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::classification_erreurs::est_interdit;
use crate::infrastructure::web::middleware::scope_guard::verify_acp_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

// ==================== Resolution Endpoints ====================

/// Vérifie que l'appelant a un mandat sur l'ACP dont relève une AG.
///
/// **Pourquoi une fonction et pas une ligne recopiée.** La chaîne à remonter
/// fait quatre sauts — AG → immeuble → ACP → organisation. Recopiée dans
/// chaque gestionnaire, elle finit par manquer là où on n'y a pas pensé :
/// c'est exactement ce qui est arrivé à `list_meeting_resolutions` et
/// `list_resolution_votes`, qui rendaient 200 sur les données d'une autre
/// copropriété pendant que l'accès direct à la même ressource rendait 403.
///
/// Constaté en recette le 2026-09-06 (RN-2) : un syndic concurrent lisait le
/// sens du vote de copropriétaires nommés, avec leur poids. Le vote en AG est
/// confidentiel et nominatif : c'est une violation de données, pas un simple
/// écart de périmètre.
///
/// Le maillon absent est traité comme un refus. Une AG dont on ne retrouve ni
/// l'immeuble ni l'ACP ne peut pas être autorisée « par défaut » : c'est
/// précisément le cas où l'on ne sait pas à qui elle appartient.
///
/// Rend `None` quand l'accès est accordé, `Some(reponse)` quand il est refusé.
/// Un `Result<(), HttpResponse>` dirait la même chose, mais clippy le refuse à
/// juste titre : `HttpResponse` est volumineux, et le porter dans la variante
/// d'erreur alourdirait chaque valeur de retour, y compris sur le chemin
/// nominal.
async fn verifier_mandat_sur_ag(
    state: &web::Data<AppState>,
    user: &AuthenticatedUser,
    meeting_id: Uuid,
) -> Option<HttpResponse> {
    if user.is_superadmin() {
        return None;
    }
    let introuvable = || {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Meeting not found"
        }))
    };
    let Ok(Some(meeting)) = state.meeting_use_cases.get_meeting(meeting_id).await else {
        return Some(introuvable());
    };
    let Ok(Some(building)) = state
        .building_use_cases
        .get_building(meeting.building_id)
        .await
    else {
        return Some(introuvable());
    };
    let Ok(acp_id) = Uuid::parse_str(&building.acp_id) else {
        return Some(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Invalid building.acp_id format"
        })));
    };
    verify_acp_org_access(user, acp_id, &state.acp_use_cases)
        .await
        .err()
        .map(|err| err.error_response())
}

#[utoipa::path(
    post,
    path = "/meetings/{meeting_id}/resolutions",
    tag = "Resolutions",
    summary = "Create a resolution for a meeting",
    params(
        ("meeting_id" = Uuid, Path, description = "Meeting UUID")
    ),
    request_body = CreateResolutionRequest,
    responses(
        (status = 201, description = "Resolution created"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/meetings/{meeting_id}/resolutions")]
pub async fn create_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    meeting_id: web::Path<Uuid>,
    request: web::Json<CreateResolutionRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let meeting_id = *meeting_id;

    match state
        .resolution_use_cases
        .create_resolution(
            meeting_id,
            request.title.clone(),
            request.description.clone(),
            request.resolution_type.clone(),
            request.majority_required.clone(),
            request.agenda_item_index,
        )
        .await
    {
        Ok(resolution) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Resolution", resolution.id)
            .log();

            HttpResponse::Created().json(ResolutionResponse::from(resolution))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/resolutions/{id}",
    tag = "Resolutions",
    summary = "Get resolution by ID",
    params(
        ("id" = Uuid, Path, description = "Resolution UUID")
    ),
    responses(
        (status = 200, description = "Resolution found"),
        (status = 404, description = "Resolution not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/resolutions/{id}")]
pub async fn get_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.resolution_use_cases.get_resolution(*id).await {
        Ok(Some(resolution)) => {
            // Hotfix #603 — multi-tenant isolation via ACP→organization resolution.
            // Resolution → Meeting → Building → Acp → Organization
            if let Ok(Some(meeting)) = state
                .meeting_use_cases
                .get_meeting(resolution.meeting_id)
                .await
            {
                if let Ok(Some(building)) = state
                    .building_use_cases
                    .get_building(meeting.building_id)
                    .await
                {
                    let acp_id = match Uuid::parse_str(&building.acp_id) {
                        Ok(id) => id,
                        Err(_) => {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Invalid building.acp_id format"
                            }));
                        }
                    };
                    if let Err(err) =
                        verify_acp_org_access(&user, acp_id, &state.acp_use_cases).await
                    {
                        return err.error_response();
                    }
                }
            }
            HttpResponse::Ok().json(ResolutionResponse::from(resolution))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Resolution not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    get,
    path = "/meetings/{meeting_id}/resolutions",
    tag = "Resolutions",
    summary = "List all resolutions for a meeting",
    params(
        ("meeting_id" = Uuid, Path, description = "Meeting UUID")
    ),
    responses(
        (status = 200, description = "List of resolutions"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/meetings/{meeting_id}/resolutions")]
pub async fn list_meeting_resolutions(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    meeting_id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(refus) = verifier_mandat_sur_ag(&state, &user, *meeting_id).await {
        return refus;
    }
    match state
        .resolution_use_cases
        .get_meeting_resolutions(*meeting_id)
        .await
    {
        Ok(resolutions) => {
            let responses: Vec<ResolutionResponse> = resolutions
                .into_iter()
                .map(ResolutionResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/resolutions/{id}",
    tag = "Resolutions",
    summary = "Delete a resolution",
    params(
        ("id" = Uuid, Path, description = "Resolution UUID")
    ),
    responses(
        (status = 204, description = "Resolution deleted"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Resolution not found"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/resolutions/{id}")]
pub async fn delete_resolution(
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

    match state.resolution_use_cases.delete_resolution(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Resolution", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Resolution not found"
        })),
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

// ==================== Vote Endpoints ====================

#[utoipa::path(
    post,
    path = "/resolutions/{resolution_id}/vote",
    tag = "Resolutions",
    summary = "Cast a vote on a resolution",
    params(
        ("resolution_id" = Uuid, Path, description = "Resolution UUID")
    ),
    request_body = CastVoteRequest,
    responses(
        (status = 201, description = "Vote cast"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/resolutions/{resolution_id}/vote")]
pub async fn cast_vote(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
    request: web::Json<CastVoteRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // #850 — QUI vote est résolu depuis l'utilisateur authentifié, jamais
    // repris tel quel du corps de la requête. `AuthenticatedUser` était
    // présent dans cette signature depuis toujours, mais ne servait qu'à
    // `require_organization()` et au journal d'audit : rien ne rapprochait
    // son identité de `request.owner_id`. Même geste que #849
    // (`poll_handlers.rs::cast_poll_vote`) : résoudre le copropriétaire, pas
    // l'utilisateur.
    let caller_owner_id = match state
        .owner_use_cases
        .find_owner_by_user_id(user.user_id)
        .await
    {
        Ok(Some(owner)) => match Uuid::parse_str(&owner.id) {
            Ok(id) => id,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Identifiant de copropriétaire illisible : {}", e)
                }));
            }
        },
        Ok(None) => {
            // Un utilisateur sans fiche de copropriétaire ne peut voter pour
            // personne — ni pour lui-même (il n'a pas de lot), ni comme
            // mandataire (aucune identité à opposer à `caller_owner_id`).
            //
            // Ceci ferme, pour l'instant, la question que #850 pose sans la
            // trancher : le syndic doit-il pouvoir saisir des votes en séance ?
            // Cette route ne le permet plus. Si l'usage l'exige, il faut une
            // route dédiée, réservée au syndic, journalisée comme saisie pour
            // compte de tiers et soumise à la limite des procurations — pas
            // rouvrir celle-ci.
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Aucune fiche de copropriétaire n'est rattachée à ce compte : \
                          voter à une assemblée est réservé aux copropriétaires.",
                "kind": "owner_not_linked"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to resolve owner for user: {}", e)
            }));
        }
    };

    match state
        .resolution_use_cases
        .cast_vote(
            *resolution_id,
            request.owner_id,
            request.unit_id,
            request.vote_choice.clone(),
            // `request.voting_power` n'est PAS transmis : le cas d'usage relit
            // la quotité du lot. Accepter ici une puissance déclarée sans s'en
            // servir ressemblerait à un contrôle — c'est exactement ce que #850
            // reproche à cette route.
            request.proxy_owner_id,
            request.auth_method,
            caller_owner_id,
        )
        .await
    {
        Ok(vote) => {
            AuditLogEntry::new(
                AuditEventType::VoteCast,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Vote", vote.id)
            .with_metadata(serde_json::json!({
                "resolution_id": *resolution_id,
                "vote_choice": format!("{:?}", vote.vote_choice)
            }))
            .log();

            HttpResponse::Created().json(VoteResponse::from(vote))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::VoteCast,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            // Le refus de l'Art. 3.87 § 1er porte son code, et repart en 422.
            //
            // `AppError::VotingRightSuspended` existe, avec son statut 422 et
            // son payload `{code, unit_id}` que l'interface consomme pour dire
            // au syndic QUEL lot est concerné. Le pont
            // `From<VotingRightSuspendedError> for String` existe aussi, et son
            // commentaire annonce un préfixe « parsable par le gate vote ».
            //
            // Les deux bouts étaient écrits. Personne ne les avait reliés : le
            // refus repartait en 400 avec une chaîne plate, et le badge du
            // frontend n'avait jamais rien à consommer. Mesuré en exerçant le
            // scénario d'indivision de #848 pour la première fois.
            //
            // Classer par préfixe est ce que #762 veut faire disparaître. Ici
            // le préfixe a été posé EXPRÈS comme pont vers `Result<_, String>`,
            // et s'en passer suppose de typer l'erreur du cas d'usage — soit la
            // migration des 1263 `Result<_, String>` de #555.
            if let Some(reste) = err.strip_prefix("VOTING_RIGHT_SUSPENDED: unit ") {
                if let Ok(unit_id) = uuid::Uuid::parse_str(reste.trim()) {
                    return crate::application::error::AppError::VotingRightSuspended { unit_id }
                        .error_response();
                }
            }

            // Story 4.2 (#48) — `auth_method` absent (422) ou insuffisant pour
            // le mode distanciel de l'AG (403). Même geste que
            // VOTING_RIGHT_SUSPENDED ci-dessus : le préfixe posé côté cas
            // d'usage (`application/error.rs::From<VoteAuthError> for String`)
            // est reconnu ici pour reconstruire l'erreur typée.
            if err == "VOTE_AUTH_METHOD_REQUIRED" {
                return crate::application::error::AppError::VoteAuthMethodRequired
                    .error_response();
            }
            if let Some(reste) = err.strip_prefix("VOTE_AUTH_INSUFFICIENT:") {
                let mut parts = reste.splitn(2, ':');
                if let (Some(mode), Some(auth_method)) = (parts.next(), parts.next()) {
                    return crate::application::error::AppError::VoteAuthInsufficient {
                        mode: mode.to_string(),
                        auth_method: auth_method.to_string(),
                    }
                    .error_response();
                }
            }

            // #850 — usurpation d'identité ou de lot : 403, pas 400. Le préfixe
            // `FORBIDDEN` posé côté cas d'usage est reconnu par `est_interdit`
            // (déjà le lexique bilingue partagé par les autres gestionnaires).
            if est_interdit(&err) {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": err}));
            }

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/resolutions/{resolution_id}/votes",
    tag = "Resolutions",
    summary = "List all votes for a resolution",
    params(
        ("resolution_id" = Uuid, Path, description = "Resolution UUID")
    ),
    responses(
        (status = 200, description = "List of votes"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/resolutions/{resolution_id}/votes")]
pub async fn list_resolution_votes(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
) -> impl Responder {
    // On remonte à l'AG par la résolution, puis on applique le même mandat.
    let Ok(Some(resolution)) = state
        .resolution_use_cases
        .get_resolution(*resolution_id)
        .await
    else {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Resolution not found"
        }));
    };
    if let Some(refus) = verifier_mandat_sur_ag(&state, &user, resolution.meeting_id).await {
        return refus;
    }
    match state
        .resolution_use_cases
        .get_resolution_votes(*resolution_id)
        .await
    {
        Ok(votes) => {
            let responses: Vec<VoteResponse> = votes.into_iter().map(VoteResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    put,
    path = "/votes/{vote_id}",
    tag = "Resolutions",
    summary = "Change an existing vote",
    params(
        ("vote_id" = Uuid, Path, description = "Vote UUID")
    ),
    request_body = ChangeVoteRequest,
    responses(
        (status = 200, description = "Vote changed"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/votes/{vote_id}")]
pub async fn change_vote(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    vote_id: web::Path<Uuid>,
    request: web::Json<ChangeVoteRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .resolution_use_cases
        .change_vote(*vote_id, request.vote_choice.clone())
        .await
    {
        Ok(vote) => {
            AuditLogEntry::new(
                AuditEventType::VoteChanged,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Vote", vote.id)
            .with_metadata(serde_json::json!({
                "new_choice": format!("{:?}", vote.vote_choice)
            }))
            .log();

            HttpResponse::Ok().json(VoteResponse::from(vote))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::VoteChanged,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    put,
    path = "/resolutions/{resolution_id}/close",
    tag = "Resolutions",
    summary = "Close voting on a resolution and calculate result",
    params(
        ("resolution_id" = Uuid, Path, description = "Resolution UUID")
    ),
    request_body = CloseVotingRequest,
    responses(
        (status = 200, description = "Voting closed and result calculated"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/resolutions/{resolution_id}/close")]
pub async fn close_voting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
    // Corps accepté mais inutilisé : voir le commentaire du calcul ci-dessous.
    // On le garde dans la signature pour continuer d'accepter les appelants
    // qui envoient encore `total_voting_power`, sans jamais s'en servir.
    _request: web::Json<CloseVotingRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Le dénominateur de la majorité est lu SUR L'IMMEUBLE, jamais reçu du
    // client.
    //
    // `close_voting` le prenait dans le corps de la requête. Deux défauts d'un
    // coup : le frontend envoyait `{}`, donc la désérialisation échouait en 400
    // et le bouton « Clôturer le vote » paraissait inerte — rapporté trois fois
    // en recette (R3-3, RN-8), et deuxième des trois verrous qui empêchent une
    // AG d'aboutir (#780). Et surtout, un client qui aurait fourni un total
    // erroné faisait proclamer une majorité qui n'existe pas.
    //
    // Les tantièmes de l'acte de base sont une donnée de l'immeuble. C'est là
    // qu'on les prend.
    let total_voting_power = {
        let Ok(Some(resolution)) = state
            .resolution_use_cases
            .get_resolution(*resolution_id)
            .await
        else {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Resolution not found"
            }));
        };
        let Ok(Some(meeting)) = state
            .meeting_use_cases
            .get_meeting(resolution.meeting_id)
            .await
        else {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Meeting not found"
            }));
        };
        let Ok(Some(building)) = state
            .building_use_cases
            .get_building(meeting.building_id)
            .await
        else {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Building not found"
            }));
        };
        rust_decimal::Decimal::from(building.total_tantiemes)
    };

    match state
        .resolution_use_cases
        .close_voting(*resolution_id, total_voting_power)
        .await
    {
        Ok(resolution) => {
            AuditLogEntry::new(
                AuditEventType::VotingClosed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Resolution", resolution.id)
            .with_metadata(serde_json::json!({
                "final_status": format!("{:?}", resolution.status)
            }))
            .log();

            HttpResponse::Ok().json(ResolutionResponse::from(resolution))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::VotingClosed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/meetings/{meeting_id}/vote-summary",
    tag = "Resolutions",
    summary = "Get vote summary for a meeting",
    params(
        ("meeting_id" = Uuid, Path, description = "Meeting UUID")
    ),
    responses(
        (status = 200, description = "Vote summary for all meeting resolutions"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/meetings/{meeting_id}/vote-summary")]
pub async fn get_meeting_vote_summary(
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
        .resolution_use_cases
        .get_meeting_vote_summary(*meeting_id)
        .await
    {
        Ok(resolutions) => {
            let responses: Vec<ResolutionResponse> = resolutions
                .into_iter()
                .map(ResolutionResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
