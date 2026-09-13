//! Cliquet : un harnais de test qui n'est cité nulle part ne s'exécute jamais.
//!
//! ── Le défaut ─────────────────────────────────────────────────────────────
//!
//! `ci.yml` n'appelle pas `cargo test --tests`. Il énumère ses cibles **par
//! nom**, en cinq blocs :
//!
//! ```yaml
//! cargo test --no-fail-fast \
//!   --test e2e_security_incidents --test e2e_resolutions \
//!   --test e2e_meetings ...
//! ```
//!
//! Un fichier ajouté dans `tests/` et absent de ces listes compile, passe en
//! local, et **ne tourne jamais**. La CI reste verte : elle n'a rien vu.
//!
//! ── Ce que cela avait déjà coûté ──────────────────────────────────────────
//!
//! Mesuré le 2026-09-11 : 92 cibles, 90 citées, **deux jamais exécutées**.
//!
//! L'une venait d'être écrite. L'autre est
//! `e2e_cloisonnement_inter_organisations` — les trois tests sur lesquels le
//! WBS s'appuie pour déclarer la dette de cloisonnement résorbée : « un test
//! end-to-end prouve qu'aucune donnée ne traverse d'une organisation à
//! l'autre ». Il passe. Il n'avait jamais tourné en CI.
//!
//! C'est le motif dominant de ce dépôt appliqué aux tests eux-mêmes : une
//! capacité écrite, correcte, et inatteignable.
//!
//! ── Pourquoi une liste, et pourquoi on la garde plutôt que la supprimer ───
//!
//! Les blocs existent pour paralléliser : les harnais e2e montent chacun une
//! base en conteneur, et les répartir tient la durée de la CI sous contrôle.
//! Remplacer le tout par `--tests` sérialiserait 92 harnais.
//!
//! La liste a donc une raison d'être. Ce qui n'en avait pas, c'est qu'elle
//! soit tenue de mémoire.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn racine_du_depot() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("le dépôt est le parent de backend/")
        .to_path_buf()
}

/// Les cibles déclarées par `backend/tests/*.rs`.
fn cibles() -> BTreeSet<String> {
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    fs::read_dir(&dossier)
        .expect("backend/tests doit être lisible")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string)
        })
        .collect()
}

/// Les cibles citées par un `--test <nom>` dans les workflows.
fn citees() -> BTreeSet<String> {
    let workflows = racine_du_depot().join(".github/workflows");
    let mut vues = BTreeSet::new();
    for entree in fs::read_dir(&workflows).expect(".github/workflows doit être lisible") {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.extension().is_none_or(|x| x != "yml") {
            continue;
        }
        let texte = fs::read_to_string(&chemin).expect("workflow lisible");
        for morceau in texte.split("--test") {
            if let Some(nom) = morceau.split_whitespace().next() {
                if nom
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    && !nom.is_empty()
                {
                    vues.insert(nom.to_string());
                }
            }
        }
    }
    vues
}

#[test]
fn le_cliquet_lit_bien_des_cibles_et_des_citations() {
    // Vérification d'aveuglement. Si la lecture du dossier ou des workflows
    // cesse de correspondre — répertoire déplacé, extension changée — la règle
    // suivante devient vraie sur deux ensembles vides.
    let cibles = cibles();
    let citees = citees();
    assert!(
        cibles.len() > 50,
        "Seulement {} cibles lues dans backend/tests/. Le chemin ne correspond \
         plus : un vert obtenu sur zéro cible ne dit rien.",
        cibles.len()
    );
    assert!(
        citees.len() > 50,
        "Seulement {} cibles citées dans les workflows. Le motif `--test <nom>` \
         ne correspond plus.",
        citees.len()
    );
}

#[test]
fn chaque_harnais_est_execute_quelque_part() {
    let jamais: Vec<String> = cibles().difference(&citees()).cloned().collect();

    assert!(
        jamais.is_empty(),
        "Ces harnais ne sont cités par AUCUN workflow, donc ne s'exécutent \
         jamais :\n  {}\n\n\
         Ils compilent, ils passent en local, et la CI reste verte sans les \
         avoir vus. Ajoutez-les à un bloc `cargo test --no-fail-fast --test …` \
         de `.github/workflows/ci.yml`, en choisissant celui dont la durée \
         supporte un harnais de plus.",
        jamais.join("\n  ")
    );
}
