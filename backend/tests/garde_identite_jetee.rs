//! Cliquet : une identité PRISE et JETÉE doit porter sa justification (#882).
//!
//! ── Le motif, et pourquoi il est exact ────────────────────────────────────
//!
//! En instruisant #864, un motif à un seul caractère s'est révélé plus
//! précis que la liste `DECISION` de `garde_identite_sans_decision` :
//!
//! ```text
//! grep -nE '^\s+_[a-z_]*: AuthenticatedUser' backend/src/infrastructure/web/handlers/*.rs
//! ```
//!
//! Le souligné n'est pas une approximation textuelle : c'est l'auteur qui
//! écrit, DANS LE TYPE, qu'il prend cette identité et ne s'en sert pas.
//! Contrairement au détecteur de #864 (une liste de mots-clés qui devine si
//! le corps « décide »), celui-ci n'a ni faux positif ni faux négatif sur ce
//! qu'il mesure : soit le paramètre commence par `_`, soit non.
//!
//! ── Ce que ce cliquet exige, et pourquoi il n'a pas de liste d'exceptions ──
//!
//! « Un souligné devant `AuthenticatedUser` est une décision explicite ; si
//! elle est légitime, elle s'écrit en commentaire au-dessus. » (#882)
//!
//! Le motif étant exact, le cliquet peut l'être aussi : aucune liste
//! d'exceptions à tenir à jour ailleurs, seulement un commentaire à écrire
//! au bon endroit — juste au-dessus du gestionnaire signalé — et à
//! référencer par `#882`. `chaque_identite_jetee_est_justifiee` vérifie
//! exactement cela : zéro identité jetée sans justification écrite.
//!
//! Une justification n'atteste PAS que le détecteur s'est tu. Elle atteste
//! que le gestionnaire a été LU et classé : légitimement dépourvu de
//! périmètre à cloisonner (fichier statique, par ex.), ou dette assumée avec
//! sa raison, en attendant correction ou arbitrage produit.

use std::fs;
use std::path::{Path, PathBuf};

fn dossier_handlers() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers")
}

/// Une ligne de PARAMÈTRE qui nomme son identité avec un souligné.
///
/// Motif exact : `_[a-z_]*: AuthenticatedUser`, en tête de ligne après
/// indentation — la forme même que `grep -nE '^\s+_[a-z_]*: AuthenticatedUser'`
/// cherche dans l'issue.
fn est_ligne_identite_jetee(ligne: &str) -> bool {
    let sans_indentation = ligne.trim_start();
    if sans_indentation.len() == ligne.len() {
        // Aucune indentation : ne peut pas être un paramètre de signature
        // (une signature de gestionnaire est toujours indentée d'un niveau).
        return false;
    }
    let Some(reste) = sans_indentation.strip_prefix('_') else {
        return false;
    };
    let Some(i) = reste.find(':') else {
        return false;
    };
    let nom = &reste[..i];
    if !nom.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        return false;
    }
    reste[i + 1..].trim_start().starts_with("AuthenticatedUser")
}

/// Une identité jetée relevée dans un handler, et si elle porte sa
/// justification écrite.
struct IdentiteJetee {
    identifiant: String,
    justifiee: bool,
}

