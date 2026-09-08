//! `scope_guard` middleware — Story 1.3 (refonte UX multi-rôle ACP).
//!
//! Reads the optional scope hint provided by the client (header
//! `X-Scope-AcpId` *or* query parameter `?acp_id=...`), resolves the
//! caller's effective `AcpCaller` from JWT, and injects an `AcpScope`
//! into the request extensions for downstream handlers to consume.
//!
//! Refuses the request early (without hitting the handler) if the
//! caller attempts to address an ACP outside their scope.
//!
//! Design:
//! - The middleware is a thin `Transform` registered on `/buildings` /
//!   `/acps` routes; it consults `AppState::list_acps_use_case` to
//!   verify the scope.
//! - For routes that don't need a forced ACP scope (e.g. admin listing
//!   "all"), the middleware is permissive: no scope hint → no
//!   restriction, the use-case will fall back to the role-derived
//!   default scope.
//!
//! Error semantics (cf. architecture §6.3) :
//! - 401 `Unauthorized` if no/invalid JWT
//! - 403 `AcpNotInScope { acp_id }` if scope forged out of perimeter
//! - 400 `Validation` if header/query are malformed *and* the role
//!   needs an explicit scope id (non-admin)

use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;

use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    web, Error, HttpMessage, HttpResponse, ResponseError,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::use_cases::acp_use_cases::{AcpCaller, AcpUseCases};
use crate::application::use_cases::building_use_cases::BuildingUseCases;
use crate::application::use_cases::call_for_funds_use_cases::CallForFundsUseCases;
use crate::application::use_cases::convocation_use_cases::ConvocationUseCases;
use crate::application::use_cases::document_use_cases::DocumentUseCases;
use crate::application::use_cases::local_exchange_use_cases::LocalExchangeUseCases;
use crate::application::use_cases::owner_contribution_use_cases::OwnerContributionUseCases;
use crate::application::use_cases::owner_use_cases::OwnerUseCases;
use crate::application::use_cases::poll_use_cases::PollUseCases;
use crate::application::use_cases::quote_use_cases::QuoteUseCases;
use crate::application::use_cases::resource_booking_use_cases::ResourceBookingUseCases;
use crate::application::use_cases::technical_spec_use_cases::TechnicalSpecUseCases;
use crate::application::use_cases::ticket_use_cases::TicketUseCases;
use crate::application::use_cases::unit_use_cases::UnitUseCases;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::AuthenticatedUser;

/// Header name accepted as a scope hint. Case-insensitive per HTTP RFC.
pub const SCOPE_ACP_HEADER: &str = "X-Scope-AcpId";

/// Resolved scope context, injected into request extensions by the
/// `ScopeGuard` middleware. Handlers read it via
/// `req.extensions().get::<AcpScope>()` (or via an extractor in a
/// follow-up story).
#[derive(Debug, Clone)]
pub struct AcpScope {
    /// Caller derived from JWT (mapped by the same convention as
    /// `acp_handlers::caller_from_user`).
    pub caller: AcpCaller,
    /// ACP id explicitly requested by the client (header/query).
    /// `None` = use role-derived default scope.
    pub requested_acp_id: Option<Uuid>,
    /// `true` if the middleware has verified the caller is allowed to
    /// see the requested ACP. Always `true` when `requested_acp_id` is
    /// `None` (no forging possible).
    pub allowed: bool,
}

/// Errors surfaced by `ScopeGuard`. Mapped to HTTP via `ResponseError`.
#[derive(Debug, Error)]
pub enum ScopeGuardError {
    #[error("Unauthorized — missing or invalid JWT")]
    Unauthorized,

    #[error("ACP {acp_id} not in scope")]
    AcpNotInScope { acp_id: Uuid },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ScopeGuardError {
    pub fn kind(&self) -> &'static str {
        match self {
            ScopeGuardError::Unauthorized => "unauthorized",
            ScopeGuardError::AcpNotInScope { .. } => "acp_not_in_scope",
            ScopeGuardError::Validation(_) => "validation",
            ScopeGuardError::Internal(_) => "internal",
        }
    }
}

impl ResponseError for ScopeGuardError {
    fn status_code(&self) -> StatusCode {
        match self {
            ScopeGuardError::Unauthorized => StatusCode::UNAUTHORIZED,
            ScopeGuardError::AcpNotInScope { .. } => StatusCode::FORBIDDEN,
            ScopeGuardError::Validation(_) => StatusCode::BAD_REQUEST,
            ScopeGuardError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": self.to_string(),
            "kind": self.kind(),
        }))
    }
}

