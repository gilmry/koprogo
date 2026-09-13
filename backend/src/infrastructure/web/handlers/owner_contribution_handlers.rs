use crate::application::dto::{
    CreateOwnerContributionRequest, OwnerContributionResponse, RecordPaymentRequest,
};
use crate::infrastructure::web::middleware::scope_guard::verify_contribution_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, ResponseError};
use uuid::Uuid;

/// Distingue, pour ce corps de requête, un JSON malformé (`@negative`
/// générique, 400 — le corps n'est pas exploitable) d'un JSON valide qui ne
/// respecte pas le schéma déclaré (422 — un champ requis manque, un champ
/// inconnu traîne, ou un type ne correspond pas).
///
/// Issue #852 : `CreateOwnerContributionRequest::unit_id` est redevenu
/// obligatoire, mais le refus qu'il provoque doit apprendre au client la
/// règle de la signature, pas se confondre avec « je n'ai pas su lire du
/// JSON ». D'où l'extraction en deux temps : `web::Json<Value>` capte encore
/// la syntaxe JSON et le Content-Type (comportement inchangé, 400 en cas de
/// souci) ; c'est seulement la conversion vers le type précis qui distingue
/// le 422 de validation.
fn parse_create_contribution_request(
    body: serde_json::Value,
) -> Result<CreateOwnerContributionRequest, HttpResponse> {
    serde_json::from_value(body).map_err(|e| {
        HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": "Validation error: the request body does not match the schema",
            "details": e.to_string(),
        }))
    })
}

