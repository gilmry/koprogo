//! `community_access_guard` middleware — Story 5.5 (refonte UX multi-rôle ACP).
//!
//! Contrepartie serveur de l'ADR 0052 : le comptable (`accountant`,
//! `accountant.encodeur`, `accountant.emetteur`) est un prestataire, pas un
//! copropriétaire — il n'a pas accès aux routes communautaires (SEL,
//! sondages, annonces, objets partagés, réservations de ressources
//! communes), sauf s'il cumule aussi un rôle `owner` sur l'organisation
//! (auquel cas il y accède via CE rôle-là, pas via sa casquette comptable).
//!
//! FR/INV : FR28, FR30 ; INV-6. Deps : Story 5.3 (#587). Refs : #556.
//!
//! Design :
//! - Il n'existe pas de scope Actix `/community` dédié — les routes SEL /
//!   sondages / annonces / objets partagés / réservations vivent à plat sous
//!   `/api/v1` (cf. `routes.rs`). Comme `GdprRateLimit`, ce garde est donc
//!   câblé GLOBALEMENT dans `main.rs` et s'auto-filtre par préfixe de
//!   chemin plutôt que par un `.wrap()` scopé sur un sous-arbre de routes.
//! - JWT absent/invalide : laisse filer la requête — l'extracteur
//!   `AuthenticatedUser` du handler renverra son propre 401 ; ce garde n'a
//!   pas à dupliquer cette vérification.
//! - Erreur de lecture des rôles cumulés (DB indisponible) : refuse plutôt
//!   qu'ignorer — même convention que les gardes `verify_*_org_access` de
//!   `scope_guard.rs` (« refusé plutôt qu'ignoré »).

use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;

use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpResponse,
};
use serde_json::json;
use uuid::Uuid;

use crate::domain::entities::{UserRole, UserRoleAssignment};
use crate::infrastructure::web::app_state::AppState;

// ============================================================================
// Pure helpers (testés sans machinerie actix — cf. convention scope_guard.rs).
// ============================================================================

/// Marqueurs de chemin identifiant une route "communautaire" (SEL, sondages,
/// annonces, objets partagés, réservations de ressources communes).
///
/// Pas de scope Actix `/community` : ces routes vivent à plat sous
/// `/api/v1/{exchanges,polls,notices,shared-objects,resource-bookings}` et
/// leurs variantes `/buildings/{id}/...` ou `/owners/{id}/...` (cf.
/// `routes.rs`). Liste explicite plutôt que motif générique : mieux vaut
/// oublier une route (visible en revue, corrigible) que sur-filtrer une
/// route non communautaire par coïncidence de nom.
const COMMUNITY_PATH_MARKERS: &[&str] = &[
    "/exchanges",
    "/polls",
    "/notices",
    "/shared-objects",
    "/resource-bookings",
    "/sel-statistics",
    "/leaderboard",
    "/credit-balance",
    "/exchange-summary",
];

/// `true` si le chemin correspond à une route communautaire protégée.
pub fn is_community_path(path: &str) -> bool {
    COMMUNITY_PATH_MARKERS
        .iter()
        .any(|marker| path.contains(marker))
}

/// Décide si l'accès communautaire doit être refusé pour ce jeu
/// d'assignments de rôles (les assignments expirées — Story 3.5 délégations
/// — sont ignorées).
///
/// Règle (FR28/FR30, INV-6) : refusé si et seulement si l'appelant cumule au
/// moins une casquette comptable ET aucun rôle `owner`. Les DEUX sous-rôles
/// comptables sont exclus, encodeur comme émetteur — n'en exclure qu'un
/// laisserait la porte ouverte par l'autre.
pub fn community_access_denied(assignments: &[UserRoleAssignment]) -> bool {
    let active: Vec<&UserRoleAssignment> = assignments
        .iter()
        .filter(|a| a.is_currently_active())
        .collect();

    let has_owner = active.iter().any(|a| a.role == UserRole::Owner);
    if has_owner {
        return false;
    }

    active.iter().any(|a| a.role.is_accountant())
}

fn forbidden_response() -> HttpResponse {
    HttpResponse::Forbidden().json(json!({
        "error": "Accès communauté réservé aux copropriétaires — le comptable n'y a pas accès (ADR 0052, INV-6)",
        "kind": "community_access_forbidden",
    }))
}

// ============================================================================
// Actix middleware
// ============================================================================

/// `CommunityAccessGuard` — factory Actix, à câbler GLOBALEMENT (cf. design
/// ci-dessus). Wrap :
/// ```rust,ignore
/// use actix_web::App;
/// use koprogo_api::infrastructure::web::middleware::CommunityAccessGuard;
///
/// App::new().wrap(CommunityAccessGuard::new());
/// ```
#[derive(Clone, Default)]
pub struct CommunityAccessGuard;

impl CommunityAccessGuard {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for CommunityAccessGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CommunityAccessGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CommunityAccessGuardMiddleware {
            service: Arc::new(service),
        }))
    }
}

