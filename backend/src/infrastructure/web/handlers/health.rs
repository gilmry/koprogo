use actix_web::{get, HttpResponse, Responder};
use serde_json::json;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// L'empreinte de CETTE instance, posée une fois et jamais renouvelée.
///
/// ── Ce qu'elle sert à distinguer ──────────────────────────────────────────
///
/// Une campagne e2e menée contre un backend en rechargement à chaud peut être
/// coupée en deux par une recompilation. Le 2026-09-13, cela a produit
/// **quatre-vingt-quinze spécifications rouges** qu'aucun artefact ne
/// distinguait d'une régression : le rapport HTML montrait 95 échecs, le code
/// de sortie valait 2 dans les deux cas, et il fallait *savoir* qu'il y avait
/// eu une coupure pour aller chercher l'heure dans les journaux du conteneur
/// (#880).
///
/// Un relecteur de promotion concluait à une régression massive. Un agent du
/// fan-out concluait que sa story avait tout cassé.
///
/// Deux valeurs suffisent à trancher : un identifiant de processus et un
/// horodatage. Si l'empreinte relevée à la fin d'une campagne diffère de
/// celle du début, **le serveur a redémarré en cours de route** — et les
/// échecs postérieurs à la coupure ne prouvent rien.
///
/// ── Ce qu'elle ne divulgue pas ────────────────────────────────────────────
///
/// Aucun secret, aucun chemin d'hôte, aucune version : un UUID tiré au
/// démarrage et une date. C'est la contrainte `@security` de #880, et elle
/// est facile à tenir — pour répondre « est-ce le même processus ? », il
/// suffit que la valeur change quand il change.
fn empreinte() -> &'static (String, u64) {
    static EMPREINTE: OnceLock<(String, u64)> = OnceLock::new();
    EMPREINTE.get_or_init(|| {
        let demarre_a = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        (Uuid::new_v4().to_string(), demarre_a)
    })
}

/// Health check endpoint
///
/// Returns system health status. No authentication required.
///
/// Porte aussi l'empreinte de l'instance (`instance_id`, `started_at`), qui
/// permet à un harnais de recette de dire si le serveur a redémarré pendant
/// sa campagne. Voir `empreinte()` ci-dessus et #880.
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "System is healthy", body = serde_json::Value,
            example = json!({
                "status": "ok",
                "service": "koprogo-api",
                "instance_id": "3f2a0c1e-…",
                "started_at": 1789900000_u64
            }))
    )
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    let (instance_id, started_at) = empreinte();
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "koprogo-api",
        "instance_id": instance_id,
        "started_at": started_at,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'empreinte ne change pas d'un appel à l'autre.
    ///
    /// C'est toute sa valeur. Si elle variait au sein d'un même processus, un
    /// harnais conclurait à un redémarrage qui n'a pas eu lieu — et écarterait
    /// des échecs bien réels en les attribuant à une coupure. Le faux négatif
    /// serait pire que le défaut qu'on corrige.
    #[test]
    fn lempreinte_est_stable_dans_un_meme_processus() {
        let (id1, debut1) = empreinte();
        let (id2, debut2) = empreinte();
        assert_eq!(
            id1, id2,
            "l'identifiant d'instance a changé sans redémarrage"
        );
        assert_eq!(debut1, debut2, "l'horodatage de démarrage a changé");
    }

    /// Elle ne porte QUE ce qui répond à « est-ce le même processus ? ».
    #[test]
    fn security_lempreinte_ne_divulgue_ni_secret_ni_chemin() {
        let (id, _) = empreinte();
        assert!(
            Uuid::parse_str(id).is_ok(),
            "l'identifiant d'instance doit être un UUID, et rien d'autre : \
             un format libre finirait par porter un nom d'hôte, une version \
             ou un chemin"
        );
    }
}
