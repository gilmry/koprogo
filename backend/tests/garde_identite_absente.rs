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
///
/// Le chiffre précédent, 19, était FAUX : je l'avais obtenu en soustrayant
/// 5 de 24 plutôt qu'en mesurant. Le détecteur en trouvait 9. Douzième écart
/// de mesure de la journée, et le mien.
///
/// ── L'arbitrage a été rendu le 2026-09-10 (ADR 0048) ────────────────────
///
/// Les six routes qui restaient attendaient une décision, pas du code. Elle
/// est tombée, et elle n'est PAS uniforme :
///
/// - les quatre routes de campagne énergétique sont **publiques** :
///   « l'achat groupé d'énergie sera ouvert à tout le monde » ;
/// - le pixel de suivi d'ouverture est **public** : un client de messagerie
///   ne porte aucun jeton ;
/// - l'état daté par référence ne l'est **pas**. La réponse n'a pas été « on
///   la laisse ouverte » mais « le notaire aura son tableau de bord ». Une
///   référence circule dans des courriels et des dossiers de vente : ce n'est
///   pas un secret, et derrière elle il y a les dettes d'un copropriétaire
///   nommé.
///
/// Reste donc **une** route non gardée, et elle a une issue : l'identité
/// notaire est à créer (#845, ADR 0048).
const DETTE_AU_2026_09_10: usize = 1;

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
    // ── ADR 0048, tranché le 2026-09-10 ──────────────────────────────────
    //
    // « L'achat groupé d'énergie sera ouvert à tout le monde. » Un particulier
    // hors copropriété rejoint une campagne, y consent, y déclare sa
    // consommation et s'en retire — sans compte KoproGo. C'est le sens même
    // d'un achat groupé : plus il y a de monde, meilleur est le prix.
    (
        "/energy-campaigns/{campaign_id}/join-as-individual",
        "achat groupé ouvert aux particuliers hors copropriété (ADR 0048)",
    ),
    (
        "/energy-campaigns/{campaign_id}/members/{member_id}/consent",
        "consentement d'un participant individuel sans compte (ADR 0048)",
    ),
    (
        "/energy-campaigns/{campaign_id}/members/{member_id}/consumption",
        "déclaration de consommation d'un participant individuel (ADR 0048)",
    ),
    (
        "/energy-campaigns/{campaign_id}/members/{member_id}/withdraw",
        "retrait d'un participant individuel : exiger un compte pour partir \
         serait un piège (ADR 0048)",
    ),
    (
        "/convocation-recipients/{id}/email-opened",
        "pixel de suivi d'ouverture : l'appelant est un client de messagerie, \
         qui ne porte aucun jeton (ADR 0048)",
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
        let Ok(source_brute) = fs::read_to_string(&chemin) else {
            continue;
        };
        // Dépouillé AVANT l'extraction : sans cela le relevé compte les
        // déclarations de route COMMENTÉES. `POST /seed/realistic` est dans ce
        // cas — son gestionnaire entier est en commentaire — et il apparaissait
        // comme une route nue alors qu'il n'existe pas.
        let source = sans_commentaires(&source_brute);
        let source = source.as_str();
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

/// Le code, commentaires retirés.
///
/// ── Pourquoi ce dépouillement est indispensable ──────────────────────────
///
/// Sans lui, la garde lit les COMMENTAIRES comme du code. Et les commentaires
/// qui expliquent ce défaut nomment forcément `AuthenticatedUser` :
///
/// ```text
/// // Cette route ne prenait AUCUNE identité : ni `AuthenticatedUser`, ni
/// // jeton lu à la main.
/// ```
///
/// Un gestionnaire portant cette note passait donc pour gardé **du seul fait
/// de documenter qu'il ne l'était pas**. Constaté le 2026-09-08 en vérifiant
/// par témoin : le paramètre retiré de `list_work_reports_paginated`, la garde
/// restait verte.
///
/// Le dépouillement s'applique AVANT l'extraction des routes, et pas seulement
/// avant la vérification. Sinon le relevé compte les déclarations COMMENTÉES :
/// `POST /seed/realistic` est dans ce cas, son gestionnaire entier étant en
/// commentaire, et il ressortait comme une route nue alors qu'il n'existe
/// pas.
///
/// C'est le sixième angle mort de garde de la journée, et le plus retors :
/// les cinq autres ne voyaient pas un défaut, celui-ci se laissait convaincre
/// par sa propre documentation.
fn sans_commentaires(code: &str) -> String {
    let mut sortie = String::with_capacity(code.len());
    let mut dans_bloc = false;
    for ligne in code.lines() {
        let mut reste = ligne;
        if dans_bloc {
            match reste.find("*/") {
                Some(i) => {
                    dans_bloc = false;
                    reste = &reste[i + 2..];
                }
                None => continue,
            }
        }
        if let Some(i) = reste.find("/*") {
            dans_bloc = !reste[i..].contains("*/");
            reste = &reste[..i];
        }
        let sans_ligne = match reste.find("//") {
            Some(i) => &reste[..i],
            None => reste,
        };
        sortie.push_str(sans_ligne);
        sortie.push('\n');
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
        n <= DETTE_AU_2026_09_10,
        "{n} routes ne vérifient AUCUNE identité, contre {DETTE_AU_2026_09_10} \
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
        // Seconde vague : deux routes de paiement et trois du communautaire.
        // Une offre de compétence nomme une personne et décrit ce qu'elle sait
        // faire ; une annonce d'objet prêté nomme son propriétaire et,
        // indirectement, son adresse. Ce sont des données personnelles, pas un
        // annuaire public.
        "GET /payments/stripe/{stripe_payment_intent_id}",
        "GET /payment-methods/stripe/{stripe_payment_method_id}",
        "GET /notices/{id}",
        "GET /skills/{id}",
        "GET /shared-objects/{id}",
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
