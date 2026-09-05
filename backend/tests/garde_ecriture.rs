//! Garde-fou des écritures : qui peut écrire dans le dossier d'une ACP.
//!
//! Le recentrage de l'ADR-0045 a corrigé la **lecture** : le périmètre d'un
//! syndic se dérive désormais de son mandat, et un cabinet ne voit plus le
//! dossier d'une ACP qu'on ne lui a pas confiée.
//!
//! La **lecture seulement**. Rien n'empêche encore un cabinet de poster une
//! dépense ou un appel de fonds dans l'immeuble d'un autre : le use-case
//! résout bien l'ACP depuis l'immeuble — ce qui interdit de forger un
//! rattachement — mais personne ne vérifie que l'appelant a un mandat sur
//! cette ACP-là.
//!
//! `verify_building_org_access` fait exactement cette vérification et existe
//! depuis un moment. Le problème n'a jamais été de l'écrire, mais de l'appeler
//! partout où il faut.
//!
//! Ce test ne peut pas garder les routes à la place du développeur. Il fait
//! deux choses plus modestes et vérifiables :
//!
//! 1. il **borne la dette** : le nombre de routes d'écriture non gardées peut
//!    diminuer, jamais augmenter ;
//! 2. il **nomme le reste** : la liste de ce qui manque est dans le message
//!    d'échec, pas dans un document qui se périme.
//!
//! Voir ADR-0045 et le lot J5 du WBS.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Les modules dont les écritures touchent le patrimoine ou le dossier de
/// gestion d'une ACP. Ailleurs (gamification, échanges locaux, préférences),
/// une écriture mal cloisonnée n'engage pas le patrimoine d'une copropriété.
const MODULES_PATRIMOINE: &[&str] = &[
    "acp_handlers",
    "budget_handlers",
    "building_handlers",
    "call_for_funds_handlers",
    "charge_distribution_handlers",
    "convocation_handlers",
    "etat_date_handlers",
    "expense_handlers",
    "journal_entry_handlers",
    "meeting_handlers",
    "owner_contribution_handlers",
    "owner_handlers",
    "payment_reminder_handlers",
    "unit_handlers",
    "unit_owner_handlers",
];

/// Solde constaté le 2026-09-03, au moment où la frontière a été posée.
///
/// Sur 79 routes d'écriture touchant le patrimoine d'une ACP, 10 appelaient
/// une garde. Cinq de plus ont été posées dans la foulée — création
/// d'assemblée, de convocation, de seconde convocation, d'écriture manuelle —
/// parce que leur immeuble était déjà dans la requête. Il en reste 69.
///
/// Ce nombre est un **plafond** : il descend, il ne remonte pas. Il n'est pas
/// une cible acceptable, seulement la mesure honnête de ce qui reste.
///
/// Les 67 restantes se répartissent en deux familles :
///
/// - celles qui **agissent sur une pièce par son identifiant** (approuver un
///   budget, envoyer un appel de fonds, escalader une relance). Elles ne
///   portent pas d'immeuble dans leur requête ; il faut charger la pièce, lire
///   son `acp_id` — que le recentrage vient justement de lui donner — et
///   vérifier le mandat. C'est désormais possible partout, ça ne l'était pas
///   avant ;
/// - celles qui sont **déjà réservées au SuperAdmin** (création de lot, par
///   exemple). La garde y passerait sans rien vérifier, puisqu'un superadmin
///   la traverse par construction. Les compter reste juste : le jour où le
///   rôle s'élargit, la garde manquera vraiment.
///
/// **Ce jour est arrivé le 2026-09-05**, et pour l'exemple même cité
/// ci-dessus. `create_building` et `create_unit` se sont ouvertes au syndic,
/// ce qui a rendu leur garde de périmètre réellement nécessaire ; elles la
/// portent désormais, et la dette tombe de 69 à 67. Les avoir comptées alors
/// qu'elles étaient inoffensives est ce qui a permis de les voir au moment où
/// elles ont cessé de l'être.
const DETTE_AU_2026_09_05: usize = 67;

fn racine_handlers() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers")
}

fn est_garde(corps: &str) -> bool {
    corps.contains("verify_building_org_access") || corps.contains("verify_acp_org_access")
}

/// Les routes d'écriture d'un module, avec le verdict de garde.
fn routes_decriture(source: &str) -> Vec<(String, bool)> {
    let mut trouvees = Vec::new();
    for verbe in ["post", "put", "patch", "delete"] {
        let marqueur = format!("#[{verbe}(");
        let mut reste = source;
        while let Some(pos) = reste.find(&marqueur) {
            reste = &reste[pos + marqueur.len()..];
            let Some(debut_fn) = reste.find("pub async fn ") else {
                break;
            };
            let apres = &reste[debut_fn + "pub async fn ".len()..];
            let nom: String = apres
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            // Le corps s'arrête à la prochaine déclaration de premier niveau,
            // pour ne pas attribuer à une route la garde posée par sa voisine.
            // On cherche un `#[` EN DÉBUT DE LIGNE : couper au premier `#[`
            // venu tronquait le corps sur les attributs internes, et faisait
            // compter comme non gardées des routes qui l'étaient.
            let corps = &apres[..apres.find("\n#[").unwrap_or(apres.len())];
            trouvees.push((nom, est_garde(corps)));
        }
    }
    trouvees
}

fn recenser() -> BTreeMap<String, Vec<(String, bool)>> {
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
        if !MODULES_PATRIMOINE.contains(&module.as_str()) {
            continue;
        }
        let source = fs::read_to_string(&chemin).expect("source lisible");
        par_module.insert(module, routes_decriture(&source));
    }
    par_module
}

#[test]
fn la_dette_de_garde_decriture_ne_grossit_pas() {
    let par_module = recenser();

    let mut non_gardees: Vec<String> = Vec::new();
    for (module, routes) in &par_module {
        for (nom, garde) in routes {
            if !garde {
                non_gardees.push(format!("  {module}::{nom}"));
            }
        }
    }

    assert!(
        non_gardees.len() <= DETTE_AU_2026_09_05,
        "la dette de garde d'écriture a grossi : {} routes non gardées contre {} \
         au moment où la frontière a été posée.\n\n{}\n\n\
         Une route d'écriture neuve sur le patrimoine d'une ACP appelle \
         `verify_building_org_access` ou `verify_acp_org_access`. Sans elle, un \
         cabinet peut écrire dans le dossier d'une copropriété qu'on ne lui a \
         pas confiée (ADR-0045).",
        non_gardees.len(),
        DETTE_AU_2026_09_05,
        non_gardees.join("\n")
    );
}

/// Le recensement doit rester lisible : si plus aucune route n'est détectée,
/// c'est que le motif de reconnaissance a cessé de correspondre au code, pas
/// que la dette a disparu.
#[test]
fn le_recensement_trouve_bien_des_routes() {
    let par_module = recenser();
    let total: usize = par_module.values().map(Vec::len).sum();

    assert!(
        total >= 70,
        "seulement {total} routes d'écriture détectées sur le patrimoine : le motif \
         de reconnaissance ne correspond plus au code, et ce test ne garde donc plus rien."
    );
    assert_eq!(
        par_module.len(),
        MODULES_PATRIMOINE.len(),
        "un module de patrimoine a disparu ou changé de nom : {:?}",
        par_module.keys().collect::<Vec<_>>()
    );
}
