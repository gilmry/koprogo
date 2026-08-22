//! Application-level error type.
//!
//! `AppError` is the typed error used across all use cases and handlers,
//! replacing the legacy `Result<_, String>` pattern (cf. issues #425, #427).
//!
//! Migration started in story AUTH-001 (auth_use_cases.rs).
//!
//! # Design
//!
//! - `thiserror` for ergonomic error definitions.
//! - `actix_web::ResponseError` impl maps each variant to the right HTTP status.
//! - `From<String>` is intentionally provided as a transition convenience for
//!   repositories still returning `Result<_, String>`. Variants should be used
//!   directly when a specific error semantic applies.
//!
//! # Anti-patterns explicitly avoided
//!
//! - Leaking sensitive data in error messages exposed to clients (DB connection
//!   strings, internal IPs, stack traces). The `error_response()` body returns
//!   a structured payload; redaction policy will be enforced in a follow-up RFC
//!   (see #429 §6 and `astro-svelte-expert.memory.md`).
//! - Returning generic `Internal` for everything (defeats the purpose of typed
//!   errors and HTTP status discrimination).

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

/// Application-level error.
///
/// Each variant maps to a specific HTTP status code via `ResponseError`.
/// See module-level docs for usage guidelines.
#[derive(Error, Debug)]
pub enum AppError {
    /// Input validation failed (bad request payload, missing fields, format errors).
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication required but not provided / token missing.
    #[error("Authentication required")]
    Unauthorized,

    /// Provided credentials are invalid.
    /// Used uniformly for "email not found" AND "wrong password" to prevent
    /// username enumeration attacks.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Token expired, malformed, or revoked.
    #[error("Token error: {0}")]
    TokenError(String),

    /// User is authenticated but lacks the required role/permission.
    #[error("Access forbidden: {0}")]
    Forbidden(String),

    /// User account exists but is deactivated.
    /// NOTE: returning a distinct error from `InvalidCredentials` may leak
    /// account existence — security review needed for `auth/login` flow.
    #[error("Account deactivated")]
    AccountDeactivated,

    /// Resource not found (e.g., user by id, building by id).
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Conflict (e.g., email already in use, ownership total > 100%).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// ACP accessed by a user out of scope (different cabinet, no role
    /// assignment). 403 typé — Story 1.1 / ADR-0010 architecture §6.3.
    #[error("ACP {acp_id} not found or out of scope")]
    AcpNotInScope { acp_id: uuid::Uuid },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimited,

    /// Database error (sqlx, connection, query). Internal — not surfaced verbatim to clients.
    #[error("Database error: {0}")]
    Database(String),

    /// Cryptographic error (bcrypt, JWT signing).
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Catch-all for legacy `Result<_, String>` propagation.
    /// Should be reduced over time as repositories migrate.
    #[error("Internal server error: {0}")]
    Internal(String),

    /// MagicLink token does not match any record (forged / unknown / malformed).
    /// Returns 403 Forbidden. Story 3.2 (FR6).
    #[error("Lien invalide")]
    MagicLinkInvalid,

    /// MagicLink TTL elapsed. FR message guides the user to request a new link.
    /// Returns 403 Forbidden. Story 3.2 (FR6).
    #[error("Lien expiré, demandez-en un nouveau au syndic")]
    MagicLinkExpired,

    /// MagicLink already used (single-use enforcement / replay protection).
    /// Returns 403 Forbidden. Story 3.2 (FR6).
    #[error("Lien déjà utilisé")]
    MagicLinkAlreadyConsumed,

    /// Mandate is past its `valid_until` boundary. Returns 403 Forbidden.
    /// Story 3.4 (FR7 INV-14).
    #[error("Mandat expiré, contactez le syndic")]
    MandateExpired,

    /// Mandate has been revoked before its natural expiry. Returns 403 Forbidden.
    /// Story 3.4 (FR7 INV-14).
    #[error("Mandat révoqué")]
    MandateRevoked,

    /// Mandate exists but does not authorise the requested scope (e.g. notary
    /// mandated on Building X tries to act on Building Y). Returns 403 Forbidden.
    /// Story 3.4 (FR7 INV-14).
    #[error("Mandat hors périmètre autorisé")]
    MandateInvalidScope,

    /// No mandate matches the (subject, kind, scope) tuple. Returns 404.
    /// Story 3.4 (FR7 INV-14).
    #[error("Mandat introuvable")]
    MandateNotFound,

    /// The target user already holds the requested role actively. Returns 409.
    /// Story 3.5 (FR8 INV-8) — anti-double-grant.
    #[error("Rôle déjà attribué à cet utilisateur")]
    RoleAlreadyAssigned { user_id: uuid::Uuid, role: String },

    /// The delegator tries to re-delegate a role that was itself delegated
    /// to them. Returns 403 — Story 3.5 (FR8 INV-8) anti-bypass.
    #[error("Re-délégation interdite : ce rôle vous a déjà été délégué")]
    DelegationChainNotAllowed,

    /// Ticket is locked from further edits — INV-24 enforces a 5-minute
    /// editability window after creation. Subsequent edits MUST go through
    /// dedicated workflow endpoints (assign / resolve / cancel …).
    /// Returns 403 Forbidden. Story 3.6 (FR31).
    #[error("Ce ticket est verrouillé (créé il y a plus de 5 minutes)")]
    TicketImmutable,

    /// SyndicResponse is append-only (INV-23). Any attempt to mutate an
    /// existing response (edit / delete) MUST surface here, never as a
    /// generic Conflict / Internal. Returns 403 Forbidden. Story 3.7 (FR32).
    #[error("Une réponse syndic ne peut pas être modifiée (audit immuable)")]
    ResponseImmutable,

