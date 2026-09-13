//! Un contrôle que rien n'appelle ne protège rien.
//!
//! Le motif dominant des six recettes navigateur est **la capacité écrite,
//! testée et inatteignable**. On le connaissait à l'écran : un bouton qui
//! n'appelle rien, une route qu'aucun écran ne sollicite. Il existe aussi
//! dans le domaine, et il y est plus grave, parce que ce qui dort n'est pas
//! une commodité mais une garantie.
//!
//! Le cas qui a fait écrire cette garde :
//! `assert_single_voting_representative` porte l'Art. 3.87 § 1er CC — un seul
//! représentant votant par lot. Une suite BDD entière lui est consacrée
//! (`tests/bdd_voting_right.rs`), dont l'en-tête le nomme. La règle est donc
//! **démontrée**. Et aucun code de production ne l'appelle : elle n'est
//! **appliquée** nulle part. Les tests prouvent qu'elle marche tout en
//! masquant qu'on ne s'en sert pas.
//!
//! Le module de répartition des charges avait déjà nommé le motif sans le
//! borner : `verify_distribution` y était décrit comme le « troisième
//! garde-fou dormant de ce module, après `resolve_owner_quota` et
//! `expense_has_journal_entries` ». Trois dans un seul fichier.
//!
//! L'ampleur, mesurée : **28 contrôles publics dans `src/domain`, dont 14
//! qu'aucun code de production n'appelle**. Une moitié.
//!
//! La garde borne un ENSEMBLE, pas un nombre. Réveiller un contrôle et en
//! endormir un autre laisserait le compte identique et la protection
//! déplacée ; c'est le choix déjà fait par `garde_schema_mort`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Les contrôles endormis relevés le 2026-09-08.
///
/// Retirer un nom d'ici quand un appelant de production existe. En ajouter un
/// demande de justifier, dans le commit, pourquoi une garantie neuve n'est
/// appliquée nulle part.
const DORMANTS_AU_2026_09_08: &[&str] = &[
    "assert_single_voting_representative",
    "can_access_building",
    "can_add_building",
    "can_add_user",
    "can_moderate_community",
    "can_process_data",
    "can_refund",
    "can_send_marketing",
    "peut_convoquer_lui_meme",
    "peut_engager",
    "verifier",
    "verifier_dotation",
    "verifier_exercice",
    "verifier_les_documents",
];

/// Ce qui, dans un nom, annonce un contrôle plutôt qu'un calcul.
const PREFIXES_DE_CONTROLE: &[&str] = &[
    "can_",
    "peut_",
    "assert_",
    "verify_",
    "verifier",
    "valide",
    "est_autorise",
];

/// Le code de production d'un fichier : tout ce qui précède `#[cfg(test)]`.
///
/// Sans cette coupe, un contrôle appelé vingt fois par ses propres tests
/// passerait pour appliqué — ce qui est exactement l'illusion à défaire.
fn production(source: &str) -> &str {
    match source.find("#[cfg(test)]") {
        Some(i) => &source[..i],
        None => source,
    }
}

fn fichiers_rs(racine: &Path, dans: &mut Vec<String>) {
    let Ok(entrees) = fs::read_dir(racine) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            fichiers_rs(&chemin, dans);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            dans.push(chemin.to_string_lossy().into_owned());
        }
    }
}

/// Les contrôles publics déclarés dans `src/domain`.
fn controles_declares() -> BTreeSet<String> {
    let mut fichiers = Vec::new();
    fichiers_rs(Path::new("src/domain"), &mut fichiers);
    let mut noms = BTreeSet::new();
    for chemin in fichiers {
        let source = fs::read_to_string(&chemin).unwrap_or_default();
        for ligne in production(&source).lines() {
            let l = ligne.trim_start();
            let reste = l
                .strip_prefix("pub fn ")
                .or_else(|| l.strip_prefix("pub async fn "));
            let Some(reste) = reste else { continue };
            let nom: String = reste
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if PREFIXES_DE_CONTROLE.iter().any(|p| nom.starts_with(p)) {
                noms.insert(nom);
            }
        }
    }
    noms
}

