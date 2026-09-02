//! Garde-fou des contextes bornés du domaine.
//!
//! Le domaine de KoproGo répond de trois autorités qui n'ont ni le même
//! rythme de changement, ni le même arbitre :
//!
//! - `copropriete` répond du Code civil, Livre 3 (Art. 3.84 à 3.100) ;
//! - `comptabilite` répond de l'AR du 12/07/2012 (PCMN) et de l'Art. 3.89
//!   § 5, 15° et 16° ;
//! - `economie_circulaire` répond de choix produit ;
//! - `plateforme` répond du RGPD, de la sécurité et du contrat SaaS.
//!
//! Tant que ces quatre univers partageaient une couche plate, leurs règles
//! fuyaient l'une dans l'autre. Le cas qui a motivé cette frontière :
//! `organization`, notion de plateforme, servait de clé d'accès aux pièces
//! comptables, notion légale — si bien que le dossier de gestion
//! appartenait au syndic et non à l'ACP (ADR-0045).
//!
//! Ce test lit les sources et refuse une dépendance interdite. Il ne
//! remplace pas le jugement : il empêche une classe d'erreur de revenir en
//! silence.
//!
//! Voir RFC-0002.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Les dépendances autorisées, contexte par contexte.
///
/// Se lit : « `comptabilite` peut connaître `copropriete` » — une charge se
/// répartit sur des quotités, elle ne peut pas les ignorer. L'inverse est
/// faux : la loi sur la copropriété n'a pas besoin du plan comptable pour
/// dire ce qu'est un lot.
const REGLE: &[(&str, &[&str])] = &[
    ("copropriete", &[]),
    ("comptabilite", &["copropriete"]),
    ("economie_circulaire", &["copropriete"]),
    ("plateforme", &[]),
];

fn racine_domaine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domain")
}

fn sources_rust(dossier: &Path) -> Vec<PathBuf> {
    let mut trouvees = Vec::new();
    let Ok(entrees) = fs::read_dir(dossier) else {
        return trouvees;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            trouvees.extend(sources_rust(&chemin));
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            trouvees.push(chemin);
        }
    }
    trouvees
}

/// Les contextes qu'un fichier référence, hors le sien.
fn contextes_references(source: &str, propre: &str) -> BTreeSet<String> {
    REGLE
        .iter()
        .map(|(nom, _)| *nom)
        .filter(|nom| *nom != propre)
        .filter(|nom| source.contains(&format!("domain::{nom}")))
        .map(str::to_string)
        .collect()
}

#[test]
fn chaque_contexte_du_domaine_existe() {
    let racine = racine_domaine();
    let manquants: Vec<&str> = REGLE
        .iter()
        .map(|(nom, _)| *nom)
        .filter(|nom| !racine.join(nom).is_dir())
        .collect();

    assert!(
        manquants.is_empty(),
        "contextes bornés absents de src/domain/ : {manquants:?}\n\
         Chaque autorité (Code civil, PCMN, produit, RGPD) doit avoir son module."
    );
}

#[test]
fn un_contexte_ne_depend_que_de_ce_que_la_regle_autorise() {
    let racine = racine_domaine();
    let mut infractions = Vec::new();

    for (contexte, autorises) in REGLE {
        for fichier in sources_rust(&racine.join(contexte)) {
            let source = fs::read_to_string(&fichier).expect("source lisible");
            for reference in contextes_references(&source, contexte) {
                if !autorises.contains(&reference.as_str()) {
                    let nom = fichier
                        .strip_prefix(&racine)
                        .unwrap_or(&fichier)
                        .display()
                        .to_string();
                    infractions.push(format!(
                        "  {nom} : `{contexte}` référence `{reference}`, qui ne lui est pas ouvert"
                    ));
                }
            }
        }
    }

    assert!(
        infractions.is_empty(),
        "dépendances interdites entre contextes bornés :\n{}\n\n\
         Un identifiant venu d'un autre contexte reste un Uuid nu ; il ne \n\
         justifie pas d'importer son type. Cf. SyndicMandate.organization_id.",
        infractions.join("\n")
    );
}

/// La couche plate historique doit se vider, pas grossir.
///
/// `domain/entities/` reste une façade de transition qui re-exporte les
/// contextes, le temps que les 703 sites d'appel convergent. Ce test fige
/// le solde : on peut en retirer, jamais en ajouter.
#[test]
fn la_facade_de_transition_ne_grossit_pas() {
    const SOLDE_AU_2026_09_02: usize = 70;

    let restantes = sources_rust(&racine_domaine().join("entities"))
        .into_iter()
        .filter(|f| f.file_name().is_some_and(|n| n != "mod.rs"))
        .count();

    assert!(
        restantes <= SOLDE_AU_2026_09_02,
        "la couche plate a grossi : {restantes} entités contre {SOLDE_AU_2026_09_02} au moment \
         où la frontière a été posée.\nUne entité neuve appartient à un contexte, pas à la façade."
    );
}