impl From<AppError> for ScopeGuardError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::AcpNotInScope { acp_id } => ScopeGuardError::AcpNotInScope { acp_id },
            AppError::Unauthorized | AppError::InvalidCredentials | AppError::TokenError(_) => {
                ScopeGuardError::Unauthorized
            }
            AppError::Validation(s) => ScopeGuardError::Validation(s),
            other => ScopeGuardError::Internal(other.to_string()),
        }
    }
}

// ============================================================================
// Pure helpers (used by middleware AND tested by BDD/unit tests without
// actix machinery).
// ============================================================================

/// Map a `UserRoleString + organization_id + user_id` triple to an
/// `AcpCaller`. Same convention as `acp_handlers::caller_from_user` —
/// duplicated here to keep the middleware free of handler imports.
pub fn caller_from_role(role: &str, organization_id: Option<Uuid>, user_id: Uuid) -> AcpCaller {
    match role.to_lowercase().as_str() {
        "superadmin" => AcpCaller::SuperAdmin,
        "admin" => match organization_id {
            Some(org) => AcpCaller::Admin {
                organization_id: org,
            },
            None => AcpCaller::SuperAdmin,
        },
        "syndic" | "accountant" => match organization_id {
            Some(org) => AcpCaller::Syndic {
                organization_id: org,
            },
            None => AcpCaller::Owner { user_id },
        },
        _ => AcpCaller::Owner { user_id },
    }
}

/// Extract the requested ACP id from headers or query string.
/// Header `X-Scope-AcpId` takes precedence over `?acp_id=`.
/// Returns `Err(Validation)` if a value is present but malformed.
pub fn extract_requested_acp_id(
    header_value: Option<&str>,
    query_value: Option<&str>,
) -> Result<Option<Uuid>, ScopeGuardError> {
    let raw = header_value.or(query_value);
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Uuid::parse_str(s.trim())
            .map(Some)
            .map_err(|_| ScopeGuardError::Validation(format!("invalid acp_id: {}", s))),
    }
}

/// Decide whether the caller is allowed to *attach* the requested scope
/// id, without hitting the DB. Returns:
/// - `Ok(None)` : caller has no requested scope → no enforcement needed
/// - `Ok(Some(acp_id))` : the middleware must consult the use-case to
///   verify `assert_caller_can_see(acp_id)`
/// - `Err(Validation)` : the caller is non-admin AND owns no role-scope
///   information (e.g. syndic with `organization_id = None` AND no
///   explicit acp_id) — refuse rather than guess.
pub fn requires_repository_check(
    caller: &AcpCaller,
    requested: Option<Uuid>,
) -> Result<Option<Uuid>, ScopeGuardError> {
    match (caller, requested) {
        // SuperAdmin can pin any ACP, but we still want to verify it exists.
        (AcpCaller::SuperAdmin, Some(id)) => Ok(Some(id)),
        (AcpCaller::SuperAdmin, None) => Ok(None),

        // Admin / Syndic with their own org are allowed if requested
        // matches their org-derived scope or is None.
        (AcpCaller::Admin { .. }, req) | (AcpCaller::Syndic { .. }, req) => Ok(req),

        // Owner: every request must be checked via the use-case (no
        // direct shortcut — story 1.3 conservatively refuses pinning
        // until story 3.5 wires user_role_assignments.scope/scope_id).
        (AcpCaller::Owner { .. }, req) => Ok(req),
    }
}

/// Hotfix #603 — résout `building.acp_id -> acp.organization_id` et applique
/// l'isolation multi-tenant sur les GET-by-id (building, budget, expense,
/// meeting, resolution, unit, work_report).
///
/// Après #602 (`Building.organization_id -> acp_id`), `BuildingResponseDto`
/// ne porte plus `organization_id` ; les 7 handlers ci-dessus ont perdu leur
/// `user.verify_org_access(...)`. Ce helper recâble la chaîne en lookup ACP.
///
/// Sémantique :
/// - SuperAdmin : toujours autorisé (bypass).
/// - Sinon : `acp.organization_id` MUST == `user.organization_id`. Sinon
///   `AppError::AcpNotInScope` (HTTP 403 via `ResponseError`).
/// - ACP introuvable OU `acp.organization_id IS NULL` (auto-gérée) :
///   refuse pour non-superadmin (conservateur — gouvernance ACP auto-gérée
///   en story 4.x).
pub async fn verify_acp_org_access(
    user: &AuthenticatedUser,
    acp_id: Uuid,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let acp = acp_use_cases
        .find_acp(acp_id)
        .await?
        .ok_or(AppError::AcpNotInScope { acp_id })?;

    let acp_org_id = acp
        .organization_id
        .ok_or(AppError::AcpNotInScope { acp_id })?;

    user.verify_org_access(acp_org_id)
        .map_err(|_| AppError::AcpNotInScope { acp_id })
}

