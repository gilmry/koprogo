//! Cliquet : le classement des erreurs par sous-chaîne ne grossit pas.
//!
//! ── Le défaut ──────────────────────────────────────────────────────────────
//!
//! Des gestionnaires HTTP décident du code de réponse en cherchant des
//! sous-chaînes dans un message d'erreur :
//!
//! ```ignore
//! if e.contains("not found") { NotFound } else { InternalServerError }
//! ```
//!
//! Tout message qui ne correspond à aucun motif tombe dans la branche « erreur
//! inattendue », donc en **500**.
//!
//! Le relevé des motifs cherchés disait le reste : `not found` cinquante-deux
//! fois, `introuvable` six fois. **Le même concept, en deux langues.** Les
//! messages du domaine juridique sont en français, ceux des couches techniques
//! en anglais, et un gestionnaire qui ne connaît qu'une des deux rend 500 sur
//! l'autre — une panne serveur là où l'utilisateur a demandé une ressource
//! absente.
//!
//! Constaté le 2026-09-04 : « Impossible de déterminer l'ACP : une écriture
//! manuelle doit désigner un immeuble » ne correspondait à rien, et une saisie
//! incomplète ressortait en 500 (#762).
//!
//! ── Pourquoi un cliquet, et pas un interdit ───────────────────────────────
//!
//! La vraie correction est de typer les erreurs : `Result<_, AppError>` plutôt
//! que `Result<_, String>` (#555, #762). C'est une migration de plus de trois
//! mille sites, qui ne se fait pas en une fois.
//!
//! En attendant, le nombre ne doit que **baisser**. Chaque gestionnaire touché
//! passe par `classification_erreurs`, qui rassemble le lexique bilingue en un
//! seul endroit — de sorte que la prochaine langue ou le prochain synonyme
//! s'ajoute une fois et profite à tous, au lieu d'être découvert site par
//! site, à chaque 500 injustifié.
//!
//! Un lexique dispersé sur cent vingt-six sites ne se corrige jamais
//! entièrement : on corrige celui qui a fait mal, et on laisse les autres.
//!
//! Le compte part de **118** et non de 130 : `poll_handlers.rs` a été migré
//! dans le commit qui pose ce cliquet, parce qu'un cliquet posé sans une
//! première baisse n'est qu'une constatation.
//!
//! ── Une leçon sur la mesure elle-même ─────────────────────────────────────
//!
//! La constante valait d'abord 114, chiffre obtenu par `grep -rn`. Le test a
//! échoué du premier coup : `grep -rn` compte des LIGNES, ce test compte des
//! OCCURRENCES, et quatre lignes en portent deux.
//!
//! Un écart de quatre, sans conséquence ici — mais il rappelle qu'une mesure
//! non exécutée n'est qu'une estimation. Le cliquet a corrigé son propre
//! chiffre de départ, ce qu'aucune relecture n'aurait fait.

use std::fs;
use std::path::Path;

/// Occurrences de `.contains("` dans les gestionnaires. **Ne doit que BAISSER.**
const DETTE_AU_2026_09_07: usize = 39;

fn compter(repertoire: &Path) -> (usize, Vec<String>) {
    let mut total = 0;
    let mut details = Vec::new();
    let entrees = fs::read_dir(repertoire).expect("répertoire des gestionnaires lisible");
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let contenu = fs::read_to_string(&chemin).expect("fichier lisible");
        let n = contenu.matches(".contains(\"").count();
        if n > 0 {
            total += n;
            details.push(format!(
                "{:>3}  {}",
                n,
                chemin.file_name().unwrap().to_string_lossy()
            ));
        }
    }
    details.sort();
    details.reverse();
    (total, details)
}

#[test]
fn le_classement_par_souschaines_ne_grossit_pas() {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers");
    let (total, details) = compter(&racine);

    assert!(
        total <= DETTE_AU_2026_09_07,
        "Le classement d'erreurs par sous-chaîne a GROSSI : {total} occurrences \
         contre {DETTE_AU_2026_09_07} au 2026-09-07.\n\n\
         Un message qui ne correspond à aucun motif tombe en 500. C'est ainsi \
         qu'une saisie incomplète est ressortie en panne serveur le \
         2026-09-04, parce que le message du domaine était en français et le \
         motif cherché en anglais.\n\n\
         Employez `infrastructure::web::classification_erreurs`, qui rassemble \
         le lexique des deux langues en un seul endroit. La vraie correction \
         reste de typer les erreurs (#555, #762).\n\n\
         Par fichier :\n{}",
        details.join("\n")
    );
}

/// Sans ce contrôle, déplacer le répertoire des gestionnaires rendrait le
/// cliquet définitivement vert en ne trouvant plus rien à compter.
///
/// ── Ce que ce contrôle exigeait, et pourquoi c'était faux ────────────────
///
/// Il demandait « plus de cinquante occurrences de `.contains(\"` ». C'était
/// confondre deux choses : que le détecteur LISE encore les fichiers, et que
/// la dette soit encore grosse.
///
/// La conséquence s'est vue le 2026-09-08 : le routage des occurrences
/// génériques vers `classification_erreurs` a fait tomber la dette de 118 à
/// 39, et **ce contrôle a échoué pour cette raison**. Une garde qui punit la
/// disparition de ce qu'elle traque décourage exactement le travail qu'elle
/// réclame.
///
/// Il vérifie désormais que les gestionnaires sont lus — leur nombre, et la
/// présence de réponses HTTP — sans rien exiger du compte de violations. La
/// dette peut donc légitimement atteindre zéro.
#[test]
fn le_cliquet_lit_encore_les_gestionnaires() {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers");

    let mut fichiers = 0usize;
    let mut reponses_http = 0usize;
    for entree in fs::read_dir(&racine)
        .expect("répertoire des gestionnaires lisible")
        .flatten()
    {
        let chemin = entree.path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        fichiers += 1;
        let contenu = fs::read_to_string(&chemin).expect("fichier lisible");
        reponses_http += contenu.matches("HttpResponse::").count();
    }

    assert!(
        fichiers > 40 && reponses_http > 500,
        "{fichiers} gestionnaires lus, {reponses_http} réponses HTTP trouvées. \
         Le répertoire a bougé, ou l'analyse ne lit plus rien : le cliquet \
         serait alors vert faute de matière. Vérifiez avant de vous réjouir."
    );
}
