//! Cliquet : le schéma ne promet pas ce que le code ne tient pas.
//!
//! ── Le constat ────────────────────────────────────────────────────────────
//!
//! 84 tables sont créées et non supprimées par les migrations. **Onze ne sont
//! nommées nulle part dans `src/`**, ni dans le cache `.sqlx`.
//!
//! La plus embarrassante est `meeting_proxy_mandates`, créée avec un index
//! partiel et une vue :
//!
//! ```sql
//! COMMENT ON VIEW proxy_mandate_stats IS
//!   'Aggregate proxy mandate stats for validation (max 3 mandats, max 10% quotas)';
//! ```
//!
//! Rien n'y écrit jamais. Les procurations vivent dans
//! `convocation_recipients.proxy_owner_id`, et la règle est appliquée par
//! `domain/copropriete/procurations.rs`. La règle est donc tenue — mais **la
//! vue qui prétend la vérifier renvoie toujours zéro ligne**. Qui
//! l'interrogerait pour contrôler une assemblée conclurait qu'aucune
//! procuration n'existe.
//!
//! C'est le motif dominant du dépôt, appliqué au schéma : écrit, documenté,
//! commenté, et inatteignable.
//!
//! ── Ce que ce cliquet garde ───────────────────────────────────────────────
//!
//! Il n'exige pas la suppression : une migration de suppression est
//! irréversible en production, et certaines de ces tables sont peut-être
//! réservées à une fonctionnalité à venir. Il empêche la **douzième**.
//!
//! Une table créée aujourd'hui doit être lue par du code, ou rejoindre la
//! liste `RESERVEES` avec l'issue qui l'activera. Une liste sans
//! justification se remplirait toute seule.
//!
//! Suivi en #846.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Les tables muettes connues au 2026-09-08.
///
/// Un cliquet qui ne compterait que le NOMBRE se satisferait d'une table
/// supprimée et d'une autre ajoutée. On borne donc l'ensemble : toute table
/// muette qui n'est pas dans cette liste fait échouer la garde, et en retirer
/// une est toujours permis.
const MUETTES_AU_2026_09_08: &[&str] = &[
    "api_key_usage",
    "mcp_models",
    "mcp_requests",
    "mcp_responses",
    "mcp_sessions",
    "mcp_tasks",
    "mcp_tool_calls",
    "meeting_proxy_mandates",
    "mqtt_devices",
    "mqtt_messages",
    "user_building_access",
];

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Le nom de table qui suit un mot-clé, à la position donnée.
fn nom_apres(source: &str, mot_cle: &str) -> Vec<String> {
    let bas = source.to_lowercase();
    let mut sortie = Vec::new();
    let mut depuis = 0usize;
    while let Some(pos) = bas[depuis..].find(mot_cle) {
        let debut = depuis + pos + mot_cle.len();
        let reste = bas[debut..].trim_start();
        // « IF NOT EXISTS » / « IF EXISTS » se glissent entre le mot-clé et le nom.
        let reste = reste
            .strip_prefix("if not exists")
            .or_else(|| reste.strip_prefix("if exists"))
            .unwrap_or(reste)
            .trim_start();
        let nom: String = reste
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !nom.is_empty() {
            sortie.push(nom);
        }
        depuis = debut;
    }
    sortie
}

