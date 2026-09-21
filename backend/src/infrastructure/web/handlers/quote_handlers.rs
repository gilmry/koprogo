use crate::application::dto::{
    CreateQuoteDto, QuoteComparisonRequestDto, QuoteDecisionDto, SubmitQuoteDto,
};
use crate::infrastructure::web::middleware::scope_guard::{
    verify_building_org_access, verify_quote_org_access,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// POST /api/v1/quotes
/// Create new quote request (Syndic action)
#[post("/quotes")]
pub async fn create_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateQuoteDto>,
) -> impl Responder {
    // Cloisonnement AVANT la création (#864).
    //
    // L'identité était nommée `_auth` : le souligné disait explicitement
    // qu'on ne s'en servait pas, et le cas d'usage ne la reçoit même pas —
    // `create_quote(dto)` ne prend que le DTO. N'importe quel utilisateur
    // authentifié pouvait donc demander un devis sur l'immeuble de n'importe
    // quelle copropriété, en connaissant son UUID.
    //
    // Trois autres routes de ce fichier appellent déjà
    // `verify_building_org_access` (lignes 82, 138, 479). La création,
    // c'est-à-dire le seul geste qui INSCRIT quelque chose au patrimoine
    // d'une ACP, ne l'appelait pas.
    let building_id = match Uuid::parse_str(&request.building_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid building_id format"
            }))
        }
    };
    if let Err(err) = verify_building_org_access(
        &auth,
        building_id,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data
        .quote_use_cases
        .create_quote(request.into_inner())
        .await
    {
        Ok(quote) => HttpResponse::Created().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/quotes/:id
/// Get quote by ID
#[get("/quotes/{id}")]
pub async fn get_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement : ce devis doit relever d'une ACP que cet utilisateur a le
    // droit de voir. La route n'ayant pas d'immeuble en chemin, la chaîne
    // devis → immeuble → ACP est remontée par le garde.
    //
    // Ce contrôle porte sur le PÉRIMÈTRE, pas sur le droit d'agir : il dit que
    // ce devis n'est pas celui d'une autre copropriété, pas que cet
    // utilisateur a qualité pour l'examiner (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data.quote_use_cases.get_quote(id.into_inner()).await {
        Ok(Some(quote)) => HttpResponse::Ok().json(quote),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Quote not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes
/// List all quotes for a building
#[get("/buildings/{building_id}/quotes")]
pub async fn list_building_quotes(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement : l'immeuble visé doit relever d'une ACP que cet
    // utilisateur a le droit de voir. Un devis dit qui a soumis quel prix pour
    // quels travaux — le lire hors de son ACP, c'est lire la concurrence.
    // L'identité était prise puis ignorée — `_auth` (#772).
    if let Err(err) = verify_building_org_access(
        &auth,
        *building_id,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data
        .quote_use_cases
        .list_by_building(building_id.into_inner())
        .await
    {
        Ok(quotes) => HttpResponse::Ok().json(quotes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/contractors/:contractor_id/quotes
/// List all quotes for a contractor
///
/// Cloisonnement (#882) : classée, non corrigée. `list_by_contractor` rend
/// Les devis de ce prestataire, **restreints aux immeubles de l'appelant**.
///
/// La clé de cette route est le PRESTATAIRE, pas un immeuble : aucun garde
/// de ce fichier ne s'y appliquait directement, et elle rendait donc TOUS
/// ses devis, toutes ACP confondues — la même donnée (prix, projet) que
/// `list_building_quotes` protège déjà par immeuble.
///
/// Un cabinet syndic pouvait ainsi lire les prix qu'un prestataire avait
/// remis à un cabinet concurrent (#976).
///
/// Des deux voies envisagées — filtrer par les ACP visibles, ou restreindre
/// la route au superadministrateur — c'est la première qui est retenue : la
/// seconde priverait le syndic d'une lecture légitime, celle des devis de
/// SES immeubles.
#[get("/contractors/{contractor_id}/quotes")]
pub async fn list_contractor_quotes(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    contractor_id: web::Path<Uuid>,
) -> impl Responder {
    let quotes = match data
        .quote_use_cases
        .list_by_contractor(contractor_id.into_inner())
        .await
    {
        Ok(quotes) => quotes,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    };

    // ── Le filtrage par ACP visibles, retenu parmi les deux voies ──────────
    //
    // Le commentaire ci-dessus posait le choix : filtrer par les ACP visibles
    // de l'appelant, ou restreindre la route. C'est la première, parce que la
    // seconde retirerait au syndic une lecture légitime — les devis de SES
    // immeubles, remis par ce prestataire.
    //
    // Le paramètre s'appelait `_auth` : la convention Rust pour « je prends
    // cette identité et je ne m'en sers pas ». Il disait vrai, et c'était le
    // défaut — n'importe quel utilisateur authentifié lisait les devis de
    // n'importe quel prestataire, donc **les prix pratiqués chez un cabinet
    // concurrent** (#976).
    //
    // Chaque immeuble n'est vérifié qu'UNE fois : un prestataire remet
    // typiquement plusieurs devis sur la même copropriété, et interroger le
    // serveur par devis transformerait une lecture en rafale de requêtes.
    let mut vus: std::collections::HashMap<Uuid, bool> = std::collections::HashMap::new();
    let mut visibles = Vec::with_capacity(quotes.len());

    for devis in quotes {
        let building_id = match Uuid::parse_str(&devis.building_id) {
            Ok(id) => id,
            // Un identifiant illisible ne s'affiche pas « par défaut » : on
            // écarte, faute de pouvoir prouver que l'appelant y a droit.
            Err(_) => continue,
        };

        let autorise = match vus.get(&building_id) {
            Some(deja) => *deja,
            None => {
                let ok = verify_building_org_access(
                    &auth,
                    building_id,
                    &data.building_use_cases,
                    &data.acp_use_cases,
                )
                .await
                .is_ok();
                vus.insert(building_id, ok);
                ok
            }
        };

        if autorise {
            visibles.push(devis);
        }
    }

    HttpResponse::Ok().json(visibles)
}

/// GET /api/v1/buildings/:building_id/quotes/status/:status
/// List quotes by status
#[get("/buildings/{building_id}/quotes/status/{status}")]
pub async fn list_quotes_by_status(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    // Cloisonnement : l'immeuble visé doit relever d'une ACP que cet
    // utilisateur a le droit de voir. Un devis dit qui a soumis quel prix pour
    // quels travaux — le lire hors de son ACP, c'est lire la concurrence.
    // L'identité était prise puis ignorée — `_auth` (#772).
    if let Err(err) =
        verify_building_org_access(&auth, path.0, &data.building_use_cases, &data.acp_use_cases)
            .await
    {
        return err.error_response();
    }

    let (building_id, status) = path.into_inner();

    match data
        .quote_use_cases
        .list_by_status(building_id, &status)
        .await
    {
        Ok(quotes) => HttpResponse::Ok().json(quotes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/submit
/// Submit quote (Contractor/Syndic action). Body is optional: a quote that
/// already carries price data (cf. `CreateQuoteDto`'s escape hatch) can be
/// submitted bodyless — otherwise pricing is required in the body.
#[post("/quotes/{id}/submit")]
pub async fn submit_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    body: web::Bytes,
) -> impl Responder {
    // Cloisonnement : ce devis doit relever d'une ACP que cet utilisateur a le
    // droit de voir. La route n'ayant pas d'immeuble en chemin, la chaîne
    // devis → immeuble → ACP est remontée par le garde.
    //
    // Ce contrôle porte sur le PÉRIMÈTRE, pas sur le droit d'agir : il dit que
    // ce devis n'est pas celui d'une autre copropriété, pas que cet
    // utilisateur a qualité pour l'examiner (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    // No body = "confirm existing pricing" (use case rejects this unless the
    // quote already has a price). A non-empty body must parse as
    // SubmitQuoteDto — unlike Option<web::Json<T>>, a malformed body here
    // is a real 400 rather than silently falling back to "no pricing".
    let pricing = if body.is_empty() {
        None
    } else {
        match serde_json::from_slice::<SubmitQuoteDto>(&body) {
            Ok(dto) => Some(dto),
            Err(e) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid pricing payload: {}", e)
                }))
            }
        }
    };

    match data
        .quote_use_cases
        .submit_quote(id.into_inner(), pricing)
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/review
/// Start quote review (Syndic action)
#[post("/quotes/{id}/review")]
pub async fn start_review(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement : ce devis doit relever d'une ACP que cet utilisateur a le
    // droit de voir. La route n'ayant pas d'immeuble en chemin, la chaîne
    // devis → immeuble → ACP est remontée par le garde.
    //
    // Ce contrôle porte sur le PÉRIMÈTRE, pas sur le droit d'agir : il dit que
    // ce devis n'est pas celui d'une autre copropriété, pas que cet
    // utilisateur a qualité pour l'examiner (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data.quote_use_cases.start_review(id.into_inner()).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/accept
/// Accept quote (Syndic action - winner)
#[post("/quotes/{id}/accept")]
pub async fn accept_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<QuoteDecisionDto>,
) -> impl Responder {
    // Cloisonnement : accepter ou rejeter un devis engage l'ACP sur un marché.
    // Le contrôle porte sur le PÉRIMÈTRE — ce devis n'est pas celui d'une autre
    // copropriété — et non sur la qualité pour décider (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data
        .quote_use_cases
        .accept_quote(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/reject
/// Reject quote (Syndic action)
#[post("/quotes/{id}/reject")]
pub async fn reject_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<QuoteDecisionDto>,
) -> impl Responder {
    // Cloisonnement : accepter ou rejeter un devis engage l'ACP sur un marché.
    // Le contrôle porte sur le PÉRIMÈTRE — ce devis n'est pas celui d'une autre
    // copropriété — et non sur la qualité pour décider (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data
        .quote_use_cases
        .reject_quote(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/withdraw
/// Withdraw quote (Contractor action)
#[post("/quotes/{id}/withdraw")]
pub async fn withdraw_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement : ce devis doit relever d'une ACP que cet utilisateur a le
    // droit de voir. La route n'ayant pas d'immeuble en chemin, la chaîne
    // devis → immeuble → ACP est remontée par le garde.
    //
    // Ce contrôle porte sur le PÉRIMÈTRE, pas sur le droit d'agir : il dit que
    // ce devis n'est pas celui d'une autre copropriété, pas que cet
    // utilisateur a qualité pour l'examiner (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data.quote_use_cases.withdraw_quote(id.into_inner()).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/compare
/// Compare multiple quotes (Belgian professional best practice: 3 quotes minimum)
/// Returns quotes sorted by automatic score (best first)
#[post("/quotes/compare")]
pub async fn compare_quotes(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<QuoteComparisonRequestDto>,
) -> impl Responder {
    // Cloisonnement (#882) : chaque devis comparé doit relever d'une ACP que
    // cet utilisateur a le droit de voir. Le cas d'usage vérifie seulement que
    // les devis partagent le même immeuble — une cohérence métier, pas un
    // périmètre — donc lire la concurrence d'une autre copropriété suffisait
    // à obtenir ses prix et prestataires, en connaissant trois UUID de devis.
    for id_str in &request.quote_ids {
        let quote_id = match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid quote_id format: {}", id_str)
                }))
            }
        };
        if let Err(err) = verify_quote_org_access(
            &auth,
            quote_id,
            &data.quote_use_cases,
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
        {
            return err.error_response();
        }
    }

    match data
        .quote_use_cases
        .compare_quotes(request.into_inner())
        .await
    {
        Ok(comparison) => HttpResponse::Ok().json(comparison),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// PUT /api/v1/quotes/:id/contractor-rating
/// Update contractor rating (for scoring algorithm)
#[put("/quotes/{id}/contractor-rating")]
pub async fn update_contractor_rating(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<serde_json::Value>,
) -> impl Responder {
    // Cloisonnement : ce devis doit relever d'une ACP que cet utilisateur a le
    // droit de voir. La route n'ayant pas d'immeuble en chemin, la chaîne
    // devis → immeuble → ACP est remontée par le garde.
    //
    // Ce contrôle porte sur le PÉRIMÈTRE, pas sur le droit d'agir : il dit que
    // ce devis n'est pas celui d'une autre copropriété, pas que cet
    // utilisateur a qualité pour l'examiner (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    let rating = match request.get("rating").and_then(|v| v.as_i64()) {
        Some(r) => r as i32,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Rating field is required and must be an integer (0-100)"
            }))
        }
    };

    match data
        .quote_use_cases
        .update_contractor_rating(id.into_inner(), rating)
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// DELETE /api/v1/quotes/:id
/// Delete quote
#[delete("/quotes/{id}")]
pub async fn delete_quote(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement : ce devis doit relever d'une ACP que cet utilisateur a le
    // droit de voir. La route n'ayant pas d'immeuble en chemin, la chaîne
    // devis → immeuble → ACP est remontée par le garde.
    //
    // Ce contrôle porte sur le PÉRIMÈTRE, pas sur le droit d'agir : il dit que
    // ce devis n'est pas celui d'une autre copropriété, pas que cet
    // utilisateur a qualité pour l'examiner (#772).
    if let Err(err) = verify_quote_org_access(
        &auth,
        *id,
        &data.quote_use_cases,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data.quote_use_cases.delete_quote(id.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Quote not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes/count
/// Count total quotes for building
#[get("/buildings/{building_id}/quotes/count")]
pub async fn count_building_quotes(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement : l'immeuble visé doit relever d'une ACP que cet
    // utilisateur a le droit de voir. Un devis dit qui a soumis quel prix pour
    // quels travaux — le lire hors de son ACP, c'est lire la concurrence.
    // L'identité était prise puis ignorée — `_auth` (#772).
    if let Err(err) = verify_building_org_access(
        &auth,
        *building_id,
        &data.building_use_cases,
        &data.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data
        .quote_use_cases
        .count_by_building(building_id.into_inner())
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "count": count
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes/status/:status/count
/// Count quotes by status for building
#[get("/buildings/{building_id}/quotes/status/{status}/count")]
pub async fn count_quotes_by_status(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    // Cloisonnement : l'immeuble visé doit relever d'une ACP que cet
    // utilisateur a le droit de voir. Un devis dit qui a soumis quel prix pour
    // quels travaux — le lire hors de son ACP, c'est lire la concurrence.
    // L'identité était prise puis ignorée — `_auth` (#772).
    if let Err(err) =
        verify_building_org_access(&auth, path.0, &data.building_use_cases, &data.acp_use_cases)
            .await
    {
        return err.error_response();
    }

    let (building_id, status) = path.into_inner();

    match data
        .quote_use_cases
        .count_by_status(building_id, &status)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "count": count
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

#[cfg(test)]
mod tests {
    // Handler tests are covered by E2E tests in tests/e2e/

    #[test]
    fn test_handler_structure_quotes() {
        // This test verifies handler function signatures compile
        // Real testing happens in E2E tests with testcontainers
    }
}
