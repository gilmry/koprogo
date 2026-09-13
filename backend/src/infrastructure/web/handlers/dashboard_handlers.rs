// Infrastructure Web Handlers: Dashboard
//
// HTTP handlers for dashboard endpoints

use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};

/// Les rôles qui peuvent lire les chiffres financiers d'une organisation.
///
/// ── Ce que la route faisait avant ─────────────────────────────────────────
///
/// Elle prenait `AuthenticatedUser` et ne le consultait QUE pour lire
/// `organization_id`. Tout membre de l'organisation — copropriétaire compris —
/// obtenait donc le total encaissé, le total en attente, et le nombre de
/// copropriétaires en retard de paiement.
///
/// Ce dernier chiffre est le plus sensible : il dit combien de voisins ne
/// paient pas. Un copropriétaire n'a aucun titre à le connaître, et le lui
/// servir est un manquement au cloisonnement, pas une commodité.
///
/// C'est un des 87 cas de #864 — des routes qui PRENNENT une identité sans
/// s'en servir pour décider. Elles sont plus trompeuses que les routes nues :
/// une route sans `AuthenticatedUser` se voit, une garde la compte, personne
/// ne la croit protégée. Celle-ci avait l'air gardée.
const ROLES_FINANCIERS: &[&str] = &[
    "superadmin",
    "admin",
    "syndic",
    "accountant",
    "accountant.encodeur",
    "accountant.emetteur",
];

/// GET /api/v1/dashboard/accountant/stats
/// Get accountant dashboard statistics
#[get("/dashboard/accountant/stats")]
pub async fn get_accountant_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Fail-closed : un rôle inconnu n'obtient rien. Autoriser par défaut
    // ferait de chaque nouveau rôle un lecteur des comptes sans qu'on l'ait
    // décidé.
    if !ROLES_FINANCIERS.contains(&user.role.to_lowercase().as_str()) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Les chiffres financiers de la copropriété sont réservés \
                      au syndic, au comptable et à l'administration."
        }));
    }

    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest().body("User does not belong to an organization");
        }
    };

    match state
        .dashboard_use_cases
        .get_accountant_stats(organization_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// GET /api/v1/dashboard/accountant/transactions?limit=10
/// Get recent transactions for dashboard
#[get("/dashboard/accountant/transactions")]
pub async fn get_recent_transactions(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<RecentTransactionsQuery>,
) -> impl Responder {
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest().body("User does not belong to an organization");
        }
    };

    let limit = query.limit.unwrap_or(10).min(50); // Max 50 transactions

    match state
        .dashboard_use_cases
        .get_recent_transactions(organization_id, limit)
        .await
    {
        Ok(transactions) => HttpResponse::Ok().json(transactions),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(serde::Deserialize)]
pub struct RecentTransactionsQuery {
    pub limit: Option<usize>,
}