/// Isolation multi-tenant sur les ÉCRITURES qui désignent un immeuble par le
/// CORPS de la requête.
///
/// `verify_acp_org_access` ci-dessus protège les lectures par identifiant
/// (Hotfix #603). Le côté écriture n'avait pas d'équivalent : 29 routes
/// `POST`/`PUT` acceptent un `building_id` dans leur DTO et aucune ne
/// vérifiait qu'il appartient à l'organisation de l'appelant.
///
/// Mesuré en conditions réelles le 2026-09-02 entre deux cabinets syndics
/// indépendants. Le cabinet B a pu, sur l'immeuble du cabinet A :
///
///   POST /expenses          → 201, dépense de 50 000 € VISIBLE dans la
///                             liste des charges de l'immeuble de A ;
///   POST /call-for-funds    → 201, puis `send` → 200, générant une
///                             quote-part de 25 000 € réclamée à une
///                             copropriétaire de A.
///
/// Les lectures, elles, répondaient bien 403 dans toutes les directions : la
/// faille était strictement du côté écriture.
///
/// La garde existante `dto.organization_id = <celle du JWT>` ne protège pas
/// de cela — elle empêche d'ESTAMPILLER l'enregistrement au nom d'autrui, pas
/// de le RATTACHER au patrimoine d'autrui. Le champ contrôlé n'était pas le
/// bon.
///
/// Sémantique : superadmin passe ; sinon l'immeuble doit exister et son ACP
/// appartenir à l'organisation de l'appelant. Un immeuble introuvable est
/// refusé plutôt qu'ignoré — accepter une écriture pointant vers le vide
/// créerait un orphelin invisible.
pub async fn verify_building_org_access(
    user: &AuthenticatedUser,
    building_id: Uuid,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let building = building_use_cases
        .get_building(building_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!(
            "Building not found: {building_id}"
        )))?;

    let acp_id = Uuid::parse_str(&building.acp_id)
        .map_err(|_| AppError::Internal("Invalid building.acp_id format".to_string()))?;

    verify_acp_org_access(user, acp_id, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'ACP dont relève une **convocation**.
///
/// La convocation porte directement son `building_id` : le saut est unique.
///
/// `GET /convocations/{id}/recipients` sert la liste NOMINATIVE des
/// copropriétaires convoqués, avec leur adresse de courriel et le mode d'envoi
/// retenu. C'est un fichier de personnes, et il était servi à qui connaissait
/// un identifiant de convocation.
///
/// `tracking-summary` en dit davantage encore : qui a ouvert le courriel et
/// quand. Une donnée de comportement, pas seulement d'identité.
pub async fn verify_convocation_org_access(
    user: &AuthenticatedUser,
    convocation_id: Uuid,
    convocation_use_cases: &ConvocationUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let convocation = convocation_use_cases
        .get_convocation(convocation_id)
        .await
        .map_err(AppError::from)?;

    verify_building_org_access(
        user,
        convocation.building_id,
        building_use_cases,
        acp_use_cases,
    )
    .await
}

/// Vérifie le mandat de l'appelant sur l'ACP dont relève un **document**.
///
/// Remonte document → immeuble → ACP → organisation.
///
/// Un document de copropriété n'est pas un fichier anodin : l'acte de base,
/// les procès-verbaux d'assemblée et les factures nominatives passent par là.
/// `GET /documents/{id}/download` servait le contenu même du fichier à qui
/// connaissait son identifiant.
///
/// Les deux routes de rattachement — vers une assemblée, vers une dépense —
/// sont des **écritures** : elles permettaient de raccrocher le document d'un
/// cabinet au dossier d'un autre.
pub async fn verify_document_org_access(
    user: &AuthenticatedUser,
    document_id: Uuid,
    document_use_cases: &DocumentUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let document = document_use_cases
        .get_document(document_id)
        .await
        .map_err(AppError::from)?;

    verify_building_org_access(
        user,
        document.building_id,
        building_use_cases,
        acp_use_cases,
    )
    .await
}

/// Vérifie le mandat de l'appelant sur l'ACP dont relève un **lot**.
///
/// Remonte lot → immeuble → ACP → organisation.
///
/// `GET /units/{id}/etats-dates` sert les états datés d'un lot : le document
/// remis au notaire lors d'une vente, qui porte les arriérés du vendeur et
/// l'état du fonds de réserve. C'est une pièce financière nominative.
/// Vérifie le mandat de l'appelant sur l'organisation d'une **réservation**.
///
/// ── Ce que cette garde protège ─────────────────────────────────────────────
///
/// Une réservation de ressource commune dit qui a réservé la salle, la buanderie
/// ou le parking visiteur, et **quand**. Ce n'est pas une donnée neutre : elle
/// dit aussi qui n'était pas chez lui à ce moment-là.
///
/// Trois routes agissent sur une réservation par son seul identifiant —
/// `complete`, `no-show`, `confirm` — et deux la lisent. Aucune ne reçoit
/// d'immeuble : la chaîne réservation → immeuble → ACP → organisation doit
/// donc être remontée ici.
///
/// Elles prenaient `_auth: AuthenticatedUser`, l'identité soulignée d'un
/// underscore pour dire qu'on ne s'en sert pas (#772).
///
/// ── Pourquoi elle délègue plutôt que de comparer ──────────────────────────
///
/// Comme les huit autres, elle finit par appeler `verify_building_org_access`,
/// qui remonte à l'ACP. Refaire la comparaison ici dupliquerait la règle de
/// cloisonnement en un endroit de plus — et c'est cette duplication, recopiée
/// à la main dans chaque gestionnaire, que l'issue #772 désigne comme la cause
/// première de la fuite.
/// Vérifie le mandat de l'appelant sur l'organisation d'un **devis**.
///
/// ── Ce que cette garde protège ─────────────────────────────────────────────
///
/// Un devis dit qui a soumis quel prix pour quels travaux. Le lire hors de son
/// ACP, c'est lire la concurrence — un entrepreneur ayant un compte sur la
/// plateforme y verrait les offres de ses concurrents, montants compris.
///
/// Neuf routes agissent sur un devis par son seul identifiant : le lire, le
/// soumettre, l'examiner, le retirer, le noter, le supprimer. Aucune ne reçoit
/// d'immeuble, d'où la remontée devis → immeuble → ACP → organisation.
///
/// ── Ce qu'elle ne fait PAS ────────────────────────────────────────────────
///
/// Elle vérifie le **périmètre**, pas le droit d'agir. Examiner un devis
/// (`review`) ou noter un prestataire n'appartient pas à tout membre de l'ACP —
/// ce sont des actes de gestion. Ce garde ne dit donc pas « cet utilisateur
/// peut le faire », il dit « ce devis n'est pas celui d'une autre
/// copropriété ».
///
/// Confondre les deux serait le défaut que l'issue #772 décrit : un garde qui
/// vérifie la mauvaise chose est pire qu'un garde absent, puisqu'il fait
/// croire la route protégée.
pub async fn verify_quote_org_access(
    user: &AuthenticatedUser,
    quote_id: Uuid,
    quote_use_cases: &QuoteUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let quote = quote_use_cases
        .get_quote(quote_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!("Quote not found: {quote_id}")))?;

    let building_id = Uuid::parse_str(&quote.building_id)
        .map_err(|_| AppError::Internal("Invalid quote.building_id format".to_string()))?;

    verify_building_org_access(user, building_id, building_use_cases, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'organisation d'un **échange local**.
///
/// ── Ce que cette garde protège ─────────────────────────────────────────────
///
/// Le système d'échange local — le SEL — enregistre qui rend quel service à
/// qui, et pour combien de crédits. Six routes agissent sur un échange par son
/// seul identifiant : le demander, le démarrer, le clore, l'annuler, noter le
/// prestataire, noter le demandeur.
///
/// Les deux dernières comptent particulièrement : une note engage la
/// réputation d'un voisin dans sa propre copropriété. La poser depuis une
/// autre ACP n'a aucun sens légitime.
///
/// ── Pourquoi elle délègue ─────────────────────────────────────────────────
///
/// Comme les dix autres gardes du module, elle remonte jusqu'à l'immeuble puis
/// confie la comparaison à `verify_building_org_access`. C'est la duplication
/// de cette comparaison, recopiée à la main dans chaque gestionnaire, que
/// l'issue #772 désigne comme la cause première de la fuite.
pub async fn verify_exchange_org_access(
    user: &AuthenticatedUser,
    exchange_id: Uuid,
    exchange_use_cases: &LocalExchangeUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let exchange = exchange_use_cases
        .get_exchange(exchange_id)
        .await
        .map_err(AppError::from)?;

    verify_building_org_access(
        user,
        exchange.building_id,
        building_use_cases,
        acp_use_cases,
    )
    .await
}

/// Vérifie le mandat de l'appelant sur l'organisation d'un **ticket**.
///
/// ── Ce que cette garde ajoute à ce qui existait ───────────────────────────
///
/// `syndic_response_handlers` appelait déjà `require_syndic_or_superadmin`, et
/// ce contrôle est juste : répondre à un ticket est un acte de gestion, pas de
/// copropriétaire.
///
/// Mais il vérifie le RÔLE, et rien d'autre. Un syndic de l'organisation A y
/// passait pour répondre au ticket d'un copropriétaire de l'organisation B —
/// et sa réponse s'y inscrivait, signée de son nom.
///
/// Les deux contrôles sont donc nécessaires et ne se remplacent pas : l'un dit
/// « vous avez qualité pour cela », l'autre « ce ticket est bien le vôtre ».
/// C'est la distinction que l'issue #772 demande de tenir, et que le tableau
/// de bord du conseil (#816) illustrait déjà.
pub async fn verify_ticket_org_access(
    user: &AuthenticatedUser,
    ticket_id: Uuid,
    ticket_use_cases: &TicketUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let ticket = ticket_use_cases
        .get_ticket(ticket_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!("Ticket not found: {ticket_id}")))?;

    verify_building_org_access(user, ticket.building_id, building_use_cases, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'ACP d'une **fiche technique**.
///
/// ── Pourquoi l'ACP, et non l'immeuble ─────────────────────────────────────
///
/// `TechnicalSpec` porte `acp_id: Uuid` **obligatoire** et
/// `building_id: Option<Uuid>`. Une fiche peut donc concerner l'ACP entière
/// sans viser un immeuble précis — un marché d'entretien couvrant tout le
/// patrimoine, par exemple.
///
/// Remonter par l'immeuble aurait laissé sans contrôle toutes les fiches dont
/// il est absent. Le périmètre juste est celui que l'entité rend obligatoire,
/// et c'est l'ACP.
///
/// C'est le genre de choix qu'on ne peut pas recopier d'un garde voisin : les
/// onze autres remontent à l'immeuble parce que leurs entités le portent
/// toujours. Ici, suivre le modèle aurait produit un trou pour les fiches sans
/// immeuble — soit exactement les plus larges (#772).
pub async fn verify_technical_spec_org_access(
    user: &AuthenticatedUser,
    spec_id: Uuid,
    spec_use_cases: &TechnicalSpecUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let spec = spec_use_cases.get(spec_id).await?;
    verify_acp_org_access(user, spec.acp_id, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'ACP d'un **appel de fonds**.
///
/// Un appel de fonds engage l'argent des copropriétaires : il dit combien
/// chacun doit, et pour quoi. `POST /call-for-funds/{id}/send` le leur envoie
/// — un acte qui, hors de son ACP, écrirait à des personnes qu'on n'a pas à
/// contacter, au nom d'une copropriété qui n'est pas la sienne.
///
/// Le périmètre est l'ACP : `CallForFunds` porte `acp_id` obligatoire et ne
/// vise pas d'immeuble. C'est cohérent avec l'Art. 3.86, où le fonds de
/// roulement et le fonds de réserve appartiennent à l'association, non aux
/// immeubles qu'elle regroupe.
/// Vérifie le mandat de l'appelant sur l'ACP d'une **quote-part**.
///
/// `PUT /owner-contributions/{id}/mark-paid` déclare qu'un copropriétaire
/// nommé a payé. C'est l'écriture la plus lourde de conséquence du produit
/// après la clôture d'un vote : elle éteint une dette, et son absence de
/// contrôle permettait de le faire dans la comptabilité d'une autre
/// copropriété.
///
/// Le périmètre est l'ACP : `OwnerContribution` porte `acp_id` obligatoire et
/// `unit_id: Option<Uuid>` — une quote-part peut n'être rattachée à aucun lot
/// précis, notamment lors d'une régularisation.
pub async fn verify_contribution_org_access(
    user: &AuthenticatedUser,
    contribution_id: Uuid,
    contribution_use_cases: &OwnerContributionUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let contribution = contribution_use_cases
        .get_contribution(contribution_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!(
            "Contribution not found: {contribution_id}"
        )))?;

    verify_acp_org_access(user, contribution.acp_id, acp_use_cases).await
}

pub async fn verify_call_for_funds_org_access(
    user: &AuthenticatedUser,
    cff_id: Uuid,
    cff_use_cases: &CallForFundsUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let cff = cff_use_cases
        .get_call_for_funds(cff_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!(
            "Call for funds not found: {cff_id}"
        )))?;

    verify_acp_org_access(user, cff.acp_id, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'organisation d'un **sondage**.
///
/// Un sondage recueille l'avis des copropriétaires sur une question qui les
/// concerne. Quatre routes agissent sur lui par son seul identifiant : lire
/// ses résultats, le publier, le clore, l'annuler.
///
/// Lire les résultats d'un sondage d'une autre ACP, c'est apprendre ce que des
/// voisins qui ne sont pas les vôtres pensent d'un sujet qui ne vous regarde
/// pas. Le publier ou le clore depuis l'extérieur serait pire : cela
/// interromprait une consultation en cours.
///
/// Le périmètre est l'immeuble — `Poll.building_id` est obligatoire — et non
/// l'ACP, contrairement aux appels de fonds et aux fiches techniques. Une
/// consultation porte sur la vie d'un bâtiment, pas sur le patrimoine d'une
/// association.
pub async fn verify_poll_org_access(
    user: &AuthenticatedUser,
    poll_id: Uuid,
    poll_use_cases: &PollUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let poll = poll_use_cases
        .get_poll(poll_id)
        .await
        .map_err(AppError::from)?;

    let building_id = Uuid::parse_str(&poll.building_id)
        .map_err(|_| AppError::Internal("Invalid poll.building_id format".to_string()))?;

    verify_building_org_access(user, building_id, building_use_cases, acp_use_cases).await
}

pub async fn verify_booking_org_access(
    user: &AuthenticatedUser,
    booking_id: Uuid,
    booking_use_cases: &ResourceBookingUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let booking = booking_use_cases
        .get_booking(booking_id)
        .await
        .map_err(AppError::from)?;

    verify_building_org_access(user, booking.building_id, building_use_cases, acp_use_cases).await
}

pub async fn verify_unit_org_access(
    user: &AuthenticatedUser,
    unit_id: Uuid,
    unit_use_cases: &UnitUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let unit = unit_use_cases
        .get_unit(unit_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!("Unit not found: {unit_id}")))?;

    let building_id = Uuid::parse_str(&unit.building_id)
        .map_err(|_| AppError::Internal("Invalid unit.building_id format".to_string()))?;

    verify_building_org_access(user, building_id, building_use_cases, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'organisation d'un **copropriétaire**.
///
/// ── Pourquoi cette garde-ci compte plus que les autres ─────────────────────
///
/// Les routes portées par un copropriétaire servent, nominativement, ce qu'une
/// personne doit et ce qu'elle a payé :
///
///     GET /owners/{id}/payments            montants et dates
///     GET /owners/{id}/payments/total      ce qu'elle a versé
///     GET /owners/{id}/payment-methods     ses instruments enregistrés
///     GET /owners/{id}/payment-reminders   ses rappels, donc ses retards
///     GET /owners/{id}/distributions       sa quote-part de chaque charge
///     GET /owners/{id}/total-due           ce qu'elle doit
///
/// Un identifiant de copropriétaire suffisait à les obtenir. Ce n'est pas un
/// écart de périmètre, c'est la situation financière d'une personne nommée
/// servie à qui la demande — au sens du RGPD, une violation de données.
///
/// ── Le chemin est court, et c'est ce qui le rend sûr ───────────────────────
///
/// `Owner` porte directement son `organization_id` (`domain/copropriete/owner.rs:9`).
/// Pas de chaîne à remonter, donc pas de chaîne à recopier de travers — c'est
/// la recopie manuelle d'une chaîne de quatre sauts qui avait laissé fuir les
/// bulletins de vote nominatifs (RN-2, issue #772).
///
/// Sémantique : superadmin passe ; sinon le copropriétaire doit exister et
/// relever de l'organisation de l'appelant. Un copropriétaire introuvable est
/// refusé plutôt qu'ignoré.
pub async fn verify_owner_org_access(
    user: &AuthenticatedUser,
    owner_id: Uuid,
    owner_use_cases: &OwnerUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let owner = owner_use_cases
        .get_owner(owner_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!("Owner not found: {owner_id}")))?;

    let org_id = Uuid::parse_str(&owner.organization_id)
        .map_err(|_| AppError::Internal("Invalid owner.organization_id format".to_string()))?;

    user.verify_org_access(org_id)
        .map_err(|_| AppError::Forbidden("Owner outside your organization".to_string()))
}

/// Vérifie le mandat de l'appelant sur l'ACP dont relève une **assemblée**.
///
/// Remonte AG → immeuble → ACP → organisation. Cette chaîne était recopiée à
/// la main dans chaque gestionnaire qui en avait besoin, et c'est ainsi
/// qu'elle a fini par manquer : `GET /meetings/{id}/resolutions` et
/// `GET /resolutions/{id}/votes` rendaient `200` sur les données d'une autre
/// copropriété, bulletins nominatifs compris (RN-2, issue #772).
///
/// Une AG introuvable est refusée, jamais ignorée : c'est précisément le cas
/// où l'on ne sait pas à qui elle appartient.
pub async fn verify_meeting_org_access(
    user: &AuthenticatedUser,
    meeting_id: Uuid,
    meeting_use_cases: &crate::application::use_cases::MeetingUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let meeting = meeting_use_cases
        .get_meeting(meeting_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!(
            "Meeting not found: {meeting_id}"
        )))?;

    verify_building_org_access(user, meeting.building_id, building_use_cases, acp_use_cases).await
}

/// Vérifie le mandat de l'appelant sur l'ACP dont relève une **dépense**.
///
/// Remonte dépense → immeuble → ACP → organisation. Même motif que
/// `verify_meeting_org_access` : une dépense porte des montants et souvent des
/// pièces jointes nominatives — factures, devis — qui n'ont pas à circuler
/// entre cabinets.
pub async fn verify_expense_org_access(
    user: &AuthenticatedUser,
    expense_id: Uuid,
    expense_use_cases: &crate::application::use_cases::ExpenseUseCases,
    building_use_cases: &BuildingUseCases,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let depense = expense_use_cases
        .get_expense(expense_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound(format!(
            "Expense not found: {expense_id}"
        )))?;

    let building_id = Uuid::parse_str(&depense.building_id)
        .map_err(|_| AppError::Internal("Invalid expense.building_id format".to_string()))?;

    verify_building_org_access(user, building_id, building_use_cases, acp_use_cases).await
}

// ============================================================================
// Actix middleware
// ============================================================================

/// `ScopeGuard` Actix middleware factory.
///
/// Wrap your routes with:
/// ```rust,ignore
/// use actix_web::web;
/// use koprogo_api::infrastructure::web::middleware::ScopeGuard;
///
/// cfg.service(
///     web::scope("/buildings")
///         .wrap(ScopeGuard::new())
///         // ... .service(...)
/// );
/// ```
#[derive(Clone, Default)]
pub struct ScopeGuard;

impl ScopeGuard {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for ScopeGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ScopeGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ScopeGuardMiddleware {
            service: Arc::new(service),
        }))
    }
}

pub struct ScopeGuardMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for ScopeGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        // 1. Extract JWT claims.
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(s) => s.clone(),
            None => {
                let err = ScopeGuardError::Internal("AppState missing".into());
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let token = match auth_header.as_deref() {
            Some(h) if h.starts_with("Bearer ") => {
                h.trim_start_matches("Bearer ").trim().to_string()
            }
            _ => {
                let err = ScopeGuardError::Unauthorized;
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let claims = match app_state.auth_use_cases.verify_token(&token) {
            Ok(c) => c,
            Err(_) => {
                let err = ScopeGuardError::Unauthorized;
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(u) => u,
            Err(_) => {
                let err = ScopeGuardError::Unauthorized;
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let caller = caller_from_role(&claims.role, claims.organization_id, user_id);

        // 2. Extract requested acp_id from header OR query string.
        let header_val = req
            .headers()
            .get(SCOPE_ACP_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        // Parse `?acp_id=` ourselves to avoid double-Deserialize collisions
        // with arbitrary handlers' Query<T> extractors.
        let query_val = req.query_string().split('&').find_map(|kv| {
            let mut it = kv.splitn(2, '=');
            match (it.next(), it.next()) {
                (Some("acp_id"), Some(v)) => Some(v.to_string()),
                _ => None,
            }
        });

        let requested_acp_id =
            match extract_requested_acp_id(header_val.as_deref(), query_val.as_deref()) {
                Ok(v) => v,
                Err(err) => {
                    let resp = req.into_response(err.error_response().map_into_right_body());
                    return Box::pin(async move { Ok(resp) });
                }
            };

        let check = match requires_repository_check(&caller, requested_acp_id) {
            Ok(v) => v,
            Err(err) => {
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        // 3. Optional repository check.
        Box::pin(async move {
            if let Some(acp_id) = check {
                if let Err(app_err) = app_state
                    .acp_use_cases
                    .assert_can_see_acp(&caller, acp_id)
                    .await
                {
                    let guard_err = ScopeGuardError::from(app_err);
                    let resp = req.into_response(guard_err.error_response().map_into_right_body());
                    return Ok(resp);
                }
            }

            // 4. Inject AcpScope into request extensions for handlers.
            req.extensions_mut().insert(AcpScope {
                caller: caller.clone(),
                requested_acp_id,
                allowed: true,
            });

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

// ============================================================================
// Tests — taxonomie 4-cat (CRITICAL.md §3). Pure helpers only ; the
// Transform integration is exercised end-to-end via the BDD harness
// `tests/features/list_buildings_role_based.feature`.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- @happy --------------------------------------------------------------

    #[test]
    fn happy_caller_from_role_admin() {
        let org = Uuid::new_v4();
        let c = caller_from_role("admin", Some(org), Uuid::new_v4());
        assert!(matches!(c, AcpCaller::Admin { organization_id } if organization_id == org));
    }

    #[test]
    fn happy_extract_acp_id_from_header() {
        let id = Uuid::new_v4();
        let s = id.to_string();
        let got = extract_requested_acp_id(Some(&s), None).unwrap();
        assert_eq!(got, Some(id));
    }

    #[test]
    fn happy_extract_acp_id_from_query_when_no_header() {
        let id = Uuid::new_v4();
        let s = id.to_string();
        let got = extract_requested_acp_id(None, Some(&s)).unwrap();
        assert_eq!(got, Some(id));
    }

    // ----- @edge ---------------------------------------------------------------

    #[test]
    fn edge_header_takes_precedence_over_query() {
        let h = Uuid::new_v4();
        let q = Uuid::new_v4();
        let got = extract_requested_acp_id(Some(&h.to_string()), Some(&q.to_string())).unwrap();
        assert_eq!(got, Some(h));
    }

    #[test]
    fn edge_empty_header_treated_as_none() {
        let got = extract_requested_acp_id(Some(""), None).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn edge_super_admin_with_no_scope_is_unrestricted() {
        let r = requires_repository_check(&AcpCaller::SuperAdmin, None).unwrap();
        assert!(r.is_none());
    }

    // ----- @security -----------------------------------------------------------

    #[test]
    fn security_owner_requested_acp_must_be_checked_via_repo() {
        let user = Uuid::new_v4();
        let target = Uuid::new_v4();
        let r =
            requires_repository_check(&AcpCaller::Owner { user_id: user }, Some(target)).unwrap();
        assert_eq!(r, Some(target));
    }

    #[test]
    fn security_caller_from_role_unknown_falls_back_to_owner() {
        let uid = Uuid::new_v4();
        let c = caller_from_role("contractor", Some(Uuid::new_v4()), uid);
        assert!(matches!(c, AcpCaller::Owner { user_id } if user_id == uid));
    }

    // ----- @negative -----------------------------------------------------------

    #[test]
    fn negative_malformed_header_returns_validation_error() {
        let err = extract_requested_acp_id(Some("not-a-uuid"), None).unwrap_err();
        assert!(matches!(err, ScopeGuardError::Validation(_)));
    }

    #[test]
    fn negative_scope_guard_error_kind_strings_are_stable() {
        assert_eq!(ScopeGuardError::Unauthorized.kind(), "unauthorized");
        assert_eq!(
            ScopeGuardError::AcpNotInScope {
                acp_id: Uuid::nil()
            }
            .kind(),
            "acp_not_in_scope"
        );
        assert_eq!(ScopeGuardError::Validation("x".into()).kind(), "validation");
    }

    #[test]
    fn negative_apperror_acp_not_in_scope_maps_to_scopeguard_acp_not_in_scope() {
        let id = Uuid::new_v4();
        let g = ScopeGuardError::from(AppError::AcpNotInScope { acp_id: id });
        match g {
            ScopeGuardError::AcpNotInScope { acp_id } => assert_eq!(acp_id, id),
            other => panic!("expected AcpNotInScope, got {:?}", other),
        }
    }

    #[test]
    fn negative_apperror_unauthorized_maps_to_scopeguard_unauthorized() {
        let g = ScopeGuardError::from(AppError::Unauthorized);
        assert!(matches!(g, ScopeGuardError::Unauthorized));
    }
}