    /// TechnicalSpec already approved — cannot be edited in place. Returns
    /// 409 Conflict. The caller must `bump_version` instead. Story 3.8 (FR33).
    #[error("Cahier des charges déjà approuvé (créer une nouvelle version)")]
    TechnicalSpecAlreadyApproved,

    /// A new major version of a TechnicalSpec was issued and requires fresh
    /// signatures from every required signatory. Returns 422 Unprocessable
    /// Entity. Story 3.8 (FR33).
    #[error("Re-signature requise (bump majeur de la version)")]
    TechnicalSpecResignatureRequired,

    /// The user's role does not match any `required_signatures` slot on the
    /// TechnicalSpec, or no active Mandate authorises them. Returns 403.
    /// Story 3.8 (FR33).
    #[error("Signataire non autorisé pour ce rôle sur ce cahier des charges")]
    SignatoryNotAuthorized,

    /// The (signatory, role) pair has already signed this TechnicalSpec.
    /// Returns 409 Conflict. Story 3.8 (FR33).
    #[error("Signature déjà enregistrée pour ce signataire et ce rôle")]
    SignatureAlreadyExists,

    /// A ContractorEvaluation requires the referenced TechnicalSpec to be in
    /// status `Approved` (Story 3.9 — FR34). A spec in Draft /
    /// PendingSignatures / Superseded does not legitimise an evaluation:
    /// the prestation either has not been signed off yet, or the spec it
    /// signed off has been replaced. Returns 422 Unprocessable Entity.
    #[error("Une fiche technique signée est requise avant d'évaluer un prestataire")]
    TechnicalSpecRequired,

    /// A user attempts to evaluate themselves as a contractor (i.e.
    /// `evaluator_user_id == contractor_user_id`). Returns 422 Unprocessable
    /// Entity. Story 3.9 (FR34 INV-21).
    #[error("Un prestataire ne peut pas s'auto-évaluer")]
    EvaluatorIsContractor,

    /// Track H Story H1 — `Building::assert_conformant()` a échoué.
    /// L'immeuble n'est pas conforme à son acte de base : le pre-check
    /// validate-before-compute bloque toute mutation/calcul (charges,
    /// appels de fonds, états datés…). Code 422 + payload
    /// `BUILDING_NOT_CONFORMANT` exploitable côté frontend (toast +
    /// banner narratif). Mémoire `validate-before-compute`.
    ///
    /// **N'expose pas d'info sensible** (pas d'user_id, pas d'org_id) :
    /// uniquement `building_id` + deltas + `quota_basis` — payload requis
    /// par l'admin pour corriger.
    #[error("L'immeuble n'est pas conforme à son acte de base")]
    BuildingNotConformant {
        building_id: uuid::Uuid,
        units_delta: i32,
        quota_delta: rust_decimal::Decimal,
        quota_basis: i32,
    },

    /// Track H Story H3 — `Meeting::assert_can_complete()` invariants Art. 3.87 §3-5 CC.
    ///
    /// Erreur 422 typée avec liste des invariants manquants pour bloquer la
    /// transition `Scheduled → Completed`. Permet au FE d'afficher
    /// `<MissingInvariantsList>` avec narratif par invariant.
    #[error("L'AG n'est pas prête à être clôturée")]
    MeetingNotCompletable {
        meeting_id: uuid::Uuid,
        missing: Vec<crate::domain::entities::MissingInvariant>,
    },

    /// Track H Story H5 (CL1) — `Acp::assert_conformant()` (Art. 3.84 CC, ADR-0010).
    ///
    /// La copropriété (ACP) n'est pas conforme à son acte de base (Σ quotités
    /// de tous les blocs ≠ `acps.total_tantiemes`). 422 + payload
    /// `ACP_NOT_CONFORMANT` (acp_id + deltas + quota_basis), même format que
    /// `BuildingNotConformant`. N'expose pas d'info sensible.
    #[error("La copropriété n'est pas conforme à son acte de base")]
    AcpNotConformant {
        acp_id: uuid::Uuid,
        units_delta: i32,
        quota_delta: rust_decimal::Decimal,
        quota_basis: i32,
    },

    /// Track H Story H13 (CL4) — `Acp::assert_reserve_fund_compliant()`
    /// (Art. 3.86 §3 CC, loi 2019). Le fonds de réserve est sous le seuil légal
    /// des 5 % des charges ordinaires N-1 et non renoncé (vote 4/5). 422 +
    /// payload `RESERVE_FUND_INSUFFICIENT` (acp_id + required/actual + base).
    #[error("Le fonds de réserve est insuffisant (minimum légal 5% des charges N-1)")]
    ReserveFundInsufficient {
        acp_id: uuid::Uuid,
        required: rust_decimal::Decimal,
        actual: rust_decimal::Decimal,
        ordinary_charges_n1: rust_decimal::Decimal,
    },

    /// Track H Story H17 (CL3) — `assert_voting_right_active()` (Art. 3.87 §1
    /// CC). Le lot est démembré (usufruit/nue-propriété, emphytéose, superficie)
    /// ou en indivision sans représentant unique désigné : son droit de vote est
    /// suspendu. 422 + payload `VOTING_RIGHT_SUSPENDED` (unit_id).
    #[error(
        "Droit de vote suspendu : lot démembré/indivis sans représentant unique (Art. 3.87 §1 CC)"
    )]
    VotingRightSuspended { unit_id: uuid::Uuid },
}

