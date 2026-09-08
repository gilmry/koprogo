//! Cliquet : la part des scénarios BDD **chargés** qui ne déclarent pas leur
//! catégorie ne grossit pas.
//!
//! ── Ne compter que ce qui s'exécute ───────────────────────────────────────
//!
//! Un premier relevé annonçait 1109 scénarios BDD, dont 919 sans étiquette.
//! **Il comptait `specs/bdd-non-ecrits/`**, les 419 scénarios qu'aucun harnais
//! ne charge et qui ne s'exécutent nulle part (#838).
//!
//! Compter des scénarios non exécutés comme des tests est exactement le défaut
//! que #838 dénonçait ce matin : « ils continuent à figurer dans les 921
//! scénarios BDD que le projet revendique ». Je l'ai reproduit dans l'heure
//! qui a suivi, en mesurant une taxonomie.
//!
//! Le chiffre honnête ne porte que sur `tests/features/` :
//!
//! ```text
//! 690 scénarios chargés    593 sans étiquette   85 %
//! ```
//!
//! C'est la couche la plus muette des trois, devant Playwright (84 %) et Rust
//! (72 %).
//!
//! ── Ce que ce chiffre dit, et ce qu'il ne dit pas ─────────────────────────
//!
//! Il ne dit **pas** que 85 % des scénarios sont des chemins nominaux. Il dit
//! qu'on ne peut pas savoir. #427 SUPPOSE que « la majorité sont des
//! variations happy path » ; la mesure montre qu'on n'est pas en position de
//! le confirmer.
//!
//! ── Les deux contrôles ────────────────────────────────────────────────────
//!
//! Le second est celui qui compte : un cliquet sur le seul nombre de scénarios
//! sans étiquette **se satisferait d'en supprimer**, ou plus subtilement de
//! **retirer un fichier des harnais** — ce qui le rendrait muet ET invisible.
//! Le total des scénarios chargés ne doit donc pas baisser.
//!
//! Suivi en #427.

use std::fs;
use std::path::PathBuf;

/// Scénarios chargés sans étiquette de catégorie. **Ne doit que BAISSER.**
const SANS_ETIQUETTE_AU_2026_09_08: usize = 593;

/// Total des scénarios chargés. **Ne doit pas BAISSER.**
const TOTAL_CHARGES_AU_2026_09_08: usize = 690;

const CATEGORIES: [&str; 4] = ["@happy", "@edge", "@security", "@negative"];

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/features")
}

/// Les scénarios de `tests/features/`, avec la ligne d'étiquettes qui les
/// précède.
fn scenarios() -> Vec<(String, String)> {
    let mut sortie = Vec::new();
    let Ok(entrees) = fs::read_dir(racine()) else {
        return sortie;
    };
    let mut fichiers: Vec<PathBuf> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|c| c.extension().and_then(|e| e.to_str()) == Some("feature"))
        .collect();
    fichiers.sort();

    for chemin in fichiers {
        let Ok(source) = fs::read_to_string(&chemin) else {
            continue;
        };
        let nom = chemin
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            let nu = ligne.trim_start();
            if !nu.starts_with("Scenario") && !nu.starts_with("Scénario") {
                continue;
            }
            let etiquettes = i
                .checked_sub(1)
                .map(|j| lignes[j].to_string())
                .unwrap_or_default();
            sortie.push((format!("{nom} — {}", nu.trim()), etiquettes));
        }
    }
    sortie
}

#[test]
fn aucun_scenario_supplementaire_ne_tait_sa_categorie() {
    let nus: Vec<String> = scenarios()
        .into_iter()
        .filter(|(_, etiquettes)| !CATEGORIES.iter().any(|c| etiquettes.contains(c)))
        .map(|(titre, _)| titre)
        .collect();
    let n = nus.len();

    assert!(
        n <= SANS_ETIQUETTE_AU_2026_09_08,
        "{n} scénarios BDD chargés ne déclarent pas leur catégorie, contre \
         {SANS_ETIQUETTE_AU_2026_09_08} au 2026-09-08.\n\n\
         Un décompte de scénarios sans taxonomie rassure sans informer : il ne \
         dit pas si les chemins d'erreur, les bornes et les refus d'accès sont \
         éprouvés, ou si tout est nominal (#427).\n\n\
         Étiquetez le scénario : @happy, @edge, @security ou @negative.\n\n\
         Les trois derniers relevés :\n{}",
        nus.iter()
            .rev()
            .take(3)
            .map(|s| s.chars().take(90).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Le contrôle qui empêche de solder la dette en retirant des scénarios.
///
/// Plus subtil que la suppression : **retirer un fichier de la liste d'un
/// harnais** le rendrait muet ET invisible à ce cliquet, puisqu'il ne
/// regarde que `tests/features/`. C'est exactement ce qui est arrivé aux 419
/// scénarios de `specs/bdd-non-ecrits/` — ni verts, ni rouges, ni comptés
/// (#838).
#[test]
fn le_total_des_scenarios_charges_ne_baisse_pas() {
    let total = scenarios().len();
    assert!(
        total >= TOTAL_CHARGES_AU_2026_09_08,
        "{total} scénarios chargés relevés, contre \
         {TOTAL_CHARGES_AU_2026_09_08} au 2026-09-08.\n\n\
         Le cliquet de taxonomie se satisferait d'une suppression, ou d'un \
         fichier sorti de `tests/features/` — ce qui le rendrait muet ET \
         invisible. Ce contrôle l'interdit.\n\n\
         Si des scénarios ont été légitimement retirés, abaissez la constante \
         en disant lesquels."
    );
}
