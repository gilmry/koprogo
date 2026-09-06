//! Garde-fou des lectures imbriquées : qui peut lire la sous-collection d'un
//! dossier d'ACP.
//!
//! ── Ce qui a été trouvé ────────────────────────────────────────────────────
//!
//! Recette du 2026-09-06 (RN-2). Un syndic connecté à l'organisation Grand
//! Place obtenait `200` sur les données de l'organisation RECETTE :
//!
//!     GET /resolutions/{id}/votes      → 200, trois bulletins NOMINATIFS
//!     GET /meetings/{id}/resolutions   → 200, intitulés et décomptes d'une AG
//!
//! alors que les mêmes ressources en accès direct rendaient bien `403`. Les
//! deux gestionnaires ne prenaient **même pas `AuthenticatedUser`** en
//! paramètre : pas de garde manquant, pas d'identité du tout.
//!
//! Ce qui fuyait n'était pas anodin. Le vote en assemblée est confidentiel et
//! nominatif : un cabinet concurrent lisait qui avait voté quoi, avec quel
//! poids, dans une copropriété qu'il ne gère pas. C'est une violation de
//! données au sens du RGPD, pas un simple écart de périmètre.
//!
//! ── Pourquoi la fuite s'est logée là ───────────────────────────────────────
//!
//! Le garde de `GET /resolutions/{id}` recopiait **à la main** une chaîne de
//! quatre sauts — résolution → AG → immeuble → ACP → organisation. Recopiée
//! dans chaque gestionnaire, elle finit par manquer là où personne n'y a
//! pensé. Les deux routes fermées le 2026-09-06 l'ont été par une fonction
//! unique, `verifier_mandat_sur_ag`, précisément pour ne plus la recopier.
//!
//! ── Ce que ce test fait, et ne fait pas ────────────────────────────────────
//!
//! Il ne sait pas garder une route à la place du développeur. Comme son
//! voisin `garde_ecriture`, il fait deux choses vérifiables :
//!
//! 1. il **borne la dette** : le nombre de routes imbriquées sans identité
//!    peut diminuer, jamais augmenter ;
//! 2. il **nomme le reste** dans le message d'échec, plutôt que dans un
//!    document qui se périme.
//!
//! ── Attention en lisant ce chiffre ─────────────────────────────────────────
//!
//! Toutes ces routes ne sont pas des fuites. `POST /contractor/token/{token}/
//! submit` s'authentifie par un jeton à usage unique, par conception : elle
//! n'a pas besoin de session. Le compte est donc un **majorant**, à trier.
//! Il ne l'est pas de beaucoup : la grande majorité de la liste est constituée
//! de lectures de données de copropriété — paiements, documents, états datés,
//! convocations, avis.
//!
//! Voir l'issue #772 et le lot R1 du WBS.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Routes imbriquées sans `AuthenticatedUser`.
///
/// **Ce nombre ne doit que DIMINUER.** Le baisser quand on garde une route
/// fait partie de la correction — sans quoi le cliquet rend gratuites autant
/// de régressions qu'il compte d'unités d'écart.
///
/// 73 au relevé du 2026-09-06 ; 70 après les trois listes de documents, puis
/// **33** après avoir gardé les trente-six routes portées par un immeuble —
/// annonces, objets partagés, compétences, inspections, tickets, rapports de
/// travaux, paiements, états datés, convocations.
///
/// Un cliquet posé sans une première baisse n'est qu'une constatation.
///
/// Ordre de traitement retenu, du plus exposé au moins : documents (actes de
/// base, procès-verbaux, factures nominatives), paiements et états datés
/// (montants par personne nommée), convocations, avis.
const DETTE_AU_2026_09_06: usize = 33;

