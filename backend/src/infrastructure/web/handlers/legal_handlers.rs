use actix_web::{get, web, HttpResponse, Responder};
use serde_json::{json, Value};

/// Static legal index embedded in binary (Issue #277)
const LEGAL_INDEX: &str = include_str!("../../legal_index.json");

/// GET /api/v1/legal/rules
/// List all legal rules with optional filtering by role or category
///
/// Query parameters:
/// - `role` (optional): Filter by role (syndic, coproprietaire, commissaire, conseil-copropriete, etc.)
/// - `category` (optional): Filter by category (assemblee-generale, travaux, coproprietaire, finance, syndic-mandat)
///
/// Returns:
/// * `200 OK` - Array of legal rules matching filter criteria
/// * `500 Internal Server Error` - JSON parsing error (should not happen with static embed)
///
/// # Example
/// ```
/// GET /api/v1/legal/rules?role=syndic&category=travaux
///
/// Response 200 OK:
/// [
///   {
///     "code": "T01",
///     "category": "travaux",
///     "roles": ["syndic"],
///     "article": "Art. 3.89 §5 2° CC",
///     "title": "Travaux urgents — syndic seul",
///     "content": "Le syndic peut exécuter seul les actes conservatoires...",
///     "keywords": ["urgents", "conservatoires", ...]
///   }
/// ]
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/legal/rules",
    tag = "Legal Reference",
    params(
        ("role" = Option<String>, Query, description = "Filter by role (e.g., syndic, coproprietaire)"),
        ("category" = Option<String>, Query, description = "Filter by category (e.g., assemblee-generale, travaux)")
    ),
    responses(
        (status = 200, description = "Array of legal rules", body = Vec<serde_json::Value>),
        (status = 500, description = "Server error")
    )
)]
#[get("/legal/rules")]
pub async fn list_legal_rules(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    match serde_json::from_str::<Value>(LEGAL_INDEX) {
        Ok(index) => {
            if let Some(rules) = index.get("rules").and_then(|r| r.as_array()) {
                let role_filter = query.get("role").map(|r| r.as_str());
                let category_filter = query.get("category").map(|c| c.as_str());

                let filtered: Vec<&Value> = rules
                    .iter()
                    .filter(|rule| {
                        // Filter by role if provided
                        if let Some(role) = role_filter {
                            if let Some(roles) = rule.get("roles").and_then(|r| r.as_array()) {
                                let matches = roles.iter().any(|r| {
                                    r.as_str().map(|s| s == role).unwrap_or(false)
                                });
                                if !matches {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }

                        // Filter by category if provided
                        if let Some(category) = category_filter {
                            if let Some(cat) = rule.get("category").and_then(|c| c.as_str()) {
                                if cat != category {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }

                        true
                    })
                    .collect();

                HttpResponse::Ok().json(filtered)
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Malformed legal index: missing 'rules' array"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to parse legal index: {}", e)
        })),
    }
}

/// GET /api/v1/legal/rules/:code
/// Get a specific legal rule by its code
///
/// Path parameters:
/// - `code`: Rule code (e.g., AG01, T03, F01)
///
/// Returns:
/// * `200 OK` - The requested legal rule
/// * `404 Not Found` - Rule not found
/// * `500 Internal Server Error` - JSON parsing error
///
/// # Example
/// ```
/// GET /api/v1/legal/rules/AG01
///
/// Response 200 OK:
/// {
///   "code": "AG01",
///   "category": "assemblee-generale",
///   "roles": ["syndic", "coproprietaire", "commissaire", "conseil-copropriete"],
///   "article": "Art. 3.87 §3 CC",
///   "title": "Convocation AG ordinaire — délai minimum",
///   "content": "Le syndic convoque l'assemblée générale ordinaire au moins 15 jours avant...",
///   "keywords": ["convocation", "délai", "15 jours", "ordinaire", "ordre du jour"]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/legal/rules/{code}",
    tag = "Legal Reference",
    params(
        ("code" = String, Path, description = "Legal rule code (e.g., AG01, T03)")
    ),
    responses(
        (status = 200, description = "Legal rule details", body = serde_json::Value),
        (status = 404, description = "Rule not found"),
        (status = 500, description = "Server error")
    )
)]
#[get("/legal/rules/{code}")]
pub async fn get_legal_rule(code: web::Path<String>) -> impl Responder {
    let code = code.into_inner();

    match serde_json::from_str::<Value>(LEGAL_INDEX) {
        Ok(index) => {
            if let Some(rules) = index.get("rules").and_then(|r| r.as_array()) {
                if let Some(rule) = rules.iter().find(|r| {
                    r.get("code")
                        .and_then(|c| c.as_str())
                        .map(|c| c == code)
                        .unwrap_or(false)
                }) {
                    HttpResponse::Ok().json(rule)
                } else {
                    HttpResponse::NotFound().json(json!({
                        "error": format!("Legal rule not found: {}", code)
                    }))
                }
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Malformed legal index: missing 'rules' array"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to parse legal index: {}", e)
        })),
    }
}