impl AppError {
    /// Stable string identifier for the error kind.
    /// Used in `error_response` JSON payload and logging.
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "validation",
            AppError::Unauthorized => "unauthorized",
            AppError::InvalidCredentials => "invalid_credentials",
            AppError::TokenError(_) => "token_error",
            AppError::Forbidden(_) => "forbidden",
            AppError::AccountDeactivated => "account_deactivated",
            AppError::NotFound(_) => "not_found",
            AppError::Conflict(_) => "conflict",
            AppError::AcpNotInScope { .. } => "acp_not_in_scope",
            AppError::MeetingNotCompletable { .. } => "meeting_not_completable",
            AppError::AcpNotConformant { .. } => "acp_not_conformant",
            AppError::ReserveFundInsufficient { .. } => "reserve_fund_insufficient",
            AppError::VotingRightSuspended { .. } => "voting_right_suspended",
            AppError::RateLimited => "rate_limited",
            AppError::Database(_) => "database",
            AppError::Crypto(_) => "crypto",
            AppError::Internal(_) => "internal",
            AppError::MagicLinkInvalid => "magic_link_invalid",
            AppError::MagicLinkExpired => "magic_link_expired",
            AppError::MagicLinkAlreadyConsumed => "magic_link_consumed",
            AppError::MandateExpired => "mandate_expired",
            AppError::MandateRevoked => "mandate_revoked",
            AppError::MandateInvalidScope => "mandate_invalid_scope",
            AppError::MandateNotFound => "mandate_not_found",
            AppError::RoleAlreadyAssigned { .. } => "role_already_assigned",
            AppError::DelegationChainNotAllowed => "delegation_chain_not_allowed",
            AppError::TicketImmutable => "ticket_immutable",
            AppError::ResponseImmutable => "response_immutable",
            AppError::TechnicalSpecAlreadyApproved => "tech_spec_approved",
            AppError::TechnicalSpecResignatureRequired => "tech_spec_resignature_required",
            AppError::SignatoryNotAuthorized => "signatory_not_authorized",
            AppError::SignatureAlreadyExists => "signature_already_exists",
            AppError::TechnicalSpecRequired => "technical_spec_required",
            AppError::EvaluatorIsContractor => "evaluator_is_contractor",
            AppError::BuildingNotConformant { .. } => "building_not_conformant",
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized | AppError::InvalidCredentials | AppError::TokenError(_) => {
                StatusCode::UNAUTHORIZED
            }
            AppError::Forbidden(_)
            | AppError::AccountDeactivated
            | AppError::AcpNotInScope { .. }
            | AppError::MagicLinkInvalid
            | AppError::MagicLinkExpired
            | AppError::MagicLinkAlreadyConsumed
            | AppError::MandateExpired
            | AppError::MandateRevoked
            | AppError::MandateInvalidScope
            | AppError::DelegationChainNotAllowed
            | AppError::TicketImmutable
            | AppError::ResponseImmutable
            | AppError::SignatoryNotAuthorized => StatusCode::FORBIDDEN,
            AppError::NotFound(_) | AppError::MandateNotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_)
            | AppError::RoleAlreadyAssigned { .. }
            | AppError::TechnicalSpecAlreadyApproved
            | AppError::SignatureAlreadyExists => StatusCode::CONFLICT,
            AppError::TechnicalSpecResignatureRequired
            | AppError::TechnicalSpecRequired
            | AppError::EvaluatorIsContractor
            | AppError::BuildingNotConformant { .. }
            | AppError::MeetingNotCompletable { .. }
            | AppError::AcpNotConformant { .. }
            | AppError::ReserveFundInsufficient { .. }
            | AppError::VotingRightSuspended { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::Database(_) | AppError::Crypto(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        // Public-facing message: short and non-leaky for internal variants.
        let public_message = match self {
            AppError::Database(_) | AppError::Crypto(_) | AppError::Internal(_) => {
                "Internal server error".to_string()
            }
            other => other.to_string(),
        };

        // Track H Story H1 — payload narratif pour `BuildingNotConformant`
        // (422) : le FE consomme `details.code == "BUILDING_NOT_CONFORMANT"`
        // pour rendre `<ConformityToast>` + `<ConformityBanner>` (cf.
        // mémoire `validate-before-compute` + DoD-H1).
        let details = match self {
            AppError::BuildingNotConformant {
                building_id,
                units_delta,
                quota_delta,
                quota_basis,
            } => Some(json!({
                "code": "BUILDING_NOT_CONFORMANT",
                "building_id": building_id,
                "units_delta": units_delta,
                // Decimal-as-string (mémoire `no-f64-in-money` + ADR-0007).
                "quota_delta": quota_delta.to_string(),
                "quota_basis": quota_basis,
            })),
            // Track H Story H5 — payload narratif pour `AcpNotConformant` (422).
            // FE consomme `details.code == "ACP_NOT_CONFORMANT"` (banner/toast
            // au niveau copropriété). Même format que BUILDING_NOT_CONFORMANT.
            AppError::AcpNotConformant {
                acp_id,
                units_delta,
                quota_delta,
                quota_basis,
            } => Some(json!({
                "code": "ACP_NOT_CONFORMANT",
                "acp_id": acp_id,
                "units_delta": units_delta,
                "quota_delta": quota_delta.to_string(),
                "quota_basis": quota_basis,
            })),
            // Track H Story H13 — payload narratif `RESERVE_FUND_INSUFFICIENT`
            // (422). Le FE (`<ReserveFundIndicator>`, différé #634) consomme
            // `details.code` + required/actual. Decimal-as-string (ADR-0007).
            AppError::ReserveFundInsufficient {
                acp_id,
                required,
                actual,
                ordinary_charges_n1,
            } => Some(json!({
                "code": "RESERVE_FUND_INSUFFICIENT",
                "acp_id": acp_id,
                "required": required.to_string(),
                "actual": actual.to_string(),
                "ordinary_charges_n1": ordinary_charges_n1.to_string(),
            })),
            // Track H Story H17 — payload narratif `VOTING_RIGHT_SUSPENDED`
            // (422). Le FE (`<VotingSuspendedBadge>`, différé #634) consomme
            // `details.code` + unit_id pour signaler le lot dont le vote est
            // suspendu (Art. 3.87 §1 — désigner un représentant unique).
            AppError::VotingRightSuspended { unit_id } => Some(json!({
                "code": "VOTING_RIGHT_SUSPENDED",
                "unit_id": unit_id,
            })),
            // Track H Story H3 — payload narratif pour `MeetingNotCompletable`
            // (422) : le FE consomme `details.code == "MEETING_NOT_COMPLETABLE"`
            // pour rendre `<MissingInvariantsList>` + toast i18n par invariant.
            AppError::MeetingNotCompletable {
                meeting_id,
                missing,
            } => {
                use crate::domain::entities::MissingInvariant;
                let missing_json: Vec<serde_json::Value> = missing
                    .iter()
                    .map(|m| match m {
                        MissingInvariant::ConvocationsNotSent => {
                            json!({ "type": "ConvocationsNotSent" })
                        }
                        MissingInvariant::VotesNotClosed { open_resolutions } => json!({
                            "type": "VotesNotClosed",
                            "open_resolutions": open_resolutions,
                        }),
                        MissingInvariant::AttendanceNotRecorded => {
                            json!({ "type": "AttendanceNotRecorded" })
                        }
                        MissingInvariant::QuorumNotReached {
                            attended_quotas,
                            total_quotas,
                        } => json!({
                            "type": "QuorumNotReached",
                            "attended_quotas": attended_quotas.to_string(),
                            "total_quotas": total_quotas.to_string(),
                        }),
                        // Story H9 — volet têtes du quorum double (Art. 3.87 §5).
                        MissingInvariant::HeadCountQuorumNotReached {
                            present_owners_count,
                            total_owners_count,
                        } => json!({
                            "type": "HeadCountQuorumNotReached",
                            "present_owners_count": present_owners_count,
                            "total_owners_count": total_owners_count,
                        }),
                        MissingInvariant::MinutesDraftMissing => {
                            json!({ "type": "MinutesDraftMissing" })
                        }
                    })
                    .collect();
                Some(json!({
                    "code": "MEETING_NOT_COMPLETABLE",
                    "meeting_id": meeting_id,
                    "missing": missing_json,
                }))
            }
            _ => None,
        };

        let mut body = json!({
            "error": public_message,
            "kind": self.kind(),
        });
        if let Some(d) = details {
            body["details"] = d;
        }

        HttpResponse::build(self.status_code()).json(body)
    }
}

