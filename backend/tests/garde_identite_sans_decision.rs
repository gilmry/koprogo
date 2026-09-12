//! Cliquet : une route qui prend une identité doit s'en servir pour DÉCIDER
//! (#864).
//!
//! ── Le défaut que ce cliquet borne ────────────────────────────────────────
//!
//! Une route qui écrit `user: AuthenticatedUser` dans sa signature **a l'air
//! gardée**. Elle passe la revue, elle passe les autres gardes, elle
//! journalise consciencieusement qui a fait quoi.
//!
//! Quatre `DELETE` ont été vérifiés puis corrigés le 2026-09-11 : ils
//! prenaient l'identité et ne s'en servaient que pour **journaliser après
//! coup**. Témoin à l'appui, le syndic du cabinet A supprimait le budget du
//! cabinet B et recevait `204 No Content` ; le journal d'audit enregistrait le
//! geste comme régulier.
//!
//! Une route SANS identité se voit : elle est nue, `garde_identite_absente` la
//! compte. Une route qui prend une identité et l'ignore n'était comptée par
//! rien.
//!
//! ── Ce que le compte VAUT, et ce qu'il ne vaut pas ────────────────────────
//!
//! **C'est un plafond, pas un verdict.** Le détecteur est textuel : il cherche
//! dans le corps du handler la trace d'une décision — `verify_`, `scope_guard`,
//! `Forbidden`, `require_organization`, `is_superadmin`, `.role ==`. Il ne
//! comprend pas le code.
//!
//! Deux conséquences assumées :
//!
//! - **Des faux positifs.** Une route dont le cas d'usage filtre lui-même par
//!   utilisateur — les notifications de l'appelant, par exemple — n'a besoin
//!   d'aucun contrôle dans le handler et sera pourtant comptée.
//! - **`user.user_id` NE COMPTE PAS** comme décision. C'est le marqueur du
//!   journal d'audit, qui enregistre après coup et n'interdit rien. L'inclure
//!   ferait passer les quatre `DELETE` corrigés pour des routes gardées —
//!   c'est l'erreur que j'ai faite au premier comptage, et elle divisait le
//!   résultat par quatre.
//!
//! Le cliquet ne prétend donc pas que 103 routes sont vulnérables. Il dit que
//! 103 méritent une lecture, et **qu'aucune de plus ne doit apparaître**.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Ce qui DÉCIDE. Volontairement court : chaque entrée est un idiome par
/// lequel ce dépôt refuse effectivement un accès.
const DECISION: [&str; 9] = [
    "verify_",
    "scope_guard",
    "can_",
    "Forbidden",
    "caller_from_user",
    "require_organization",
    "require_role",
    "is_superadmin",
    ".role ==",
];

/// Mesuré le 2026-09-12, après l'isolement des huit routes IoT de #864.
///
/// Relevé, jamais estimé. Il ne peut que descendre.
const SANS_DECISION_AU_2026_09_12: usize = 95;

/// Les handlers qui prennent `AuthenticatedUser` sans trace de décision.
fn sans_decision() -> BTreeMap<String, String> {
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers");
    let mut trouves = BTreeMap::new();

    for entree in fs::read_dir(&dossier).expect("le dossier des handlers doit être lisible") {
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

        // Découpe par `pub async fn` : le corps d'un handler va jusqu'au
        // suivant. Grossier, et suffisant — un handler ne contient pas d'autre
        // `pub async fn`.
        let morceaux: Vec<&str> = source.split("pub async fn ").collect();
        for morceau in morceaux.iter().skip(1) {
            let nom = morceau
                .split(['(', '<'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let Some(fin_signature) = morceau.find(')') else {
                continue;
            };
            let (signature, corps) = morceau.split_at(fin_signature);
            if !signature.contains("AuthenticatedUser") {
                continue;
            }
            if !DECISION.iter().any(|k| corps.contains(k)) {
                trouves.insert(format!("{fichier}::{nom}"), fichier.clone());
            }
        }
    }
    trouves
}

#[test]
fn le_cliquet_lit_bien_des_handlers() {
    // Vérification d'aveuglement : si le découpage ou le chemin cessent de
    // correspondre, la règle suivante devient vraie sur zéro handler.
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers");
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

    // Et la mesure doit rendre un nombre plausible : zéro signifierait que le
    // détecteur ne voit plus aucune signature.
    let n = sans_decision().len();
    assert!(
        n > 0,
        "Aucune route ne prend `AuthenticatedUser` sans marqueur de décision. \
         Ce serait une excellente nouvelle, et elle est invraisemblable tant \
         que #864 est ouverte : le détecteur ne lit probablement plus rien."
    );
}

#[test]
fn la_dette_didentite_sans_decision_ne_grossit_pas() {
    let trouves = sans_decision();
    let n = trouves.len();

    // Les dix fichiers les plus chargés, pour que le message serve à agir.
    let mut par_fichier: BTreeMap<&String, usize> = BTreeMap::new();
    for fichier in trouves.values() {
        *par_fichier.entry(fichier).or_insert(0) += 1;
    }
    let mut classement: Vec<_> = par_fichier.into_iter().collect();
    classement.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let tete: Vec<String> = classement
        .iter()
        .take(10)
        .map(|(f, n)| format!("  {n:3}  {f}"))
        .collect();

    assert!(
        n <= SANS_DECISION_AU_2026_09_12,
        "{n} routes prennent `AuthenticatedUser` sans trace de décision, contre \
         {SANS_DECISION_AU_2026_09_12} mesurées le 2026-09-12.\n\n\
         Une route qui prend une identité et ne s'en sert que pour journaliser \
         A L'AIR gardée : elle passe la revue, elle passe les autres gardes, et \
         elle laisse passer le geste.\n\n\
         `user.user_id` ne compte pas — c'est le marqueur du journal d'audit, \
         qui enregistre après coup et n'interdit rien.\n\n\
         Les fichiers les plus chargés :\n{}",
        tete.join("\n")
    );
}
