//! Cliquet : une route ne peut pas se passer entièrement d'identité.
//!
//! ── Le trou que ce contrôle bouche ────────────────────────────────────────
//!
//! `garde_lecture.rs` compte les routes qui **prennent** `AuthenticatedUser`
//! sans jamais s'en servir. C'était le défaut de #772, et sa remarque est
//! juste : « prendre l'identité sans la vérifier est pire que de ne pas la
//! prendre », parce que la revue croit la route protégée.
//!
//! Mais une route qui ne prend PAS `AuthenticatedUser` du tout lui échappe
//! entièrement. Elle n'apparaît dans aucun des deux cliquets, et rien ne la
//! distingue d'une route délibérément publique.
//!
//! Relevé du 2026-09-08 : **30 routes sur 604** ne vérifiaient aucune
//! identité, hors publics assumés. Parmi elles :
//!
//! ```
//! PUT    /meetings/{id}                  modifier n'importe quelle assemblée
//! DELETE /meetings/{id}                  la supprimer
//! GET    /documents/{id}                 lire n'importe quel document
//! GET    /pcn/export/pdf/{building_id}   exporter une comptabilité entière
//! ```
//!
//! Aucun jeton, aucun en-tête, aucune vérification. Le seul obstacle était de
//! connaître un UUID.
//!
//! ── Ce que ce cliquet vérifie ─────────────────────────────────────────────
//!
//! Une route vérifie son appelant si elle prend `AuthenticatedUser`, ou si
//! elle lit l'en-tête `Authorization` elle-même — plusieurs gestionnaires le
//! font, dont ceux du jeu de données de démonstration, et les compter comme
//! nus serait faux.
//!
//! Les routes délibérément publiques sont listées nommément, **avec leur
//! raison**. Une liste sans justification se remplirait toute seule.
//!
//! Suivi en #845.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Routes sans aucune vérification d'identité. **Ne doit que BAISSER.**
const DETTE_AU_2026_09_08: usize = 24;

/// Les routes publiques, et pourquoi.
///
/// Chaque entrée est un engagement : cette route est servie à un appelant
/// anonyme, et c'est voulu.
const PUBLIQUES: &[(&str, &str)] = &[
    ("/health", "sonde de santé, lue par l'orchestrateur"),
    ("/metrics", "métriques Prometheus, réseau interne"),
    ("/mcp/info", "descripteur du serveur MCP"),
    (
        "/auth/login",
        "on ne peut pas exiger d'être connecté pour se connecter",
    ),
    ("/auth/register", "création de compte"),
    ("/auth/refresh", "le cookie de rafraîchissement fait foi"),
    ("/auth/me", "lit le jeton lui-même"),
    ("/legal/rules", "texte de loi, public par nature"),
    ("/legal/rules/{code}", "texte de loi"),
    ("/legal/ag-sequence", "texte de loi"),
    ("/legal/majority-for/{decision_type}", "texte de loi"),
    (
        "/public/buildings/{slug}/syndic",
        "coordonnées du syndic, que l'Art. 3.89 rend publiques",
    ),
    ("/c/{token}", "lien magique : le jeton EST l'identité"),
    ("/contractor/token/{token}", "lien magique du prestataire"),
    (
        "/contractor/token/{token}/submit",
        "lien magique du prestataire",
    ),
    (
        "/contractor-reports/magic/{token}",
        "lien magique du prestataire",
    ),
    (
        "/contractor-reports/magic/{token}/submit",
        "lien magique du prestataire",
    ),
    ("/marketplace/providers", "annuaire de prestataires, public"),
    (
        "/marketplace/providers/{slug}",
        "annuaire de prestataires, public",
    ),
];

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web")
}

fn fichiers_rust(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            fichiers_rust(&chemin, sortie);
        } else if chemin.extension().and_then(|e| e.to_str()) == Some("rs") {
            sortie.push(chemin);
        }
    }
}