/// GET /api/v1/legal/ag-sequence
/// Get the mandatory sequence of general assembly agenda items
///
/// Returns the full sequence of AG agenda items with their order, mandatory status,
/// required majority, and legal notes.
///
/// Returns:
/// * `200 OK` - Array of AG sequence steps
/// * `500 Internal Server Error` - JSON parsing error
///
/// # Example
/// ```
/// GET /api/v1/legal/ag-sequence
///
/// Response 200 OK:
/// [
///   {
///     "step": 1,
///     "point_odj": "Ouverture et constitution du bureau",
///     "mandatory": true,
///     "majority": null,
///     "notes": "Élection du président (copropriétaire), désignation du secrétaire..."
///   },
///   {
///     "step": 2,
///     "point_odj": "Vérification du quorum et émargement",
///     "mandatory": true,
///     "majority": null,
///     "notes": "Signature feuille de présence, calcul quorum (>50% quotes-parts)..."
///   },
///   ...
/// ]
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/legal/ag-sequence",
    tag = "Legal Reference",
    responses(
        (status = 200, description = "AG sequence steps", body = Vec<serde_json::Value>),
        (status = 500, description = "Server error")
    )
)]
#[get("/legal/ag-sequence")]
pub async fn get_ag_sequence() -> impl Responder {
    match serde_json::from_str::<Value>(LEGAL_INDEX) {
        Ok(index) => {
            if let Some(sequence) = index.get("ag_sequence") {
                HttpResponse::Ok().json(sequence)
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Malformed legal index: missing 'ag_sequence'"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to parse legal index: {}", e)
        })),
    }
}

