//! Cliquet : tout `@skip` / `@wip` sur un scénario BDD porte sa raison.
//! Zéro toléré — pas de ratchet à la baisse.
//!
//! ── Le critère `@edge` de la story C7.2 (#427) ────────────────────────────
//!
//! « Étant donné les scénarios marqués `@skip` / `@wip`, quand on les
//! inventorie, alors chacun porte la raison de son skip. Douze l'étaient sans
//! justification : un test désactivé sans motif est un test perdu, pas un
//! test reporté. »
//!
//! `backend/tests/features/` n'a aujourd'hui aucun `@skip`/`@wip` (relevé le
//! 2026-09-16) : ce cliquet n'a donc rien à corriger, mais tout à empêcher —
//! le prochain `@wip` posé sans motif doit casser la build, pas s'ajouter
//! silencieusement aux 593 scénarios déjà sans étiquette de
//! `garde_taxonomie_bdd.rs`.
//!
//! Le pendant Playwright — `test.skip()`/`test.fixme()`, puisque Gherkin n'a
//! pas d'équivalent runtime côté frontend — vit dans
//! `frontend/src/lib/__tests__/garde-skips-e2e.test.ts`. Il y avait, lui,
//! une vraie dette : 9 `test.fixme()` sans commentaire de raison (la cause
//! réelle existait, mais seulement dans le message du commit `09e791b`, pas
//! dans le fichier). Traitée dans le même changement que ce fichier.
//!
//! ── La convention ──────────────────────────────────────────────────────
//!
//! Le tag `@skip`/`@wip` doit être précédé, sur la ligne juste au-dessus,
//! d'un commentaire `# raison: ...` ou `# reason: ...` :
//!
//! ```gherkin
//!   # raison: bloqué par #999, endpoint pas encore câblé
//!   @wip
//!   Scenario: Un truc pas fini
//! ```
//!
//! Suivi en #427.

use std::fs;
use std::path::PathBuf;

const MARQUEURS: [&str; 2] = ["@skip", "@wip"];

/// Les tags `@skip`/`@wip` de `source` qui ne sont pas précédés d'un
/// commentaire `# raison: ...` / `# reason: ...`. Chaque violation nomme le
/// scénario qui suit le tag.
fn tags_sans_raison(source: &str, fichier: &str) -> Vec<String> {
    let lignes: Vec<&str> = source.lines().collect();
    let mut sortie = Vec::new();

    for (i, ligne) in lignes.iter().enumerate() {
        let nu = ligne.trim();
        let porte_un_marqueur = nu.split_whitespace().any(|mot| MARQUEURS.contains(&mot));
        if !porte_un_marqueur {
            continue;
        }

        let precedente = i.checked_sub(1).map(|j| lignes[j].trim()).unwrap_or("");
        let justifie = precedente.starts_with('#') && {
            let bas = precedente.to_lowercase();
            bas.contains("raison") || bas.contains("reason")
        };
        if justifie {
            continue;
        }

        let nom_scenario = lignes[i + 1..]
            .iter()
            .map(|l| l.trim_start())
            .find(|n| n.starts_with("Scenario") || n.starts_with("Scénario"))
            .unwrap_or("(scénario introuvable après le tag)");

        sortie.push(format!("{fichier} — {nom_scenario} (tag: {nu})"));
    }

    sortie
}

#[test]
fn wip_sans_raison_est_detecte() {
    let source = "Feature: Exemple\n\n  @wip\n  Scenario: Un truc pas fini\n    Given ceci\n";
    let violations = tags_sans_raison(source, "exemple.feature");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("Un truc pas fini"));
}

#[test]
fn wip_avec_raison_en_francais_nest_pas_detecte() {
    let source = "Feature: Exemple\n\n  # raison: bloqué par #999, endpoint pas encore câblé\n  @wip\n  Scenario: Un truc pas fini\n    Given ceci\n";
    assert!(tags_sans_raison(source, "exemple.feature").is_empty());
}

#[test]
fn skip_avec_reason_en_anglais_nest_pas_detecte() {
    let source = "Feature: Exemple\n\n  # reason: flaky, tracked in #1000\n  @skip\n  Scenario: Un truc instable\n    Given ceci\n";
    assert!(tags_sans_raison(source, "exemple.feature").is_empty());
}

#[test]
fn happy_nest_jamais_concerne() {
    let source = "Feature: Exemple\n\n  @happy\n  Scenario: Chemin nominal\n    Given ceci\n";
    assert!(tags_sans_raison(source, "exemple.feature").is_empty());
}

#[test]
fn commentaire_sans_mot_cle_ne_justifie_pas() {
    let source = "Feature: Exemple\n\n  # TODO plus tard\n  @wip\n  Scenario: Un truc pas fini\n    Given ceci\n";
    let violations = tags_sans_raison(source, "exemple.feature");
    assert_eq!(violations.len(), 1);
}

#[test]
fn deux_scenarios_un_seul_non_justifie() {
    let source = "\
Feature: Exemple

  # raison: bloqué par #999
  @wip
  Scenario: Justifié
    Given ceci

  @skip
  Scenario: Pas justifié
    Given cela
";
    let violations = tags_sans_raison(source, "exemple.feature");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("Pas justifié"));
}

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/features")
}

/// L'inventaire réel : aucun `@skip`/`@wip` de `tests/features/` ne doit
/// être sans raison déclarée. Zéro toléré, pas de ratchet — voir le module.
#[test]
fn aucun_skip_ou_wip_sans_raison_dans_les_features_chargees() {
    let entrees = fs::read_dir(racine()).expect("tests/features introuvable");
    let mut fichiers: Vec<PathBuf> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|c| c.extension().and_then(|e| e.to_str()) == Some("feature"))
        .collect();
    fichiers.sort();

    let mut violations = Vec::new();
    for chemin in fichiers {
        let Ok(source) = fs::read_to_string(&chemin) else {
            continue;
        };
        let nom = chemin
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        violations.extend(tags_sans_raison(&source, &nom));
    }

    assert!(
        violations.is_empty(),
        "{} scénario(s) @skip/@wip sans raison déclarée.\n\n\
         Un skip sans motif est un test perdu, pas un test reporté (#427 C7.2).\n\
         Ajoutez un commentaire `# raison: ...` ou `# reason: ...` juste \
         au-dessus du tag.\n\n{}",
        violations.len(),
        violations.join("\n")
    );
}