/// Les contrôles qu'aucun code de production n'appelle.
fn controles_dormants() -> BTreeSet<String> {
    let declares = controles_declares();
    let mut fichiers = Vec::new();
    fichiers_rs(Path::new("src"), &mut fichiers);

    let mut appeles = BTreeSet::new();
    for chemin in fichiers {
        let source = fs::read_to_string(&chemin).unwrap_or_default();
        for ligne in production(&source).lines() {
            let l = ligne.trim_start();
            // On ne saute PAS la ligne entière d'une définition : un corps
            // tenant sur une seule ligne y loge de vrais appels.
            //
            //     pub fn appelant() -> bool { peut_tout_faire(true) }
            //
            // Sauter la ligne rendait cet appel invisible et faisait passer
            // `peut_tout_faire` pour dormant alors qu'il est câblé. C'est un
            // témoin en sens inverse qui l'a montré : la garde accusait à
            // tort. Un détecteur qui accuse use la même chose qu'un détecteur
            // aveugle, la confiance qu'on lui accorde.
            //
            // Seule l'OCCURRENCE précédée de `fn ` est écartée, pas la ligne.
            for nom in &declares {
                if appeles.contains(nom) {
                    continue;
                }
                // `nom(` précédé d'un caractère qui n'appartient pas à un
                // identifiant : sinon `verifier` matcherait dans
                // `verifier_dotation`.
                let motif = format!("{nom}(");
                let mut depuis = 0usize;
                while let Some(i) = l[depuis..].find(&motif) {
                    let abs = depuis + i;
                    let avant = &l[..abs];
                    let avant_est_identifiant = avant
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_');
                    let est_une_definition = avant.trim_end().ends_with("fn");
                    if !avant_est_identifiant && !est_une_definition {
                        appeles.insert(nom.clone());
                        break;
                    }
                    depuis = abs + motif.len();
                }
            }
        }
    }
    declares.difference(&appeles).cloned().collect()
}

#[test]
fn aucun_controle_dormant_nouveau() {
    let dormants = controles_dormants();
    let connus: BTreeSet<String> = DORMANTS_AU_2026_09_08
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let nouveaux: Vec<_> = dormants.difference(&connus).cloned().collect();
    assert!(
        nouveaux.is_empty(),
        "Ces contrôles du domaine ne sont appelés par aucun code de \
         production :\n  {}\n\n\
         Un contrôle que rien n'appelle ne protège rien, même si des tests \
         prouvent qu'il fonctionne. Câblez-le, ou expliquez dans le commit \
         pourquoi une garantie neuve reste inappliquée.",
        nouveaux.join("\n  ")
    );
}

#[test]
fn la_liste_ne_garde_pas_de_controle_deja_reveille() {
    // L'ensemble borné doit rester exact dans les DEUX sens. Un nom qui reste
    // listé après avoir été câblé donne une dette plus lourde qu'elle n'est,
    // et masque le progrès qui justifierait d'en réveiller un autre.
    let dormants = controles_dormants();
    let periodes: Vec<_> = DORMANTS_AU_2026_09_08
        .iter()
        .filter(|n| !dormants.contains(**n))
        .collect();
    assert!(
        periodes.is_empty(),
        "Ces contrôles sont désormais appelés en production : retirez-les de \
         DORMANTS_AU_2026_09_08.\n  {periodes:?}"
    );
}

#[test]
fn le_detecteur_lit_encore_le_domaine() {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre de dormants : une
    // garde qui exige des violations punit sa propre réussite. Il vérifie que
    // le détecteur trouve encore des contrôles ET que la plupart sont câblés,
    // ce qui subsiste quand la dette tombe à zéro.
    let declares = controles_declares();
    assert!(
        declares.len() >= 20,
        "plus assez de contrôles trouvés dans src/domain ({}) : le motif a \
         changé, ou le répertoire a bougé. Vérifiez avant de vous réjouir. \
         Il y en avait 28 au 2026-09-08.",
        declares.len()
    );
    let dormants = controles_dormants();
    assert!(
        dormants.len() < declares.len(),
        "AUCUN contrôle ne serait appelé : le détecteur d'appels ne voit plus \
         rien."
    );
}
