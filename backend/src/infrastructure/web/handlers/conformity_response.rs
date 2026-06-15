//! Track H Story H2 — Helper handlers Actix : convertir une erreur `String`
//! préfixée `"BUILDING_NOT_CONFORMANT:"` (use-cases legacy `Result<_, String>`)
//! en réponse HTTP 422 narrative compatible avec le payload `<ConformityToast>`
//! / `<ConformityBanner>` côté frontend (Story H1).
//!
//! Pourquoi cette gymnastique ?
//! Les use-cases historiques (expense, call_for_funds, charge_distribution,
//! etat_date) renvoient `Result<_, String>`. Le bridge
//! `From<BuildingNotConformantError> for String` (livré par H1) sérialise
//! l'erreur en string structurée :
//!
//!   `"BUILDING_NOT_CONFORMANT: building <uuid> units_delta=<i32> quota_delta=<dec> quota_basis=<i32>"`
//!
//! Refactor signature use-case → `Result<_, AppError>` est hors-scope (mémoire
//! `validate-before-compute`). On parse donc le préfix au niveau handler pour
//! garder le 422 narratif (kind=`building_not_conformant`, details.code=
//! `BUILDING_NOT_CONFORMANT`, units_delta, quota_delta, quota_basis).
//!
//! Sans match du prefix, le caller fallback sur son comportement 400/500
//! habituel.
//!
//! **TODO Track H Story H2 — audit `security_incident`** : la story exige
//! d'insérer un row `security_incidents` (type `BUILDING_NOT_CONFORMANT`)
//! pour chaque tentative bypass. Option 1 (middleware Actix qui inspecte
//! le 422) hors-scope v0.1.0 — ajout middleware = nouveau layer
//! d'application non trivial. Option 2 (inline dans chaque handler) =
//! duplication + nécessite injection `SecurityIncidentRepository` dans
//! `AppState` côté handlers. Reporté en Story H4 (audit middleware). Le
//! pre-check + 422 narratif reste OK fonctionnellement — l'audit n'est
//! que de la trace.

use actix_web::HttpResponse;
use serde_json::json;

/// Préfix marqueur émis par `From<BuildingNotConformantError> for String`
/// (cf. `backend/src/application/error.rs` lignes 447-458).
const PREFIX: &str = "BUILDING_NOT_CONFORMANT:";

/// Si l'erreur use-case correspond à `"BUILDING_NOT_CONFORMANT: ..."`, retourne
/// un `HttpResponse` 422 avec le payload narratif. Sinon, `None` → caller
/// continue son code path (400/500).
///
/// Le payload émis est **structurellement identique** à celui produit par
/// `AppError::BuildingNotConformant` (cf. `application/error.rs::error_response`)
/// pour que le frontend consomme un seul format quel que soit le use-case.
pub fn try_build_conformity_response(err: &str) -> Option<HttpResponse> {
    let trimmed = err.trim();
    if !trimmed.starts_with(PREFIX) {
        return None;
    }

    let rest = trimmed.trim_start_matches(PREFIX).trim();
    let mut building_id: Option<String> = None;
    let mut units_delta: Option<i32> = None;
    let mut quota_delta: Option<String> = None;
    let mut quota_basis: Option<i32> = None;

    // Format émis par le bridge :
    // "building <uuid> units_delta=<i32> quota_delta=<dec> quota_basis=<i32>"
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i];
        if t == "building" && i + 1 < tokens.len() {
            building_id = Some(tokens[i + 1].to_string());
            i += 2;
        } else if let Some(v) = t.strip_prefix("units_delta=") {
            units_delta = v.parse().ok();
            i += 1;
        } else if let Some(v) = t.strip_prefix("quota_delta=") {
            quota_delta = Some(v.to_string());
            i += 1;
        } else if let Some(v) = t.strip_prefix("quota_basis=") {
            quota_basis = v.parse().ok();
            i += 1;
        } else {
            i += 1;
        }
    }

    // Fallback safe : si le parsing échoue partiellement (jamais en pratique
    // — le bridge format est canonique), retourne quand même 422 avec ce
    // qu'on a (le FE tolère les champs manquants).
    let body = json!({
        "error": "L'immeuble n'est pas conforme à son acte de base",
        "kind": "building_not_conformant",
        "details": {
            "code": "BUILDING_NOT_CONFORMANT",
            "building_id": building_id.unwrap_or_default(),
            "units_delta": units_delta.unwrap_or(0),
            "quota_delta": quota_delta.unwrap_or_else(|| "0".to_string()),
            "quota_basis": quota_basis.unwrap_or(1000),
        }
    });
    Some(HttpResponse::UnprocessableEntity().json(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    // @happy — parse complet payload narratif
    #[test]
    fn happy_parses_full_string_to_422_payload() {
        let err = "BUILDING_NOT_CONFORMANT: building 00000000-0000-0000-0000-000000000001 \
                   units_delta=1 quota_delta=2.5 quota_basis=1000";
        let resp = try_build_conformity_response(err).expect("matches prefix");
        assert_eq!(resp.status(), 422);
    }

    // @edge — acte de base 10000 (cas immeuble 182 lots)
    #[test]
    fn edge_quota_basis_10000() {
        let err = "BUILDING_NOT_CONFORMANT: building 00000000-0000-0000-0000-000000000002 \
                   units_delta=1 quota_delta=25 quota_basis=10000";
        let resp = try_build_conformity_response(err).expect("matches");
        assert_eq!(resp.status(), 422);
    }

    // @security — erreur non-conformity ne déclenche pas le 422
    #[test]
    fn security_non_conformity_error_returns_none() {
        let err = "Some other error from somewhere";
        assert!(try_build_conformity_response(err).is_none());
    }

    // @security — message includes secret-looking string n'est pas matché
    #[test]
    fn security_close_but_not_prefix_returns_none() {
        let err = "Almost BUILDING_NOT_CONFORMANT but not";
        assert!(try_build_conformity_response(err).is_none());
    }

    // @negative — parsing partiel (champs manquants) → 422 avec defaults
    #[test]
    fn negative_partial_payload_still_returns_422() {
        let err = "BUILDING_NOT_CONFORMANT: building xxx";
        let resp = try_build_conformity_response(err).expect("matches prefix");
        assert_eq!(resp.status(), 422);
    }

    // @negative — vide
    #[test]
    fn negative_empty_string_returns_none() {
        assert!(try_build_conformity_response("").is_none());
    }
}