/// GET /api/v1/legal/majority-for/:decision_type
/// Get majority information for a specific decision type
///
/// Path parameters:
/// - `decision_type`: Type of decision (ordinary, qualified_two_thirds, qualified_four_fifths, unanimity, proxy_limit)
///
/// Returns:
/// * `200 OK` - Majority rules and examples
/// * `404 Not Found` - Decision type not found
/// * `500 Internal Server Error` - JSON parsing error
///
/// # Example
/// ```
/// GET /api/v1/legal/majority-for/qualified_two_thirds
///
/// Response 200 OK:
/// {
///   "decision_type": "qualified_two_thirds",
///   "label": "Majorité qualifiée (2/3)",
///   "threshold_description": "Au moins 2/3 des voix",
///   "article": "Art. 3.88 §1 1° CC",
///   "examples": ["Travaux non-conservatoires", "Modification statuts", "Mise en concurrence"],
///   "percentage": 66.67
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/legal/majority-for/{decision_type}",
    tag = "Legal Reference",
    params(
        ("decision_type" = String, Path, description = "Decision type (ordinary, qualified_two_thirds, qualified_four_fifths, unanimity, proxy_limit)")
    ),
    responses(
        (status = 200, description = "Majority rules for decision type", body = serde_json::Value),
        (status = 404, description = "Decision type not found"),
        (status = 500, description = "Server error")
    )
)]
#[get("/legal/majority-for/{decision_type}")]
pub async fn get_majority_for(decision_type: web::Path<String>) -> impl Responder {
    let decision_type = decision_type.into_inner();

    match serde_json::from_str::<Value>(LEGAL_INDEX) {
        Ok(index) => {
            if let Some(majorities) = index.get("majority_types").and_then(|m| m.as_array()) {
                if let Some(majority) = majorities.iter().find(|m| {
                    m.get("decision_type")
                        .and_then(|dt| dt.as_str())
                        .map(|dt| dt == decision_type)
                        .unwrap_or(false)
                }) {
                    HttpResponse::Ok().json(majority)
                } else {
                    HttpResponse::NotFound().json(json!({
                        "error": format!("Decision type not found: {}", decision_type),
                        "valid_types": ["ordinary", "qualified_two_thirds", "qualified_four_fifths", "unanimity", "proxy_limit"]
                    }))
                }
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Malformed legal index: missing 'majority_types'"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to parse legal index: {}", e)
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legal_index_is_valid_json() {
        // Verify the embedded JSON is valid
        let result = serde_json::from_str::<Value>(LEGAL_INDEX);
        assert!(result.is_ok(), "Legal index JSON is malformed");

        let index = result.unwrap();
        assert!(index.get("legal_rules").is_some(), "Missing 'legal_rules' field");
        assert!(index.get("jurisdiction").is_some(), "Missing 'jurisdiction' field");
        assert!(index.get("metadata").is_some(), "Missing 'metadata' field");
        assert!(index.get("version").is_some(), "Missing 'version' field");
    }

    #[test]
    fn test_legal_rules_have_required_fields() {
        let index: Value = serde_json::from_str(LEGAL_INDEX).unwrap();
        let rules = index.get("legal_rules").unwrap().as_array().unwrap();

        assert!(!rules.is_empty(), "legal_rules should not be empty");

        for rule in rules {
            assert!(rule.get("id").is_some(), "Rule missing 'id' field");
            assert!(rule.get("title").is_some(), "Rule missing 'title' field");
            assert!(rule.get("reference").is_some(), "Rule missing 'reference' field");
            assert!(rule.get("summary").is_some(), "Rule missing 'summary' field");
            assert!(rule.get("key_points").is_some(), "Rule missing 'key_points' field");
        }
    }

    #[test]
    fn test_majority_rules_exist() {
        let index: Value = serde_json::from_str(LEGAL_INDEX).unwrap();
        let rules = index.get("legal_rules").unwrap().as_array().unwrap();

        // Verify majority-related rules exist in legal_rules
        let majority_ids = vec!["art_3_88_1", "art_3_88_2", "art_3_88_3"];
        for id in majority_ids {
            let found = rules.iter().any(|r| {
                r.get("id")
                    .and_then(|i| i.as_str())
                    .map(|i| i == id)
                    .unwrap_or(false)
            });
            assert!(found, "Expected majority rule id {} not found", id);
        }
    }

    #[test]
    fn test_ag_related_rules_exist() {
        let index: Value = serde_json::from_str(LEGAL_INDEX).unwrap();
        let rules = index.get("legal_rules").unwrap().as_array().unwrap();

        // Verify AG-related rules exist in legal_rules
        let ag_ids = vec!["art_3_87_3", "art_3_87_5", "bc15_ag_session", "bc17_age_concertation"];
        for id in ag_ids {
            let found = rules.iter().any(|r| {
                r.get("id")
                    .and_then(|i| i.as_str())
                    .map(|i| i == id)
                    .unwrap_or(false)
            });
            assert!(found, "Expected AG rule id {} not found", id);
        }
    }

    #[test]
    fn test_sample_rules_exist() {
        let index: Value = serde_json::from_str(LEGAL_INDEX).unwrap();
        let rules = index.get("legal_rules").unwrap().as_array().unwrap();

        let ids = vec!["art_3_84", "art_3_87_3", "ar_12_07_2012", "gdpr_art_15", "quotas_distribution"];
        for id in ids {
            let found = rules.iter().any(|r| {
                r.get("id")
                    .and_then(|i| i.as_str())
                    .map(|i| i == id)
                    .unwrap_or(false)
            });
            assert!(found, "Expected rule id {} not found", id);
        }
    }
}