/// Les routes déclarées, avec les paramètres et le corps de leur gestionnaire.
fn routes() -> Vec<(String, String, String, String)> {
    let mut fichiers = Vec::new();
    fichiers_rust(&racine().join("handlers"), &mut fichiers);
    for nom in ["health.rs", "metrics.rs"] {
        let c = racine().join(nom);
        if c.is_file() {
            fichiers.push(c);
        }
    }
    fichiers.sort();

    let mut sortie = Vec::new();
    for chemin in fichiers {
        let Ok(source) = fs::read_to_string(&chemin) else {
            continue;
        };
        let fichier = chemin
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        for (debut_attr, _) in source.match_indices("#[") {
            let reste = &source[debut_attr..];
            let Some(verbe) = ["get", "post", "put", "patch", "delete"]
                .iter()
                .find(|v| reste.starts_with(&format!("#[{v}(\"")))
            else {
                continue;
            };
            let apres = &reste[verbe.len() + 4..];
            let Some(fin_chemin) = apres.find('"') else {
                continue;
            };
            let route = &apres[..fin_chemin];

            // Signature et corps du gestionnaire qui suit.
            let Some(pos_fn) = reste.find("async fn ") else {
                continue;
            };
            let apres_fn = &reste[pos_fn + 9..];
            let Some(fin_nom) = apres_fn.find(['(', ' ', '<']) else {
                continue;
            };
            let nom_fn = &apres_fn[..fin_nom];
            let Some(ouvre) = apres_fn.find('(') else {
                continue;
            };
            let Some(ferme) = apres_fn[ouvre..].find(')') else {
                continue;
            };
            let params = &apres_fn[ouvre..ouvre + ferme];

            // Le corps s'arrête au prochain attribut de route.
            let debut_corps = ouvre + ferme;
            let fin_corps = apres_fn[debut_corps..]
                .find("\n#[")
                .map(|i| debut_corps + i)
                .unwrap_or(apres_fn.len());
            let corps = &apres_fn[debut_corps..fin_corps];

            sortie.push((
                verbe.to_uppercase(),
                route.to_string(),
                format!("{fichier}::{nom_fn}"),
                format!("{params}{corps}"),
            ));
        }
    }
    sortie
}

/// La route vérifie-t-elle qui l'appelle, d'une manière ou d'une autre ?
fn verifie_lidentite(params_et_corps: &str) -> bool {
    params_et_corps.contains("AuthenticatedUser")
        // Plusieurs gestionnaires lisent l'en-tête eux-mêmes plutôt que de
        // passer par l'extracteur. C'est plus verbeux, mais ce n'est pas nu :
        // les compter comme tels serait une mesure fausse, et j'ai failli la
        // publier avant de lire `seed_handlers.rs`.
        || params_et_corps.contains("headers().get(\"Authorization\")")
        || params_et_corps.contains("verify_token")
        || params_et_corps.contains("decode_token")
}

fn nues() -> Vec<String> {
    let publiques: HashSet<&str> = PUBLIQUES.iter().map(|(r, _)| *r).collect();
    let mut sortie: Vec<String> = routes()
        .into_iter()
        .filter(|(_, route, _, pc)| !publiques.contains(route.as_str()) && !verifie_lidentite(pc))
        .map(|(verbe, route, ou, _)| format!("{verbe:6} {route:58} {ou}"))
        .collect();
    sortie.sort();
    sortie
}

#[test]
fn aucune_route_supplementaire_ne_se_passe_didentite() {
    let liste = nues();
    let n = liste.len();

    assert!(
        n <= DETTE_AU_2026_09_08,
        "{n} routes ne vérifient AUCUNE identité, contre {DETTE_AU_2026_09_08} \
         au 2026-09-08.\n\n\
         Ni `AuthenticatedUser`, ni lecture de l'en-tête `Authorization`. Le \
         seul obstacle pour l'appeler est de connaître un UUID.\n\n\
         Ces routes échappent aux deux cliquets de #772, qui ne comptent que \
         celles PRENANT une identité sans s'en servir.\n\n\
         Si la route est délibérément publique, ajoutez-la à `PUBLIQUES` avec \
         sa raison. Sinon, prenez `AuthenticatedUser` et appelez le garde de \
         périmètre qui convient.\n\n{}",
        liste.join("\n")
    );
}

/// Sans quoi une analyse cassée rendrait le cliquet vert faute de trouver quoi
/// que ce soit à mesurer.
#[test]
fn le_releve_voit_encore_les_routes() {
    let total = routes().len();
    assert!(
        total > 500,
        "seulement {total} routes relevées : l'analyse ne lit plus les \
         gestionnaires. Vérifiez avant de vous réjouir."
    );
}

/// Les quatre routes les plus graves du relevé, nommées.
///
/// Un cliquet global peut être satisfait en corrigeant n'importe lesquelles.
/// Celles-ci ne doivent jamais revenir.
#[test]
fn security_les_routes_les_plus_exposees_restent_gardees() {
    let sans_identite: HashSet<String> = routes()
        .into_iter()
        .filter(|(_, _, _, pc)| !verifie_lidentite(pc))
        .map(|(v, r, _, _)| format!("{v} {r}"))
        .collect();

    for route in [
        "PUT /meetings/{id}",
        "DELETE /meetings/{id}",
        "GET /documents/{id}",
        "GET /pcn/export/pdf/{building_id}",
        "GET /pcn/export/excel/{building_id}",
        "POST /pcn/report/{building_id}",
    ] {
        assert!(
            !sans_identite.contains(route),
            "`{route}` a de nouveau perdu toute vérification d'identité.\n\n\
             Elle a été servie à des appelants anonymes : modifier ou \
             supprimer une assemblée générale, lire un document, exporter la \
             comptabilité d'un immeuble. Cf. #845."
        );
    }
}
