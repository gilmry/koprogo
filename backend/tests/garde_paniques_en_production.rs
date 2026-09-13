//! Cliquet : les points de panique du code de production ne se multiplient pas.
//!
//! ── Ce que l'audit annonçait ───────────────────────────────────────────────
//!
//! L'audit du 2026-04-29 comptait « **1 967 `unwrap()`/`expect()`** non
//! couverts par des tests d'erreur » et en faisait l'un des cinq constats
//! justifiant un NO-GO de release (#427).
//!
//! Le chiffre est aujourd'hui de 2 546 si l'on compte de la même façon. Il
//! semble donc avoir empiré d'un tiers.
//!
//! ── Ce que le chiffre mesurait vraiment ───────────────────────────────────
//!
//! Presque uniquement du code de test.
//!
//! ```text
//!   2 546   toutes occurrences dans backend/src
//!      92   hors des modules `#[cfg(test)]`
//!      39   hors, aussi, des fichiers `*_test.rs`
//! ```
//!
//! Dans un test, `unwrap()` est la pratique NORMALE : un test qui échoue doit
//! paniquer, c'est ainsi qu'il signale. Les compter avec le code de production
//! revient à reprocher à un détecteur de fumée de faire du bruit.
//!
//! L'issue #427 s'ouvre pourtant sur « la métrique cache la maladie ». Ici,
//! elle l'exagérait : 1 967 alarmants pour 39 réels. Une métrique fausse dans
//! le sens pessimiste use la même chose qu'une métrique fausse dans l'autre
//! sens — la confiance qu'on lui accorde.
//!
//! ── Ce que ce cliquet garde ───────────────────────────────────────────────
//!
//! Les 39 restants sont, eux, de vrais points de panique en production. Ils ne
//! sont pas tous illégitimes : `main.rs` peut refuser de démarrer sans
//! configuration, `config.rs` aussi. Mais un `unwrap()` dans un gestionnaire
//! HTTP transforme une donnée inattendue en 500 sans journal utile.
//!
//! Le nombre ne doit donc que **baisser**. Celui qui touche un de ces fichiers
//! remplace le point de panique qu'il croise par une erreur typée.
//!
//! Le comptage suit les accolades du module de test plutôt que de tronquer au
//! premier `#[cfg(test)]` : quatre fichiers du dépôt portent du code APRÈS
//! leur module de test, et une troncature naïve les aurait sous-comptés.

use std::fs;
use std::path::{Path, PathBuf};

/// Points de panique dans le code de production. **Ne doit que BAISSER.**
const DETTE_AU_2026_09_07: usize = 39;

/// Retire chaque module `#[cfg(test)]` en suivant ses accolades.
fn hors_modules_de_test(texte: &str) -> String {
    let mut sortie = String::new();
    let mut reste = texte;
    while let Some(debut) = reste.find("#[cfg(test)]") {
        sortie.push_str(&reste[..debut]);
        let apres = &reste[debut..];
        let Some(ouvrante) = apres.find('{') else {
            return sortie;
        };
        let mut profondeur = 0usize;
        let mut fin = None;
        for (i, c) in apres[ouvrante..].char_indices() {
            match c {
                '{' => profondeur += 1,
                '}' => {
                    profondeur -= 1;
                    if profondeur == 0 {
                        fin = Some(ouvrante + i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match fin {
            Some(f) => reste = &apres[f..],
            None => return sortie,
        }
    }
    sortie.push_str(reste);
    sortie
}

fn fichiers_de_production(racine: &Path, trouves: &mut Vec<PathBuf>) {
    let Ok(entrees) = fs::read_dir(racine) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            fichiers_de_production(&chemin, trouves);
        } else if chemin.extension().and_then(|e| e.to_str()) == Some("rs") {
            let nom = chemin.file_stem().unwrap_or_default().to_string_lossy();
            // Un fichier `*_test.rs` est du code de test, même sans `#[cfg(test)]`.
            if !nom.ends_with("_test") && !nom.ends_with("_tests") {
                trouves.push(chemin);
            }
        }
    }
}

fn compter() -> (usize, Vec<String>) {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut fichiers = Vec::new();
    fichiers_de_production(&racine, &mut fichiers);

    let mut total = 0;
    let mut details = Vec::new();
    for chemin in fichiers {
        let Ok(contenu) = fs::read_to_string(&chemin) else {
            continue;
        };
        let utile = hors_modules_de_test(&contenu);
        let n = utile.matches(".unwrap()").count() + utile.matches(".expect(").count();
        if n > 0 {
            total += n;
            details.push(format!(
                "{:>3}  {}",
                n,
                chemin.strip_prefix(&racine).unwrap_or(&chemin).display()
            ));
        }
    }
    details.sort();
    details.reverse();
    (total, details)
}

#[test]
fn les_points_de_panique_ne_se_multiplient_pas() {
    let (total, details) = compter();

    assert!(
        total <= DETTE_AU_2026_09_07,
        "Les points de panique du code de production ont AUGMENTÉ : {total} \
         contre {DETTE_AU_2026_09_07} au 2026-09-07.\n\n\
         Un `unwrap()` dans un gestionnaire HTTP transforme une donnée \
         inattendue en 500 sans journal utile. Préférez une erreur typée, ou \
         un `expect(\"…\")` dont le message dit ce qui était attendu.\n\n\
         Par fichier :\n{}",
        details.join("\n")
    );
}

/// Sans ce contrôle, exclure un répertoire de trop rendrait le cliquet
/// définitivement vert en ne trouvant plus rien à compter.
#[test]
fn le_cliquet_examine_encore_du_code_de_production() {
    let (total, _) = compter();
    assert!(
        total > 10,
        "plus aucun point de panique trouvé en production : c'est possible, \
         mais vérifiez d'abord que le comptage regarde encore au bon endroit."
    );
}

/// Le comptage doit ignorer les modules de test — c'est TOUTE la différence
/// entre 2 546 et 39.
#[test]
fn le_comptage_ignore_les_modules_de_test() {
    let source = r#"
fn produire() { valeur.unwrap() }

#[cfg(test)]
mod tests {
    fn t1() { a.unwrap(); }
    fn t2() { b.expect("x"); }
}

fn apres_le_module_de_test() { autre.unwrap() }
"#;
    let utile = hors_modules_de_test(source);
    assert_eq!(
        utile.matches(".unwrap()").count(),
        2,
        "les deux `unwrap()` de production doivent être comptés, y compris \
         celui qui SUIT le module de test"
    );
    assert_eq!(
        utile.matches(".expect(").count(),
        0,
        "aucun appel du module de test ne doit être compté"
    );
}
