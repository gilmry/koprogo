//! Cliquet : le nombre de fichiers `.feature` que personne ne charge ne grossit pas.
//!
//! ── Le constat ─────────────────────────────────────────────────────────────
//!
//! `tests/features/` contient 91 fichiers. Les harnais en chargent 63.
//!
//! **28 fichiers, soit 412 scénarios, ne sont chargés par personne.**
//!
//! Ce chiffre vient de l'exécution de ce test, pas d'un `grep`. QUATRE
//! estimations successives ont donné 27, 26, 25 puis 28 ; les trois premières
//! étaient fausses, dont une à cause de ce test lui-même. C'est la troisième fois de la journée qu'une mesure non exécutée
//! est démentie — après les 114 occurrences de #762 qui en faisaient 118, et
//! les « 1 967 unwrap() » de #427 qui en font 39 en production.
//!
//! Le chargement est nominatif — `run_and_exit("tests/features/x.feature")`,
//! ou des listes explicites comme celle de `bdd_governance.rs`. Il n'existe
//! aucun glob de répertoire : un fichier qu'aucun harnais ne cite ne
//! s'exécute jamais.
//!
//! ── Pourquoi c'est pire qu'un test rouge ──────────────────────────────────
//!
//! Un test rouge se voit. Un fichier non chargé ne produit **aucun signal** :
//! ni vert, ni rouge, ni « skipped ». Il disparaît du décompte comme s'il
//! n'existait pas — tout en continuant à figurer dans les « 921 scénarios
//! BDD » que le projet revendique (#427).
//!
//! Le premier de la liste est le plus embarrassant. `legal_compliance.feature`
//! se présente comme « le POINT CENTRAL de suivi de conformité juridique »,
//! quarante et un scénarios rattachés chacun à un article du Code civil.
//! Aucun ne s'exécute — ce qui explique qu'il ait pu déclarer six règles « NON
//! implémentée (bloquant pour production) » pendant six mois alors qu'elles
//! l'étaient (#837). Rien ne pouvait le démentir.
//!
//! ── Pourquoi ce cliquet ne baisse pas dans le commit qui le pose ──────────
//!
//! C'est contraire à la règle habituelle du dépôt, et c'est délibéré.
//!
//! Aucun des 27 fichiers n'est un simple oubli de liste : leurs steps
//! n'existent pas. Le meilleur candidat, `resolution_agenda.feature`, a
//! environ vingt-sept de ses quarante-quatre phrases sans step déclaré. Les
//! brancher ferait passer la suite BDD au rouge sans qu'on sache quoi
//! corriger.
//!
//! Ce ne sont donc pas des tests oubliés, ce sont **des scénarios qui n'ont
//! jamais eu de code**. La baisse demande, fichier par fichier, d'écrire les
//! steps ou d'assumer que le fichier est un document et de le sortir de
//! `tests/features/`. Cela ne se fait pas en passant.
//!
//! Ce cliquet empêche seulement le 29ᵉ. Suivi en #838.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Fichiers `.feature` chargés par aucun harnais. **Ne doit que BAISSER.**
const DETTE_AU_2026_09_07: usize = 28;

fn orphelines() -> (Vec<String>, usize, usize) {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    // Tout nom de fichier `.feature` cité dans un harnais, même en commentaire.
    // Compter les commentaires comme un chargement rend la mesure PRUDENTE :
    // le nombre réel d'orphelines est supérieur ou égal à celui-ci.
    let mut cites: HashSet<String> = HashSet::new();
    if let Ok(entrees) = fs::read_dir(&racine) {
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(contenu) = fs::read_to_string(&chemin) else {
                continue;
            };
            // Un CHARGEMENT s'écrit « tests/features/X.feature » ; une simple
            // mention en commentaire s'écrit « X.feature ». Seule la première
            // compte.
            //
            // La version initiale de ce test acceptait les deux, et s'est
            // exclue elle-même de sa propre mesure : son commentaire citait
            // `legal_compliance.feature` en exemple, ce qui suffisait à le
            // faire passer pour chargé. Une garde qui se rend verte en se
            // documentant est exactement le défaut qu'elle traque.
            const PREFIXE: &str = "tests/features/";
            let mut reste = contenu.as_str();
            while let Some(pos) = reste.find(PREFIXE) {
                let apres_prefixe = &reste[pos + PREFIXE.len()..];
                let fin = apres_prefixe
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(apres_prefixe.len());
                let nom = &apres_prefixe[..fin];
                if !nom.is_empty() && apres_prefixe[fin..].starts_with(".feature") {
                    cites.insert(format!("{nom}.feature"));
                }
                reste = &reste[pos + PREFIXE.len()..];
            }
        }
    }

    let dossier = racine.join("features");
    let mut presentes = Vec::new();
    if let Ok(entrees) = fs::read_dir(&dossier) {
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.extension().and_then(|e| e.to_str()) == Some("feature") {
                presentes.push(chemin.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }
    presentes.sort();

    let total = presentes.len();
    let mut orphelines: Vec<String> = presentes
        .into_iter()
        .filter(|f| !cites.contains(f))
        .collect();
    orphelines.sort();
    let n = orphelines.len();
    (orphelines, total, n)
}

#[test]
fn aucun_fichier_feature_supplementaire_ne_devient_orphelin() {
    let (liste, total, n) = orphelines();

    assert!(
        n <= DETTE_AU_2026_09_07,
        "{n} fichiers `.feature` ne sont chargés par aucun harnais, contre \
         {DETTE_AU_2026_09_07} au 2026-09-07 (sur {total} présents).\n\n\
         Un fichier non chargé ne produit AUCUN signal : ni vert, ni rouge, ni \
         « skipped ». Il disparaît du décompte tout en continuant à compter \
         dans les scénarios que le projet revendique.\n\n\
         Ajoutez le fichier à la liste du harnais qui porte ses steps, ou \
         écrivez ces steps. S'il s'agit d'un document et non d'un test, \
         sortez-le de `tests/features/`.\n\n\
         Orphelins :\n{}",
        liste.join("\n")
    );
}

/// Sans ce contrôle, renommer le répertoire des features rendrait le cliquet
/// vert en ne trouvant plus rien à comparer.
#[test]
fn le_cliquet_voit_encore_les_deux_listes() {
    let (_, total, _) = orphelines();
    assert!(
        total > 50,
        "moins de cinquante fichiers `.feature` trouvés : le répertoire a \
         bougé, ou l'extension a changé. Vérifiez avant de vous réjouir."
    );
}