/// Routes imbriquées qui **prennent** l'identité sans jamais la **vérifier**.
///
/// ── Pourquoi ce second compte existe ───────────────────────────────────────
///
/// Le premier compte le paramètre `AuthenticatedUser`. C'est nécessaire, et ce
/// n'est pas suffisant : une route peut le recevoir et l'ignorer.
///
/// Ce n'est pas une hypothèse. En gardant les 36 routes du 2026-09-06, mon
/// propre script a inséré le paramètre dans `get_upcoming_inspections` sans y
/// insérer la garde — la forme de la fonction ne correspondait à aucun des deux
/// motifs reconnus. Le premier cliquet l'aurait comptée **protégée**. Seul
/// l'avertissement `unused variable` du compilateur l'a signalée.
///
/// Un cliquet qui mesure la présence d'un paramètre mesure une intention, pas
/// un effet. Celui-ci mesure l'effet : la route appelle-t-elle quelque chose
/// qui décide du droit d'accès ?
///
/// ── Attention en lisant ce chiffre ─────────────────────────────────────────
///
/// Toutes ces routes ne sont pas des fuites, et le compte est un **majorant**.
/// Certaines s'adressent au superadmin seul, d'autres ne servent que des
/// données de l'appelant lui-même. Le tri reste à faire, route par route, et
/// c'est le travail de l'issue #772.
///
/// **Ce nombre ne doit que DIMINUER.**
const IDENTITE_NON_VERIFIEE_AU_2026_09_06: usize = 109;

fn racine_handlers() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers")
}

/// Les appels qui décident réellement d'un droit d'accès.
///
/// `verify_*_org_access` remonte la chaîne jusqu'à l'organisation ;
/// `verifier_mandat_sur_ag` fait de même depuis une assemblée ;
/// `require_organization` et `is_superadmin` sont des décisions plus grossières
/// mais réelles. Un corps qui n'en contient aucun ne décide de rien.
const MARQUEURS_DE_GARDE: [&str; 8] = [
    "verify_acp_org_access",
    "verify_building_org_access",
    "verify_meeting_org_access",
    "verify_expense_org_access",
    "verifier_mandat",
    "require_organization",
    "is_superadmin",
    "user.organization_id",
];

/// Une route est-elle imbriquée, c'est-à-dire de la forme
/// `/{parent}/{id}/{enfants}` ?
///
/// C'est cette forme qui pose problème : elle atteint une sous-collection par
/// l'identifiant de son parent, sans passer par la route du parent — donc sans
/// le garde que celle-ci porte parfois.
fn est_imbriquee(chemin: &str) -> bool {
    let octets = chemin.as_bytes();
    let mut i = 0;
    while let Some(pos) = chemin[i..].find('}') {
        let apres = i + pos + 1;
        // Un `/` suivi d'une lettre après un paramètre : il y a un segment
        // au-delà de l'identifiant.
        if octets.get(apres) == Some(&b'/')
            && octets
                .get(apres + 1)
                .is_some_and(|c| c.is_ascii_lowercase())
        {
            return true;
        }
        i = apres;
    }
    false
}

/// Ce qu'on sait d'une route imbriquée.
struct Route {
    nom: String,
    /// Prend-elle `AuthenticatedUser` en paramètre ?
    prend_identite: bool,
    /// Appelle-t-elle quelque chose qui décide du droit d'accès ?
    verifie_identite: bool,
}

/// Les routes d'un module, avec les deux verdicts.
fn routes_imbriquees(source: &str) -> Vec<Route> {
    let mut trouvees = Vec::new();
    for verbe in ["get", "post", "put", "patch", "delete"] {
        let marqueur = format!("#[{verbe}(\"");
        let mut reste = source;
        let mut decalage = 0usize;
        while let Some(pos) = reste[decalage..].find(&marqueur) {
            let debut = decalage + pos + marqueur.len();
            let Some(fin) = reste[debut..].find('"') else {
                break;
            };
            let chemin = &reste[debut..debut + fin];
            decalage = debut + fin;

            if !est_imbriquee(chemin) {
                continue;
            }

            // Le corps s'arrête à la prochaine déclaration de premier niveau,
            // pour ne pas attribuer à une route l'identité de sa voisine.
            let suite = &reste[decalage..];
            let corps = &suite[..suite.find("\n#[").unwrap_or(suite.len())];
            trouvees.push(Route {
                nom: format!("{} {}", verbe.to_uppercase(), chemin),
                prend_identite: corps.contains("AuthenticatedUser"),
                verifie_identite: MARQUEURS_DE_GARDE.iter().any(|m| corps.contains(m)),
            });
        }
        let _ = reste;
        reste = source;
        let _ = reste;
    }
    trouvees
}