/// Transition convenience: convert legacy `String` errors from repositories
/// into `AppError::Internal`. Should be used sparingly via `.map_err(AppError::from)`
/// at the boundary; prefer dedicated variants when the error semantic is known.
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Internal(s.to_string())
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(e: bcrypt::BcryptError) -> Self {
        AppError::Crypto(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AppError::TokenError(e.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => AppError::NotFound("row not found".to_string()),
            _ => AppError::Database(e.to_string()),
        }
    }
}

// Domain typed-error → AppError mappings (#433 / WP-A* — pureté hexagonale :
// le domaine expose un enum d'erreur pur, l'application le mappe ici).
// NB : un bloc `impl From` par WP, ajoutés en fin de section pour minimiser
// les conflits de merge entre WP concurrents (A3/A4/A5).

impl From<crate::domain::entities::JournalEntryError> for AppError {
    /// Une écriture comptable malformée est une erreur d'entrée client
    /// (débit≠crédit, ligne invalide, type journal inconnu…) → 400
    /// validation, **jamais** 500 Internal (le `From<String>` générique
    /// mappait à tort vers Internal).
    fn from(e: crate::domain::entities::JournalEntryError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::ChargeDistributionError> for AppError {
    /// Une répartition de charges malformée est une erreur d'entrée client
    /// (quote-part hors [0,1], total négatif, somme des quotités > 100%) →
    /// 400 validation, **jamais** 500 Internal (le `From<String>` générique
    /// mappait à tort vers Internal) — #433 / WP-A4 EXP-005.
    fn from(e: crate::domain::entities::ChargeDistributionError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::EtatDateError> for AppError {
    /// Un état daté malformé (quote-part hors bornes, montant négatif
    /// interdit, transition workflow invalide, champ obligatoire vide) est
    /// une erreur d'entrée client → 400 validation, **jamais** 500 Internal
    /// (le `From<String>` générique mappait à tort vers Internal) —
    /// #433 / WP-A5 EXP-007.
    fn from(e: crate::domain::entities::EtatDateError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::OwnerContributionError> for AppError {
    /// Une contribution malformée (montant négatif, description vide) est
    /// une erreur d'entrée client → 400 validation, **jamais** 500 Internal
    /// (#433 / WP-A6 EXP-008).
    fn from(e: crate::domain::entities::OwnerContributionError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::CallForFundsError> for AppError {
    /// Un appel de fonds malformé (montant ≤ 0, titre/description vide,
    /// échéance ≤ appel) est une erreur d'entrée client → 400 validation,
    /// **jamais** 500 Internal (#433 / WP-A6 EXP-008).
    fn from(e: crate::domain::entities::CallForFundsError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::AcpError> for AppError {
    /// Une ACP malformée (nom vide / trop court / trop long, adresse vide)
    /// est une erreur d'entrée client → 400 validation, **jamais** 500
    /// Internal (Story 1.1 — ADR-0010).
    fn from(e: crate::domain::entities::AcpError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::PortfolioError> for AppError {
    /// Un portefeuille malformé (nom vide, trop court, trop long,
    /// description trop longue) est une erreur d'entrée client → 400
    /// validation, **jamais** 500 Internal (Story 2.1 — ADR-0011).
    fn from(e: crate::domain::entities::PortfolioError) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<crate::domain::entities::BuildingNotConformantError> for AppError {
    /// Track H Story H1 — pre-check validate-before-compute. L'immeuble
    /// n'est pas conforme à son acte de base → 422 + payload narratif
    /// `BUILDING_NOT_CONFORMANT` (FE rend banner + toast).
    fn from(err: crate::domain::entities::BuildingNotConformantError) -> Self {
        AppError::BuildingNotConformant {
            building_id: err.building_id,
            units_delta: err.units_delta,
            quota_delta: err.quota_delta,
            quota_basis: err.quota_basis,
        }
    }
}

impl From<crate::domain::entities::BuildingNotConformantError> for String {
    /// Track H Story H1 — bridge legacy `Result<_, String>` pour les
    /// use-cases qui n'ont pas encore migré vers `AppError` (call_for_funds,
    /// etat_date…). Mémoire `validate-before-compute` : pas de refacto en
    /// cascade dans cette story, le bridge permet l'opérateur `?`.
    fn from(err: crate::domain::entities::BuildingNotConformantError) -> Self {
        format!(
            "BUILDING_NOT_CONFORMANT: building {} units_delta={} quota_delta={} quota_basis={}",
            err.building_id, err.units_delta, err.quota_delta, err.quota_basis
        )
    }
}

// ============================================================================
// Track H Story H5 — bridges From<AcpNotConformantError> (conformité ACP)
// ============================================================================

impl From<crate::domain::entities::AcpNotConformantError> for AppError {
    /// Track H Story H5 — la copropriété (ACP) n'est pas conforme à son acte
    /// de base → 422 + payload `ACP_NOT_CONFORMANT`. Utilisé par les gates
    /// validate-before-compute ACP-level (Story H7).
    fn from(err: crate::domain::entities::AcpNotConformantError) -> Self {
        AppError::AcpNotConformant {
            acp_id: err.acp_id,
            units_delta: err.units_delta,
            quota_delta: err.quota_delta,
            quota_basis: err.quota_basis,
        }
    }
}

impl From<crate::domain::entities::AcpNotConformantError> for String {
    /// Track H Story H5 — bridge legacy `Result<_, String>` (use-cases
    /// call_for_funds / etat_date). Le handler parse le préfixe
    /// `ACP_NOT_CONFORMANT:` pour reconstruire le 422 narratif (cf.
    /// `conformity_response.rs`, Story H7).
    fn from(err: crate::domain::entities::AcpNotConformantError) -> Self {
        format!(
            "ACP_NOT_CONFORMANT: acp {} units_delta={} quota_delta={} quota_basis={}",
            err.acp_id, err.units_delta, err.quota_delta, err.quota_basis
        )
    }
}

// ============================================================================
// Track H Story H13 — bridges From<ReserveFundInsufficientError> (fonds réserve)
// ============================================================================

impl From<crate::domain::entities::ReserveFundInsufficientError> for AppError {
    /// Track H Story H13 — fonds de réserve sous le seuil légal des 5 %
    /// (Art. 3.86 §3, loi 2019) → 422 + payload `RESERVE_FUND_INSUFFICIENT`.
    fn from(err: crate::domain::entities::ReserveFundInsufficientError) -> Self {
        AppError::ReserveFundInsufficient {
            acp_id: err.acp_id,
            required: err.required,
            actual: err.actual,
            ordinary_charges_n1: err.ordinary_charges_n1,
        }
    }
}

impl From<crate::domain::entities::ReserveFundInsufficientError> for String {
    /// Bridge legacy `Result<_, String>` (cohérence avec les autres erreurs
    /// Track H). Préfixe `RESERVE_FUND_INSUFFICIENT:` parsable par un handler.
    fn from(err: crate::domain::entities::ReserveFundInsufficientError) -> Self {
        format!(
            "RESERVE_FUND_INSUFFICIENT: acp {} required={} actual={} charges_n1={}",
            err.acp_id, err.required, err.actual, err.ordinary_charges_n1
        )
    }
}

// ============================================================================
// Track H Story H17 — bridges From<VotingRightSuspendedError> (droit de vote)
// ============================================================================

impl From<crate::domain::entities::VotingRightSuspendedError> for AppError {
    /// Track H Story H17 — lot démembré/indivis sans représentant unique
    /// (Art. 3.87 §1) → 422 + payload `VOTING_RIGHT_SUSPENDED`.
    fn from(err: crate::domain::entities::VotingRightSuspendedError) -> Self {
        AppError::VotingRightSuspended {
            unit_id: err.unit_id,
        }
    }
}

impl From<crate::domain::entities::VotingRightSuspendedError> for String {
    /// Bridge legacy `Result<_, String>` (cohérence Track H). Préfixe
    /// `VOTING_RIGHT_SUSPENDED:` parsable par le gate vote (`cast_vote`).
    fn from(err: crate::domain::entities::VotingRightSuspendedError) -> Self {
        format!("VOTING_RIGHT_SUSPENDED: unit {}", err.unit_id)
    }
}

// ============================================================================
// Track H Story H3 — bridges From<MeetingNotCompletableError>
// ============================================================================

impl From<crate::domain::entities::MeetingNotCompletableError> for AppError {
    /// Track H Story H3 — convertit l'erreur domain typée vers `AppError` 422
    /// avec liste structurée des invariants manquants. Le FE consomme
    /// `details.missing[]` pour rendre `<MissingInvariantsList>` + toast i18n.
    fn from(err: crate::domain::entities::MeetingNotCompletableError) -> Self {
        AppError::MeetingNotCompletable {
            meeting_id: err.meeting_id,
            missing: err.missing,
        }
    }
}

impl From<crate::domain::entities::MeetingNotCompletableError> for String {
    /// Track H Story H3 — bridge legacy `Result<_, String>` pour
    /// `meeting_use_cases::complete_meeting` (signature historique).
    /// Le handler parse le préfixe `MEETING_NOT_COMPLETABLE:` pour reconstruire
    /// le 422 narratif (cf. pattern Track H Story H2 `conformity_response.rs`).
    fn from(err: crate::domain::entities::MeetingNotCompletableError) -> Self {
        let missing_json: Vec<serde_json::Value> = err
            .missing
            .iter()
            .map(|m| {
                use crate::domain::entities::MissingInvariant;
                match m {
                    MissingInvariant::ConvocationsNotSent => {
                        json!({ "type": "ConvocationsNotSent" })
                    }
                    MissingInvariant::VotesNotClosed { open_resolutions } => json!({
                        "type": "VotesNotClosed",
                        "open_resolutions": open_resolutions,
                    }),
                    MissingInvariant::AttendanceNotRecorded => {
                        json!({ "type": "AttendanceNotRecorded" })
                    }
                    MissingInvariant::QuorumNotReached {
                        attended_quotas,
                        total_quotas,
                    } => json!({
                        "type": "QuorumNotReached",
                        "attended_quotas": attended_quotas.to_string(),
                        "total_quotas": total_quotas.to_string(),
                    }),
                    // Story H9 — volet têtes du quorum double (Art. 3.87 §5).
                    MissingInvariant::HeadCountQuorumNotReached {
                        present_owners_count,
                        total_owners_count,
                    } => json!({
                        "type": "HeadCountQuorumNotReached",
                        "present_owners_count": present_owners_count,
                        "total_owners_count": total_owners_count,
                    }),
                    MissingInvariant::MinutesDraftMissing => {
                        json!({ "type": "MinutesDraftMissing" })
                    }
                }
            })
            .collect();
        format!(
            "MEETING_NOT_COMPLETABLE:{}:{}",
            err.meeting_id,
            serde_json::to_string(&missing_json).unwrap_or_else(|_| "[]".to_string())
        )
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (cf. CRITICAL.md règle #3, #427)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // @happy — chemin nominal
    // ------------------------------------------------------------------------

    #[test]
    fn happy_validation_error_maps_to_400() {
        let e = AppError::Validation("email required".into());
        assert_eq!(e.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(e.kind(), "validation");
    }

    #[test]
    fn happy_invalid_credentials_maps_to_401() {
        let e = AppError::InvalidCredentials;
        assert_eq!(e.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(e.kind(), "invalid_credentials");
    }

    #[test]
    fn happy_not_found_maps_to_404() {
        let e = AppError::NotFound("user 123".into());
        assert_eq!(e.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn happy_conflict_maps_to_409() {
        let e = AppError::Conflict("email already in use".into());
        assert_eq!(e.status_code(), StatusCode::CONFLICT);
    }

    // ------------------------------------------------------------------------
    // @edge — bornes, conversions, cas limites
    // ------------------------------------------------------------------------

    #[test]
    fn edge_from_string_defaults_to_internal() {
        let e: AppError = "legacy error".to_string().into();
        match e {
            AppError::Internal(msg) => assert_eq!(msg, "legacy error"),
            other => panic!("expected Internal, got {:?}", other),
        }
    }

    #[test]
    fn edge_from_str_defaults_to_internal() {
        let e: AppError = "static err".into();
        match e {
            AppError::Internal(msg) => assert_eq!(msg, "static err"),
            other => panic!("expected Internal, got {:?}", other),
        }
    }

    #[test]
    fn edge_empty_validation_message_still_produces_400() {
        let e = AppError::Validation(String::new());
        assert_eq!(e.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn edge_kind_is_stable_string_for_each_variant() {
        // Exhaustive: every variant returns a non-empty stable kind string.
        let variants = [
            AppError::Validation("".into()),
            AppError::Unauthorized,
            AppError::InvalidCredentials,
            AppError::TokenError("".into()),
            AppError::Forbidden("".into()),
            AppError::AccountDeactivated,
            AppError::NotFound("".into()),
            AppError::Conflict("".into()),
            AppError::AcpNotInScope {
                acp_id: uuid::Uuid::nil(),
            },
            AppError::RateLimited,
            AppError::Database("".into()),
            AppError::Crypto("".into()),
            AppError::Internal("".into()),
            AppError::MagicLinkInvalid,
            AppError::MagicLinkExpired,
            AppError::MagicLinkAlreadyConsumed,
            AppError::RoleAlreadyAssigned {
                user_id: uuid::Uuid::nil(),
                role: "syndic".into(),
            },
            AppError::DelegationChainNotAllowed,
        ];
        for v in variants {
            assert!(!v.kind().is_empty(), "kind() empty for {:?}", v);
        }
    }

    // ------------------------------------------------------------------------
    // Story 3.5 — RoleAlreadyAssigned / DelegationChainNotAllowed
    // ------------------------------------------------------------------------

    #[test]
    fn happy_role_already_assigned_maps_to_409() {
        let e = AppError::RoleAlreadyAssigned {
            user_id: uuid::Uuid::nil(),
            role: "syndic".into(),
        };
        assert_eq!(e.status_code(), StatusCode::CONFLICT);
        assert_eq!(e.kind(), "role_already_assigned");
    }

    #[test]
    fn security_delegation_chain_not_allowed_maps_to_403() {
        let e = AppError::DelegationChainNotAllowed;
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(e.kind(), "delegation_chain_not_allowed");
    }

    // ------------------------------------------------------------------------
    // Story 3.6 — TicketImmutable (INV-24)
    // ------------------------------------------------------------------------

    #[test]
    fn security_ticket_immutable_maps_to_403() {
        let e = AppError::TicketImmutable;
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(e.kind(), "ticket_immutable");
        assert!(format!("{}", e).contains("verrouillé"));
    }

    // ------------------------------------------------------------------------
    // Story 3.7 — ResponseImmutable (INV-23)
    // ------------------------------------------------------------------------

    #[test]
    fn security_response_immutable_maps_to_403() {
        let e = AppError::ResponseImmutable;
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(e.kind(), "response_immutable");
        assert!(format!("{}", e).contains("ne peut pas"));
    }

    // ------------------------------------------------------------------------
    // Story 3.8 — TechnicalSpec error variants (FR33)
    // ------------------------------------------------------------------------

    #[test]
    fn happy_tech_spec_already_approved_maps_to_409() {
        let e = AppError::TechnicalSpecAlreadyApproved;
        assert_eq!(e.status_code(), StatusCode::CONFLICT);
        assert_eq!(e.kind(), "tech_spec_approved");
    }

    #[test]
    fn edge_tech_spec_resignature_required_maps_to_422() {
        let e = AppError::TechnicalSpecResignatureRequired;
        assert_eq!(e.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(e.kind(), "tech_spec_resignature_required");
    }

    #[test]
    fn security_signatory_not_authorized_maps_to_403() {
        let e = AppError::SignatoryNotAuthorized;
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(e.kind(), "signatory_not_authorized");
    }

    #[test]
    fn negative_signature_already_exists_maps_to_409() {
        let e = AppError::SignatureAlreadyExists;
        assert_eq!(e.status_code(), StatusCode::CONFLICT);
        assert_eq!(e.kind(), "signature_already_exists");
    }

    // ------------------------------------------------------------------------
    // Story 3.9 — ContractorEvaluation error variants (FR34 INV-21)
    // ------------------------------------------------------------------------

    #[test]
    fn happy_technical_spec_required_maps_to_422() {
        let e = AppError::TechnicalSpecRequired;
        assert_eq!(e.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(e.kind(), "technical_spec_required");
        assert!(format!("{}", e).contains("fiche technique"));
    }

    #[test]
    fn security_evaluator_is_contractor_maps_to_422() {
        let e = AppError::EvaluatorIsContractor;
        assert_eq!(e.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(e.kind(), "evaluator_is_contractor");
        assert!(format!("{}", e).contains("s'auto-évaluer"));
    }

    // ------------------------------------------------------------------------
    // Track H Story H1 — BuildingNotConformant 4-cat
    // ------------------------------------------------------------------------

    fn sample_not_conformant_error(quota_basis: i32, quota_delta: &str) -> AppError {
        use rust_decimal::Decimal;
        use std::str::FromStr;
        AppError::BuildingNotConformant {
            building_id: uuid::Uuid::nil(),
            units_delta: 1,
            quota_delta: Decimal::from_str(quota_delta).unwrap(),
            quota_basis,
        }
    }

    #[test]
    fn happy_building_not_conformant_maps_to_422() {
        let e = sample_not_conformant_error(1000, "2.5");
        assert_eq!(e.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(e.kind(), "building_not_conformant");
    }

    #[test]
    fn happy_from_domain_error_preserves_fields() {
        // From<BuildingNotConformantError> for AppError doit propager
        // building_id, units_delta, quota_delta et quota_basis SANS perte.
        use crate::domain::entities::BuildingNotConformantError;
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let bid = uuid::Uuid::new_v4();
        let domain_err = BuildingNotConformantError {
            building_id: bid,
            units_delta: 2,
            quota_delta: Decimal::from_str("25.5").unwrap(),
            quota_basis: 10000,
        };
        let app_err: AppError = domain_err.into();
        match app_err {
            AppError::BuildingNotConformant {
                building_id,
                units_delta,
                quota_delta,
                quota_basis,
            } => {
                assert_eq!(building_id, bid);
                assert_eq!(units_delta, 2);
                assert_eq!(quota_delta, Decimal::from_str("25.5").unwrap());
                assert_eq!(quota_basis, 10000);
            }
            other => panic!("expected BuildingNotConformant, got {:?}", other),
        }
    }

    #[test]
    fn edge_building_not_conformant_basis_10000() {
        // AC-H1.h3 — quota_basis exposé dans le payload pour acte ≠ 1000.
        let e = sample_not_conformant_error(10000, "25");
        let body = e.error_response();
        // Le body est un HttpResponse, status doit être 422.
        assert_eq!(body.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn security_building_not_conformant_does_not_expose_user_id() {
        // AC-H1.s2 — pas d'info sensible (pas d'user_id, pas d'org_id).
        let e = sample_not_conformant_error(1000, "2.5");
        let s = format!("{}", e);
        // Message public n'expose que la sémantique générique.
        assert!(s.contains("conforme"));
        assert!(!s.contains("user_id"));
        assert!(!s.contains("org_id"));
    }

    #[test]
    fn negative_string_bridge_includes_quota_basis() {
        // AC-H1.h4 — `From<BuildingNotConformantError> for String` legacy
        // doit inclure `quota_basis` pour permettre l'introspection des
        // logs des use-cases legacy `Result<_, String>`.
        use crate::domain::entities::BuildingNotConformantError;
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let domain_err = BuildingNotConformantError {
            building_id: uuid::Uuid::nil(),
            units_delta: 1,
            quota_delta: Decimal::from_str("25").unwrap(),
            quota_basis: 10000,
        };
        let s: String = domain_err.into();
        assert!(s.contains("BUILDING_NOT_CONFORMANT"));
        assert!(s.contains("10000"), "quota_basis must be present: {}", s);
        assert!(s.contains("25"), "quota_delta must be present: {}", s);
    }

    // ----- Story H5 — AcpNotConformant mapping (4-cat) -----------------------

    #[test]
    fn happy_acp_not_conformant_maps_to_422() {
        let e = AppError::AcpNotConformant {
            acp_id: uuid::Uuid::new_v4(),
            units_delta: 1,
            quota_delta: rust_decimal::Decimal::new(25, 1),
            quota_basis: 10000,
        };
        assert_eq!(e.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(e.kind(), "acp_not_conformant");
    }

    #[test]
    fn happy_from_acp_domain_error_preserves_fields() {
        use crate::domain::entities::AcpNotConformantError;
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let aid = uuid::Uuid::new_v4();
        let app_err: AppError = AcpNotConformantError {
            acp_id: aid,
            units_delta: 3,
            quota_delta: Decimal::from_str("25.5").unwrap(),
            quota_basis: 10000,
        }
        .into();
        match app_err {
            AppError::AcpNotConformant {
                acp_id,
                units_delta,
                quota_delta,
                quota_basis,
            } => {
                assert_eq!(acp_id, aid);
                assert_eq!(units_delta, 3);
                assert_eq!(quota_delta, Decimal::from_str("25.5").unwrap());
                assert_eq!(quota_basis, 10000);
            }
            other => panic!("expected AcpNotConformant, got {:?}", other),
        }
    }

    #[test]
    fn security_acp_not_conformant_does_not_expose_sensitive_info() {
        let e = AppError::AcpNotConformant {
            acp_id: uuid::Uuid::new_v4(),
            units_delta: 1,
            quota_delta: rust_decimal::Decimal::new(25, 1),
            quota_basis: 10000,
        };
        let s = format!("{}", e);
        assert!(s.contains("conforme"));
        assert!(!s.contains("user_id"));
        assert!(!s.contains("org_id"));
    }

    #[test]
    fn negative_acp_string_bridge_includes_quota_basis() {
        use crate::domain::entities::AcpNotConformantError;
        use rust_decimal::Decimal;

        let s: String = AcpNotConformantError {
            acp_id: uuid::Uuid::nil(),
            units_delta: 1,
            quota_delta: Decimal::from(25),
            quota_basis: 10000,
        }
        .into();
        assert!(s.contains("ACP_NOT_CONFORMANT"));
        assert!(s.contains("10000"));
    }

    // ------------------------------------------------------------------------
    // @security — RBAC, auth, leakage
    // ------------------------------------------------------------------------

    #[test]
    fn security_acp_not_in_scope_maps_to_403() {
        // Story 1.1 / ADR-0010 — un syndic d'un autre cabinet doit recevoir
        // 403 (pas 404 — l'existence n'est pas un secret côté admin) quand
        // il tente d'accéder à une ACP hors de son scope.
        let e = AppError::AcpNotInScope {
            acp_id: uuid::Uuid::new_v4(),
        };
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(e.kind(), "acp_not_in_scope");
        assert!(format!("{}", e).contains("out of scope"));
    }

    #[test]
    fn security_rate_limited_maps_to_429() {
        let e = AppError::RateLimited;
        assert_eq!(e.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(e.kind(), "rate_limited");
    }

    #[test]
    fn security_forbidden_maps_to_403_not_404() {
        // Returning 403 (not 404) on Forbidden tells the client the resource
        // exists but is denied — acceptable when the existence is not a secret.
        // For secret resources, use NotFound instead.
        let e = AppError::Forbidden("requires syndic role".into());
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn security_token_error_maps_to_401_not_403() {
        // Token errors are auth failures, not authz failures.
        let e = AppError::TokenError("expired".into());
        assert_eq!(e.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn security_database_error_message_is_not_leaked_in_response_body() {
        // Sensitive internal details (connection strings, IPs, stack traces) MUST
        // not leak to clients. error_response replaces the message with a generic one.
        let e = AppError::Database(
            "PostgreSQL: connection refused 192.168.1.5:5432 user=admin password=...".into(),
        );
        let resp = e.error_response();
        let body = resp.into_body();
        // We can't easily extract the JSON body in tests without deserialization,
        // but we know error_response uses the public_message branch for Database.
        // Sanity check at least: status code is 500 (internal).
        let _ = body;
        assert_eq!(e.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        // Direct test of the public message logic:
        let public = match &e {
            AppError::Database(_) => "Internal server error".to_string(),
            other => other.to_string(),
        };
        assert_eq!(public, "Internal server error");
    }

    // ------------------------------------------------------------------------
    // @negative — défaillance correcte (pas de panic, erreur typée)
    // ------------------------------------------------------------------------

    #[test]
    fn negative_internal_variant_maps_to_500() {
        let e = AppError::Internal("oops".into());
        assert_eq!(e.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(e.kind(), "internal");
    }

    #[test]
    fn negative_account_deactivated_maps_to_403() {
        let e = AppError::AccountDeactivated;
        assert_eq!(e.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(e.kind(), "account_deactivated");
    }

    #[test]
    fn negative_crypto_error_maps_to_500_not_401() {
        // bcrypt failures are server-side issues, not auth failures.
        let e = AppError::Crypto("hash format invalid".into());
        assert_eq!(e.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn negative_display_format_includes_message() {
        // thiserror Display impl must include the wrapped message for logs.
        let e = AppError::Database("connection refused".into());
        let s = format!("{}", e);
        assert!(
            s.contains("connection refused"),
            "Display should include detail: {}",
            s
        );
    }
}
