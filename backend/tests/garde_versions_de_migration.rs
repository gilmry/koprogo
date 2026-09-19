//! Cliquet — deux migrations ne peuvent pas porter la même version.
//!
//! Né d'un vrai dégât (#939) : `20260916000000_board_alerts.sql` et
//! `20260916000000_create_funds_and_link_call_for_funds.sql`, produits par
//! deux passes d'agent différentes, ont choisi le même horodatage.
//!
//! sqlx indexe `_sqlx_migrations` sur la **version**, c'est-à-dire le nombre
//! avant le premier `_`, et pas sur le nom du fichier. Deux fichiers au même
//! horodatage font donc échouer `migrate!` sur une base neuve avec un
//! `23505 duplicate key`. Conséquence : toute la suite d'intégration, toute
//! la BDD, et **tout déploiement partant d'une base vide**.
//!
//! Rien ne l'avait vu. Le compilateur ne lit pas les noms de fichiers, et les
//! gates d'intégration n'avaient pas retourné depuis la fusion. Ce test est
//! l'instrument qui manquait.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn dossier_migrations() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

/// (version, nom complet) pour chaque migration « montante ».
///
/// Les `.down.sql` sont écartés : ils accompagnent leur montante et portent
/// la même version par construction.
fn migrations_montantes() -> Vec<(String, String)> {
    let mut trouvees = Vec::new();
    for entree in fs::read_dir(dossier_migrations()).expect("dossier migrations lisible") {
        let nom = entree.expect("entrée lisible").file_name();
        let nom = nom.to_string_lossy().to_string();
        if !nom.ends_with(".sql") || nom.ends_with(".down.sql") {
            continue;
        }
        let version = match nom.split_once('_') {
            Some((v, _)) => v.to_string(),
            // Un fichier sans `_` n'a pas de version lisible : c'est une
            // anomalie, et la signaler vaut mieux que l'ignorer.
            None => panic!("Migration sans version lisible : {nom}"),
        };
        assert!(
            version.chars().all(|c| c.is_ascii_digit()),
            "La version de {nom} n'est pas numérique : « {version} ». \
             sqlx la lit comme un entier ; un préfixe non numérique ne sera \
             pas ordonné comme vous le croyez."
        );
        trouvees.push((version, nom));
    }
    trouvees
}

#[test]
fn security_deux_migrations_ne_partagent_jamais_une_version() {
    let mut par_version: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (version, nom) in migrations_montantes() {
        par_version.entry(version).or_default().push(nom);
    }

    let collisions: Vec<String> = par_version
        .iter()
        .filter(|(_, noms)| noms.len() > 1)
        .map(|(version, noms)| {
            let mut noms = noms.clone();
            noms.sort();
            format!("  {version} → {}", noms.join(", "))
        })
        .collect();

    assert!(
        collisions.is_empty(),
        "Deux migrations partagent une version. sqlx indexe `_sqlx_migrations` \
         sur la version, pas sur le nom : sur une base neuve, `migrate!` \
         échouera en 23505 et AUCUNE migration ne passera.\n\n{}\n\n\
         Renumérotez celle qui n'a JAMAIS été appliquée, en vérifiant d'abord \
         ce que les bases enregistrent réellement :\n  \
         SELECT version, description FROM _sqlx_migrations ORDER BY version;\n\
         Déplacer une version déjà appliquée l'orphelinerait, ce qui casse \
         autrement. Cf. #939.",
        collisions.join("\n")
    );
}

/// Combien de migrations n'ont pas de `.down.sql`, au 2026-09-16.
///
/// Mesuré, pas souhaité : 93 sur 138. La convention « une descendante par
/// montante » ne vaut que pour les migrations récentes, et prétendre
/// l'inverse aurait fait de ce fichier un test rouge dès sa naissance —
/// c'est-à-dire un test qu'on apprend à ignorer.
///
/// Ce nombre ne doit que DESCENDRE. Il descend en écrivant une `.down.sql`
/// manquante, jamais en relâchant le seuil.
const SANS_DESCENDANTE_AU_2026_09_16: usize = 93;

#[test]
fn edge_le_nombre_de_montantes_sans_descendante_ne_grandit_pas() {
    let mut manquantes: Vec<String> = migrations_montantes()
        .into_iter()
        .map(|(_, nom)| nom)
        .filter(|nom| {
            !dossier_migrations()
                .join(nom.replace(".sql", ".down.sql"))
                .exists()
        })
        .collect();
    manquantes.sort();

    assert!(
        manquantes.len() <= SANS_DESCENDANTE_AU_2026_09_16,
        "{} migrations sans `.down.sql`, contre {} au 2026-09-16.\n\n\
         Une montante sans descendante ne se voit qu'au moment où l'on veut \
         revenir en arrière, c'est-à-dire au pire moment. Écrivez la \
         descendante de la migration que vous ajoutez ; ne relevez pas ce \
         seuil.\n\n{}",
        manquantes.len(),
        SANS_DESCENDANTE_AU_2026_09_16,
        manquantes.join("\n  ")
    );
}

#[test]
fn le_cliquet_lit_bien_quelque_chose() {
    // Un cliquet qui ne lit rien passe au vert pour de mauvaises raisons.
    let trouvees = migrations_montantes();
    assert!(
        trouvees.len() > 30,
        "Seulement {} migrations trouvées — le dossier a-t-il bougé ?",
        trouvees.len()
    );
}
