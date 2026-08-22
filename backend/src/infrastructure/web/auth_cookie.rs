//! Refresh-token cookie helpers (WP-FE1 — JWT hors localStorage).
//!
//! Le refresh token ne transite plus dans le corps JSON ni dans
//! `localStorage` (vol de session via XSS). Il est porté par un cookie
//! `HttpOnly; Secure; SameSite=Strict` scopé sur le chemin des endpoints
//! d'authentification — illisible par JavaScript, non rejouable hors du
//! même site. L'access token reste en mémoire JS (header `Bearer`,
//! inchangé). Couche infra uniquement : les use-cases restent purs.

use actix_web::cookie::{time::Duration, Cookie, SameSite};

/// Nom du cookie portant le refresh token.
pub const REFRESH_COOKIE_NAME: &str = "koprogo_refresh";

/// Chemin de scoping : le cookie n'est envoyé qu'aux endpoints
/// `/api/v1/auth/*` (refresh, logout) — surface minimale.
pub const REFRESH_COOKIE_PATH: &str = "/api/v1/auth";

/// Durée de vie alignée sur le `RefreshToken` domaine (7 jours,
/// cf. `domain::entities::refresh_token::RefreshToken::new`).
const REFRESH_COOKIE_MAX_AGE_DAYS: i64 = 7;

/// `Secure` flag : `true` en prod (HTTPS obligatoire), `false` seulement
/// en dev sur http (sinon le navigateur ignore le cookie). Piloté par
/// `COOKIE_SECURE` (défaut sûr = `true`).
fn cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true)
}

/// Cookie posant le refresh token (login / refresh-rotation / register /
/// switch-role). `SameSite=Strict` : front et API servis sur le même site
/// (Traefik domaine unique) — anti-CSRF natif.
pub fn build_refresh_cookie(refresh_token: &str) -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE_NAME, refresh_token.to_owned())
        .http_only(true)
        .secure(cookie_secure())
        .same_site(SameSite::Strict)
        .path(REFRESH_COOKIE_PATH)
        .max_age(Duration::days(REFRESH_COOKIE_MAX_AGE_DAYS))
        .finish()
}

/// Cookie d'expiration immédiate (logout) : même nom/chemin, `Max-Age=0`,
/// valeur vidée. Le navigateur supprime le cookie.
pub fn build_clearing_cookie() -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE_NAME, "")
        .http_only(true)
        .secure(cookie_secure())
        .same_site(SameSite::Strict)
        .path(REFRESH_COOKIE_PATH)
        .max_age(Duration::ZERO)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @happy — le cookie refresh est HttpOnly, SameSite=Strict, scopé auth.
    #[test]
    fn happy_refresh_cookie_has_security_attributes() {
        let c = build_refresh_cookie("tok-abc");
        assert_eq!(c.name(), REFRESH_COOKIE_NAME);
        assert_eq!(c.value(), "tok-abc");
        assert_eq!(c.http_only(), Some(true));
        assert_eq!(c.same_site(), Some(SameSite::Strict));
        assert_eq!(c.path(), Some(REFRESH_COOKIE_PATH));
        assert_eq!(c.max_age(), Some(Duration::days(7)));
    }

    /// @security — par défaut (aucune env) le flag Secure est actif :
    /// jamais de refresh token en clair sur une connexion non chiffrée.
    #[test]
    fn security_cookie_secure_defaults_true() {
        std::env::remove_var("COOKIE_SECURE");
        assert!(cookie_secure());
        assert_eq!(build_refresh_cookie("x").secure(), Some(true));
    }

    /// @edge — opt-out explicite dev uniquement (`COOKIE_SECURE=false`)
    /// pour http://localhost ; toute autre valeur reste sécurisée.
    #[test]
    fn edge_cookie_secure_opt_out_only_explicit_false() {
        std::env::set_var("COOKIE_SECURE", "false");
        assert!(!cookie_secure());
        std::env::set_var("COOKIE_SECURE", "true");
        assert!(cookie_secure());
        std::env::set_var("COOKIE_SECURE", "yes-please");
        assert!(cookie_secure());
        std::env::remove_var("COOKIE_SECURE");
    }

    /// @negative — le cookie de logout invalide la session : valeur vidée,
    /// Max-Age=0 (suppression navigateur), attributs sécurité conservés.
    #[test]
    fn negative_clearing_cookie_expires_immediately() {
        let c = build_clearing_cookie();
        assert_eq!(c.name(), REFRESH_COOKIE_NAME);
        assert_eq!(c.value(), "");
        assert_eq!(c.max_age(), Some(Duration::ZERO));
        assert_eq!(c.http_only(), Some(true));
        assert_eq!(c.path(), Some(REFRESH_COOKIE_PATH));
    }
}