/// POST /api/v1/owner-contributions
/// Create a new owner contribution
#[utoipa::path(
    post,
    path = "/owner-contributions",
    tag = "OwnerContributions",
    summary = "Create an owner contribution (quote-part)",
    request_body = CreateOwnerContributionRequest,
    responses(
        (status = 201, description = "Contribution created", body = OwnerContributionResponse),
        (status = 400, description = "Malformed JSON body, or wrong Content-Type"),
        (status = 401, description = "User does not belong to an organization"),
        (status = 422, description = "Body does not match the schema (e.g. unit_id missing) — see Issue #852"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/owner-contributions")]
pub async fn create_contribution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    // Get organization_id from user (required for creating contributions)
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "Organization ID required" }))
        }
    };

    let req = match parse_create_contribution_request(body.into_inner()) {
        Ok(req) => req,
        Err(response) => return response,
    };

    match state
        .owner_contribution_use_cases
        .create_contribution(
            organization_id,
            req.owner_id,
            Some(req.unit_id),
            req.description.clone(),
            req.amount,
            req.contribution_type.clone(),
            req.contribution_date,
            req.account_code.clone(),
        )
        .await
    {
        Ok(contribution) => {
            let response = OwnerContributionResponse::from(contribution);
            HttpResponse::Created().json(response)
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

#[cfg(test)]
mod create_contribution_parsing_tests {
    use super::*;

    fn corps_valide() -> serde_json::Value {
        serde_json::json!({
            "owner_id": Uuid::new_v4(),
            "unit_id": Uuid::new_v4(),
            "description": "Appel de fonds Q1 2026",
            "amount": "750.00",
            "contribution_type": "regular",
            "contribution_date": chrono::Utc::now().to_rfc3339(),
        })
    }

    // @happy — un corps complet, avec unit_id, se convertit sans erreur.
    #[test]
    fn happy_corps_avec_unit_id_est_accepte() {
        let resultat = parse_create_contribution_request(corps_valide());
        assert!(resultat.is_ok(), "corps valide attendu accepté");
    }

    // @negative — unit_id absent : 422, pas un 400 de syntaxe JSON générique
    // (Issue #852 — le client doit apprendre la règle de la signature, pas de
    // l'échec).
    #[test]
    fn negative_unit_id_absent_retourne_422() {
        let mut corps = corps_valide();
        corps.as_object_mut().unwrap().remove("unit_id");

        let erreur = parse_create_contribution_request(corps).expect_err("doit refuser");
        assert_eq!(erreur.status(), 422);
    }

    // @edge — un champ inconnu reste refusé (deny_unknown_fields), désormais
    // avec le même code 422 que les autres écarts de schéma.
    #[test]
    fn edge_champ_inconnu_retourne_422() {
        let mut corps = corps_valide();
        corps
            .as_object_mut()
            .unwrap()
            .insert("champ_fantaisiste".to_string(), serde_json::json!(true));

        let erreur = parse_create_contribution_request(corps).expect_err("doit refuser");
        assert_eq!(erreur.status(), 422);
    }

    // @security — aucun repli implicite : `unit_id` manquant fait échouer la
    // conversion, sans jamais construire de requête valide à partir d'un
    // autre champ (ex. `owner_id`) glissé à sa place. Le pendant HTTP complet
    // — le refus nomme bien `unit_id` — est vérifié par
    // `test_owner_contributions_create_missing_unit_id_returns_422`
    // (tests/e2e_owner_contributions.rs), qui lit la réponse via le harnais
    // actix éprouvé plutôt qu'en décodant le corps ici.
    #[test]
    fn security_unit_id_absent_ne_produit_jamais_de_requete_valide() {
        let mut corps = corps_valide();
        corps.as_object_mut().unwrap().remove("unit_id");

        assert!(
            parse_create_contribution_request(corps).is_err(),
            "sans unit_id, aucune CreateOwnerContributionRequest ne doit être construite"
        );
    }
}

/// GET /api/v1/owner-contributions/{id}
/// Get contribution by ID
#[utoipa::path(
    get,
    path = "/owner-contributions/{id}",
    tag = "OwnerContributions",
    summary = "Get a single owner contribution",
    params(("id" = Uuid, Path, description = "Contribution identifier")),
    responses(
        (status = 200, description = "Contribution", body = OwnerContributionResponse),
        (status = 404, description = "Contribution not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owner-contributions/{id}")]
pub async fn get_contribution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> HttpResponse {
    // Cloisonnement : cette quote-part relève d'une ACP précise (#772).
    if let Err(err) = verify_contribution_org_access(
        &user,
        *id,
        &state.owner_contribution_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match state
        .owner_contribution_use_cases
        .get_contribution(*id)
        .await
    {
        Ok(Some(contribution)) => {
            let response = OwnerContributionResponse::from(contribution);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Contribution not found" }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/owner-contributions?owner_id={uuid}
/// Get contributions by owner, or all contributions for organization if owner_id not provided
#[utoipa::path(
    get,
    path = "/owner-contributions",
    tag = "OwnerContributions",
    summary = "List contributions of the organization, or of a single owner",
    params(("owner_id" = Option<Uuid>, Query, description = "Restrict to one owner")),
    responses(
        (status = 200, description = "Contributions", body = Vec<OwnerContributionResponse>),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owner-contributions")]
pub async fn get_contributions_by_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    // If owner_id provided, filter by owner
    if let Some(id_str) = query.get("owner_id") {
        let owner_id = match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({ "error": "Invalid owner_id format" }))
            }
        };

        match state
            .owner_contribution_use_cases
            .get_contributions_by_owner(owner_id)
            .await
        {
            Ok(contributions) => {
                let responses: Vec<OwnerContributionResponse> =
                    contributions.into_iter().map(Into::into).collect();
                return HttpResponse::Ok().json(responses);
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e }))
            }
        }
    }

    // Otherwise, return all contributions for user's organization
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "Organization ID required" }))
        }
    };

    match state
        .owner_contribution_use_cases
        .get_contributions_by_organization(organization_id)
        .await
    {
        Ok(contributions) => {
            let responses: Vec<OwnerContributionResponse> =
                contributions.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/owner-contributions/outstanding?owner_id={uuid}
/// Get outstanding (unpaid) contributions for an owner
#[utoipa::path(
    get,
    path = "/owner-contributions/outstanding",
    tag = "OwnerContributions",
    summary = "List unpaid contributions",
    responses(
        (status = 200, description = "Outstanding contributions", body = Vec<OwnerContributionResponse>),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owner-contributions/outstanding")]
pub async fn get_outstanding_contributions(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let owner_id = match query.get("owner_id") {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({ "error": "Invalid owner_id format" }))
            }
        },
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "owner_id is required" }))
        }
    };

    match state
        .owner_contribution_use_cases
        .get_outstanding_contributions(owner_id)
        .await
    {
        Ok(contributions) => {
            let responses: Vec<OwnerContributionResponse> =
                contributions.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// PUT /api/v1/owner-contributions/{id}/mark-paid
/// Record payment for a contribution
#[utoipa::path(
    put,
    path = "/owner-contributions/{id}/mark-paid",
    tag = "OwnerContributions",
    summary = "Record a payment against a contribution",
    description = "Voie SUPPORTEE pour solder une quote-part depuis l'interface. \
Un paiement du module `/payments` peut aussi la solder automatiquement : \
il suffit de lui passer `contribution_id`, et la quote-part bascule quand \
le paiement atteint `succeeded`.",
    params(("id" = Uuid, Path, description = "Contribution identifier")),
    request_body = RecordPaymentRequest,
    responses(
        (status = 200, description = "Payment recorded", body = OwnerContributionResponse),
        (status = 400, description = "Already paid, or unknown field in the body"),
        (status = 404, description = "Contribution not found"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/owner-contributions/{id}/mark-paid")]
pub async fn record_payment(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    req: web::Json<RecordPaymentRequest>,
) -> HttpResponse {
    // Cloisonnement : déclarer qu'un copropriétaire a payé éteint une dette.
    // Sans ce contrôle, on pouvait le faire dans la comptabilité d'une autre
    // copropriété. L'identité était prise puis ignorée — `_user` (#772).
    if let Err(err) = verify_contribution_org_access(
        &user,
        *id,
        &state.owner_contribution_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match state
        .owner_contribution_use_cases
        .record_payment(
            *id,
            req.payment_date,
            req.payment_method.clone(),
            req.payment_reference.clone(),
        )
        .await
    {
        Ok(contribution) => {
            let response = OwnerContributionResponse::from(contribution);
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}