/// Parcourt les handlers et relève chaque gestionnaire dont la signature
/// jette son identité, avec la question : le commentaire immédiatement
/// au-dessus de `pub async fn` référence-t-il `#882` ?
///
/// Découpage par occurrences de `pub async fn ` : le corps d'un gestionnaire
/// va jusqu'au suivant, comme `garde_identite_sans_decision::sans_decision`.
/// Le PRÉAMBULE d'un gestionnaire — où vit sa justification — est isolé par
/// le dernier `\n}\n` qui précède : c'est la fin de la fonction précédente,
/// jamais le milieu de son corps, donc jamais une fausse justification
/// piochée dans un commentaire qui parle d'autre chose.
fn releve() -> Vec<IdentiteJetee> {
    let mut sortie = Vec::new();

    for entree in
        fs::read_dir(dossier_handlers()).expect("le dossier des handlers doit être lisible")
    {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let fichier = chemin
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let source = fs::read_to_string(&chemin).expect("handler lisible");

        let positions: Vec<usize> = source
            .match_indices("pub async fn ")
            .map(|(i, _)| i)
            .collect();

        for (idx, &debut) in positions.iter().enumerate() {
            let fin = positions.get(idx + 1).copied().unwrap_or(source.len());
            let apres_nom = &source[debut + "pub async fn ".len()..fin];

            let nom = apres_nom
                .split(['(', '<'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            // Signature = jusqu'à la première parenthèse fermante. Aucun type
            // de paramètre de ce dépôt n'en contient avant la fin réelle de
            // la liste (`web::Path<Uuid>`, `AuthenticatedUser`, ...) — même
            // hypothèse que `garde_identite_sans_decision::sans_decision`.
            let Some(fin_signature) = apres_nom.find(')') else {
                continue;
            };
            let signature = &apres_nom[..fin_signature];

            if !signature.lines().any(est_ligne_identite_jetee) {
                continue;
            }

            let avant = &source[..debut];
            let debut_preambule = avant.rfind("\n}\n").map(|i| i + 1).unwrap_or(0);
            let preambule = &source[debut_preambule..debut];

            sortie.push(IdentiteJetee {
                identifiant: format!("{fichier}::{nom}"),
                justifiee: preambule.contains("#882"),
            });
        }
    }

    sortie
}

#[test]
fn le_cliquet_lit_bien_des_handlers() {
    let dossier = dossier_handlers();
    let fichiers = fs::read_dir(&dossier)
        .expect("dossier lisible")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .count();
    assert!(
        fichiers > 20,
        "Seulement {fichiers} fichiers de handlers lus. Le chemin ne correspond \
         plus : un vert obtenu sur rien ne dit rien."
    );
}

/// Le cœur du cliquet : zéro identité jetée sans justification écrite.
///
/// « Un cliquet compte zéro identité jetée sans justification écrite » —
/// critère Gherkin de la story #882.
#[test]
fn chaque_identite_jetee_est_justifiee() {
    let trouvees = releve();
    let non_justifiees: Vec<&str> = trouvees
        .iter()
        .filter(|t| !t.justifiee)
        .map(|t| t.identifiant.as_str())
        .collect();

    assert!(
        non_justifiees.is_empty(),
        "Ces gestionnaires jettent leur identité (`_xxx: AuthenticatedUser`) sans \
         justification écrite référençant #882 juste au-dessus de leur \
         signature :\n{}\n\n\
         Un souligné devant `AuthenticatedUser` est une décision explicite. Si \
         cloisonner ne s'applique pas à cette route (donnée statique, par \
         exemple), la raison s'écrit en commentaire au-dessus, avec `#882`. \
         Sinon, corrigez : appelez le garde de périmètre qui convient et \
         retirez le souligné.",
        non_justifiees
            .iter()
            .map(|r| format!("  {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Mesuré le 2026-09-13, après correction des dix-huit routes de #882 : deux
/// critiques (`get_overdue_calls`, `get_outstanding_contributions`) et six
/// autres corrigées par appel d'un garde de périmètre existant ; quatre
/// restent classées ci-dessous, chacune avec sa raison écrite.
///
/// Ce nombre ne peut que baisser : soit une identité classée est corrigée
/// (elle sort du relevé), soit elle ne l'est jamais devenue (elle n'y entre
/// pas). Il ne baisse jamais par un assouplissement du détecteur.
const IDENTITES_JETEES_AU_2026_09_13: usize = 4;

#[test]
fn le_nombre_didentites_jetees_ne_grossit_pas() {
    let n = releve().len();
    assert!(
        n <= IDENTITES_JETEES_AU_2026_09_13,
        "{n} gestionnaires jettent leur identité, contre \
         {IDENTITES_JETEES_AU_2026_09_13} mesurés le 2026-09-13. Une identité \
         jetée DE PLUS doit être classée (commentaire `#882` au-dessus) ou \
         corrigée avant que ce nombre ne remonte."
    );
}

/// Les deux routes les plus graves de #882 ne jettent plus leur identité.
///
/// `GET /call-for-funds/overdue` rendait les arriérés de TOUTE l'instance à
/// n'importe quel utilisateur authentifié ; `GET
/// /owner-contributions/outstanding` rendait ceux d'un copropriétaire
/// quelconque, en connaissant son seul UUID. Les deux sont désormais
/// bornées par organisation à la SIGNATURE du cas d'usage / par
/// `verify_owner_org_access` au niveau du gestionnaire — pas seulement
/// classées.
#[test]
fn security_les_deux_routes_critiques_ne_jettent_plus_leur_identite() {
    let trouvees = releve();
    for identifiant in [
        "call_for_funds_handlers.rs::get_overdue_calls",
        "owner_contribution_handlers.rs::get_outstanding_contributions",
    ] {
        assert!(
            !trouvees.iter().any(|t| t.identifiant == identifiant),
            "{identifiant} jette de nouveau son identité : la route la plus \
             grave de #882 redevient ouverte à l'instance entière."
        );
    }
}