fn recenser() -> BTreeMap<String, Vec<Route>> {
    let mut par_module = BTreeMap::new();
    let Ok(entrees) = fs::read_dir(racine_handlers()) else {
        return par_module;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let module = chemin.file_stem().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(&chemin).expect("source lisible");
        let routes = routes_imbriquees(&source);
        if !routes.is_empty() {
            par_module.insert(module, routes);
        }
    }
    par_module
}

#[test]
fn la_dette_de_lecture_imbriquee_ne_grossit_pas() {
    let par_module = recenser();

    let mut sans_identite: Vec<String> = Vec::new();
    for (module, routes) in &par_module {
        for route in routes {
            if !route.prend_identite {
                sans_identite.push(format!("  {module}  {}", route.nom));
            }
        }
    }

    assert!(
        sans_identite.len() <= DETTE_AU_2026_09_06,
        "La dette de lecture imbriquée a GROSSI : {} routes sans \
         `AuthenticatedUser`, contre {} au 2026-09-06.\n\n\
         Une route imbriquée sans identité atteint la sous-collection d'un \
         dossier d'ACP sans que personne ne vérifie le mandat de l'appelant. \
         C'est ainsi qu'un cabinet a lu les bulletins nominatifs d'une autre \
         copropriété (RN-2, issue #772).\n\n\
         Prenez `AuthenticatedUser` en paramètre et remontez au mandat par la \
         chaîne du parent, comme `verifier_mandat_sur_ag` le fait dans \
         `resolution_handlers`.\n\n\
         Liste :\n{}",
        sans_identite.len(),
        DETTE_AU_2026_09_06,
        sans_identite.join("\n")
    );
}

/// Le recensement trouve-t-il encore quelque chose ?
///
/// Sans ce contrôle, une refonte du répertoire des gestionnaires rendrait le
/// cliquet silencieusement vert en ne trouvant plus rien à compter — le même
/// piège que celui déjà posé dans `garde_ecriture` et `garde_champs_ignores`.
#[test]
fn le_recensement_trouve_bien_des_routes_imbriquees() {
    let par_module = recenser();
    let total: usize = par_module.values().map(|v| v.len()).sum();
    assert!(
        total > 200,
        "le recensement ne reconnaît plus les routes imbriquées : {total} trouvée(s)"
    );
    assert!(
        par_module.contains_key("resolution_handlers"),
        "les modules attendus ont changé de nom : {:?}",
        par_module.keys().collect::<Vec<_>>()
    );
}

#[test]
fn la_dette_didentite_non_verifiee_ne_grossit_pas() {
    let par_module = recenser();

    let mut non_verifiees: Vec<String> = Vec::new();
    for (module, routes) in &par_module {
        for route in routes {
            if route.prend_identite && !route.verifie_identite {
                non_verifiees.push(format!("  {module}  {}", route.nom));
            }
        }
    }

    assert!(
        non_verifiees.len() <= IDENTITE_NON_VERIFIEE_AU_2026_09_06,
        "La dette d'identité non vérifiée a GROSSI : {} routes prennent \
         `AuthenticatedUser` sans jamais s'en servir, contre {} au \
         2026-09-06.\n\n\
         Prendre l'identité sans la vérifier est pire que de ne pas la prendre : \
         le premier cliquet compte la route comme protégée, et la revue passe. \
         C'est exactement ce qui est arrivé à `get_upcoming_inspections`, dont \
         seul l'avertissement `unused variable` a trahi l'absence de garde.\n\n\
         Appelez l'un des gardes de `scope_guard`, ou dites explicitement \
         pourquoi la route n'en a pas besoin.\n\n\
         Liste :\n{}",
        non_verifiees.len(),
        IDENTITE_NON_VERIFIEE_AU_2026_09_06,
        non_verifiees.join("\n")
    );
}