/// Le SQL, commentaires retirés.
///
/// Sans cela le relevé lit les instructions COMMENTÉES. C'est ainsi que
/// `audit_logs_2025_01` s'est retrouvé compté comme table muette : il n'est
/// créé nulle part, il figure dans un exemple de partitionnement commenté —
///
/// ```sql
/// -- CREATE TABLE audit_logs_2025_01 PARTITION OF audit_logs
/// ```
///
/// Septième sur-comptage de la journée, et toujours dans le même sens : la
/// dette mesurée est plus grosse que la dette réelle.
fn sans_commentaires(source: &str) -> String {
    source
        .lines()
        .map(|l| match l.find("--") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Les tables qui existent après application de toutes les migrations.
fn tables_vivantes() -> BTreeSet<String> {
    let mut creees = BTreeSet::new();
    let mut supprimees = BTreeSet::new();
    let dossier = racine().join("migrations");
    if let Ok(entrees) = fs::read_dir(&dossier) {
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.extension().and_then(|e| e.to_str()) != Some("sql") {
                continue;
            }
            let Ok(source) = fs::read_to_string(&chemin) else {
                continue;
            };
            let source = sans_commentaires(&source);
            creees.extend(nom_apres(&source, "create table"));
            supprimees.extend(nom_apres(&source, "drop table"));
        }
    }
    creees.difference(&supprimees).cloned().collect()
}

/// Tout le code susceptible de nommer une table.
fn code() -> String {
    let mut sortie = String::new();
    for racine_lecture in [racine().join("src"), racine().join(".sqlx")] {
        let mut piles = vec![racine_lecture];
        while let Some(dossier) = piles.pop() {
            let Ok(entrees) = fs::read_dir(&dossier) else {
                continue;
            };
            for entree in entrees.flatten() {
                let chemin = entree.path();
                if chemin.is_dir() {
                    piles.push(chemin);
                } else if let Ok(contenu) = fs::read_to_string(&chemin) {
                    sortie.push_str(&contenu);
                }
            }
        }
    }
    sortie.to_lowercase()
}

/// Les tables délibérément réservées, et l'issue qui les activera.
const RESERVEES: &[(&str, &str)] = &[];

fn mortes() -> Vec<String> {
    let source = code();
    let reservees: BTreeSet<&str> = RESERVEES.iter().map(|(t, _)| *t).collect();
    tables_vivantes()
        .into_iter()
        .filter(|t| !reservees.contains(t.as_str()) && !source.contains(t.as_str()))
        .collect()
}

#[test]
fn aucune_table_supplementaire_ne_devient_muette() {
    let connues: BTreeSet<&str> = MUETTES_AU_2026_09_08.iter().copied().collect();
    let nouvelles: Vec<String> = mortes()
        .into_iter()
        .filter(|t| !connues.contains(t.as_str()))
        .collect();

    assert!(
        nouvelles.is_empty(),
        "{} table(s) existent en base et ne sont lues par aucun code, et \
         elles ne figurent pas parmi les onze connues au 2026-09-08.\n\n\
         Une table que rien ne lit promet une capacité que le produit n'a \
         pas. `proxy_mandate_stats` se déclare outil de vérification des \
         procurations et rend toujours zéro ligne.\n\n\
         Branchez la table, supprimez-la par migration, ou ajoutez-la à \
         `RESERVEES` avec l'issue qui l'activera.\n\n\
         Nouvelles muettes :\n{}",
        nouvelles.len(),
        nouvelles.join("\n")
    );
}

/// Sans quoi une analyse cassée rendrait le cliquet vert faute de trouver des
/// tables à comparer.
#[test]
fn le_cliquet_voit_encore_le_schema() {
    let n = tables_vivantes().len();
    assert!(
        n > 60,
        "seulement {n} tables relevées dans les migrations : l'analyse ne lit \
         plus le schéma. Vérifiez avant de vous réjouir."
    );
}

/// Le cas nommé, pour qu'il ne se perde pas dans le décompte.
///
/// `meeting_proxy_mandates` est muette AUJOURD'HUI, et cette garde ne l'exige
/// pas branchée : la décision de la supprimer ou de la câbler demande un
/// arbitrage (#846), et une migration de suppression est irréversible en
/// production.
///
/// Ce qu'elle interdit, c'est de la faire DISPARAÎTRE de la liste des muettes
/// sans que rien ne la lise — autrement dit, de la retirer du décompte par un
/// simple renommage. Le jour où elle est branchée ou supprimée, ce test passe
/// et la liste `MUETTES_AU_2026_09_08` doit perdre son entrée.
#[test]
fn la_vue_des_procurations_reste_signalee_tant_quelle_est_muette() {
    let source = code();
    let lue = source.contains("proxy_mandate_stats") || source.contains("meeting_proxy_mandates");
    let existe = tables_vivantes().contains("meeting_proxy_mandates");

    if !existe || lue {
        // Branchée ou supprimée : retirez son entrée de MUETTES_AU_2026_09_08.
        return;
    }

    assert!(
        mortes().iter().any(|t| t == "meeting_proxy_mandates"),
        "`meeting_proxy_mandates` existe toujours, rien ne la lit, et elle a \
         pourtant disparu du décompte des muettes.\n\n\
         Sa vue `proxy_mandate_stats` se déclare « aggregate proxy mandate \
         stats for validation (max 3 mandats, max 10% quotas) » et renvoie \
         toujours zéro ligne. La règle des trois procurations EST appliquée, \
         par `domain/copropriete/procurations.rs` : c'est cette vue-ci qui \
         ment. Cf. #846."
    );
}