pub struct CommunityAccessGuardMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for CommunityAccessGuardMiddleware<S>
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
        let path = req.path().to_string();

        if !is_community_path(&path) {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(s) => s.clone(),
            None => {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
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
                // Pas de JWT exploitable : laisse le handler / l'extracteur
                // AuthenticatedUser renvoyer son propre 401.
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        };

        let claims = match app_state.auth_use_cases.verify_token(&token) {
            Ok(c) => c,
            Err(_) => {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        };

        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(u) => u,
            Err(_) => {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        };

        let service = self.service.clone();

        Box::pin(async move {
            let assignments = match app_state
                .user_use_cases
                .list_assignments_for_user(user_id)
                .await
            {
                Ok(list) => list,
                Err(_) => {
                    // Refusé plutôt qu'ignoré — même convention que les
                    // gardes verify_*_org_access de scope_guard.rs.
                    let resp = req.into_response(forbidden_response().map_into_right_body());
                    return Ok(resp);
                }
            };

            if community_access_denied(&assignments) {
                let resp = req.into_response(forbidden_response().map_into_right_body());
                return Ok(resp);
            }

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

// ============================================================================
// Tests — taxonomie 4-cat (CRITICAL.md §3). Pure helpers only ; l'intégration
// Transform (parsing JWT + lookup DB) est exercée par le harnais BDD
// `tests/features/community_accountant_forbidden.feature`.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn assignment(role: UserRole) -> UserRoleAssignment {
        UserRoleAssignment::new(Uuid::new_v4(), role, None, true)
    }

    // ----- @happy ----------------------------------------------------------

    #[test]
    fn happy_owner_only_is_allowed() {
        let assignments = vec![assignment(UserRole::Owner)];
        assert!(!community_access_denied(&assignments));
    }

    #[test]
    fn happy_board_member_owner_is_allowed() {
        // Catherine, conseillère : accède via son rôle owner sous-jacent.
        let user_id = Uuid::new_v4();
        let assignments = vec![
            UserRoleAssignment::new(user_id, UserRole::Owner, None, true),
            UserRoleAssignment::new(user_id, UserRole::BoardMember, None, false),
        ];
        assert!(!community_access_denied(&assignments));
    }

    #[test]
    fn happy_community_path_matches_known_routes() {
        assert!(is_community_path("/api/v1/exchanges"));
        assert!(is_community_path("/api/v1/polls"));
        assert!(is_community_path("/api/v1/buildings/123/notices"));
        assert!(is_community_path("/api/v1/shared-objects/my-borrowed"));
        assert!(is_community_path("/api/v1/resource-bookings"));
    }

    // ----- @edge -------------------------------------------------------------

    #[test]
    fn edge_accountant_encodeur_cumulating_owner_is_allowed() {
        // Paul, comptable encodeur, cumule le rôle owner → accède via owner.
        let user_id = Uuid::new_v4();
        let assignments = vec![
            UserRoleAssignment::new(user_id, UserRole::AccountantEncodeur, None, true),
            UserRoleAssignment::new(user_id, UserRole::Owner, None, false),
        ];
        assert!(!community_access_denied(&assignments));
    }

    #[test]
    fn edge_expired_owner_delegation_does_not_grant_access() {
        // Une délégation owner expirée (Story 3.5) ne compte pas comme cumul :
        // le comptable pur redevient refusé dès que la fenêtre est passée.
        let user_id = Uuid::new_v4();
        let expired_owner = UserRoleAssignment::new_delegated(
            user_id,
            UserRole::Owner,
            None,
            Utc::now() - Duration::seconds(1),
            Uuid::new_v4(),
        );
        let assignments = vec![
            UserRoleAssignment::new(user_id, UserRole::AccountantEmetteur, None, true),
            expired_owner,
        ];
        assert!(community_access_denied(&assignments));
    }

    // ----- @security -----------------------------------------------------------

    #[test]
    fn security_pure_accountant_emetteur_is_denied() {
        // Pierre, comptable émetteur pur sans rôle owner → 403 (INV-6).
        let assignments = vec![assignment(UserRole::AccountantEmetteur)];
        assert!(community_access_denied(&assignments));
    }

    #[test]
    fn security_pure_accountant_encodeur_is_denied() {
        // Symétrique : l'encodeur pur est exclu aussi (les DEUX sous-rôles).
        let assignments = vec![assignment(UserRole::AccountantEncodeur)];
        assert!(community_access_denied(&assignments));
    }

    #[test]
    fn security_generic_accountant_without_owner_is_denied() {
        let assignments = vec![assignment(UserRole::Accountant)];
        assert!(community_access_denied(&assignments));
    }

    // ----- @negative -----------------------------------------------------------

    #[test]
    fn negative_no_assignments_is_not_denied_by_this_guard() {
        // Hors périmètre de ce garde : un appelant sans aucune assignment
        // n'est pas un comptable, donc pas concerné par l'exclusion INV-6.
        // D'autres gardes couvrent l'authentification/l'autorisation générale.
        assert!(!community_access_denied(&[]));
    }

    #[test]
    fn negative_non_community_path_is_not_matched() {
        assert!(!is_community_path("/api/v1/buildings/123/expenses"));
        assert!(!is_community_path("/api/v1/units"));
    }
}
