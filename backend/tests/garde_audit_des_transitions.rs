//! Cliquet : le nom du geste et le nom de la ligne d'audit doivent s'accorder
//! (#881).
//!
//! ── Le défaut que ce cliquet borne ────────────────────────────────────────
//!
//! `expense_handlers.rs` porte cinq transitions — payer, marquer en retard,
//! annuler, réactiver, dé-payer — et quatre d'entre elles journalisaient
//! toutes `ExpenseMarkedPaid`, l'évènement de la première, copié-collé.
//! `unpay_expense` DÉFAIT un paiement ; le registre affirmait qu'un paiement
//! venait d'avoir lieu.
//!
//! C'est le même défaut que #780 a corrigé pour les assemblées
//! (`MeetingCancelled` vs `MeetingCompleted`, voir `infrastructure/audit.rs`) :
//! le dispositif tourne, produit une ligne, et la ligne ne dit pas le vrai.
//! Sur une comptabilité d'ACP, ce registre est ce qu'on produit quand un
//! copropriétaire conteste une écriture (Art. 3.89 § 5, 7°) — une ligne
//! fausse n'est pas incomplète, elle est fausse.
//!
//! ── Ce que ce cliquet vérifie, et ce qu'il ne vérifie pas ─────────────────
//!
//! Il lit le SOURCE de `expense_handlers.rs`, pas son exécution : comme
//! `garde_identite_sans_decision` et `garde_controles_dormants`. Ces routes ne
//! persistent pas leur ligne d'audit (juste `AuditLogEntry::log()` vers
//! stdout) : le seul témoin disponible avant l'exécution est la ligne de code
//! qui l'écrirait, et c'est exactement elle que le défaut d'origine avait
//! fausse. L'assertion porte sur cette ligne, pas sur le code HTTP retourné —
//! le défaut d'origine rendait un `200 OK` parfaitement correct tout en
//! écrivant un mensonge.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn source_expense_handlers() -> String {
    let chemin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/infrastructure/web/handlers/expense_handlers.rs");
    fs::read_to_string(&chemin).expect("expense_handlers.rs lisible")
}

/// Le corps d'un gestionnaire `pub async fn <nom>`, du premier caractère
/// après la signature jusqu'au `pub async fn` suivant (ou la fin du fichier).
fn corps_du_gestionnaire(source: &str, nom: &str) -> String {
    let morceaux: Vec<&str> = source.split("pub async fn ").collect();
    for morceau in morceaux.iter().skip(1) {
        let nom_trouve = morceau.split(['(', '<']).next().unwrap_or("").trim();
        if nom_trouve == nom {
            return (*morceau).to_string();
        }
    }
    panic!("gestionnaire {nom} introuvable dans expense_handlers.rs");
}

/// Toutes les occurrences `AuditEventType::<Variante>` dans un extrait.
fn evenements_journalises(corps: &str) -> Vec<String> {
    let motif = "AuditEventType::";
    let mut trouves = Vec::new();
    let mut depuis = 0usize;
    while let Some(i) = corps[depuis..].find(motif) {
        let abs = depuis + i + motif.len();
        let reste = &corps[abs..];
        let fin = reste
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(reste.len());
        trouves.push(reste[..fin].to_string());
        depuis = abs + fin;
    }
    trouves
}

/// Le geste que chaque gestionnaire affirme, et le SEUL évènement qu'il a le
/// droit de journaliser.
///
/// Ajouter une sixième transition dont le nom n'apparaît pas ici fait échouer
/// `edge_toute_transition_est_enregistree_ici` : c'est le point du cliquet,
/// pas un oubli à corriger en silence. Nommez le geste, choisissez ou créez la
/// variante d'`AuditEventType` qui lui correspond, ajoutez la ligne — AVANT de
/// journaliser quoi que ce soit.
const CORRESPONDANCES: &[(&str, &str)] = &[
    ("mark_expense_paid", "ExpenseMarkedPaid"),
    ("mark_expense_overdue", "ExpenseMarkedOverdue"),
    ("cancel_expense", "ExpenseCancelled"),
    ("reactivate_expense", "ExpenseReactivated"),
    ("unpay_expense", "ExpenseUnpaid"),
];

#[test]
fn happy_annuler_une_depense_journalise_expensecancelled() {
    let source = source_expense_handlers();
    let corps = corps_du_gestionnaire(&source, "cancel_expense");
    let evenements = evenements_journalises(&corps);
    assert_eq!(
        evenements,
        vec!["ExpenseCancelled".to_string()],
        "`cancel_expense` doit journaliser exactement ExpenseCancelled, pas {evenements:?}"
    );
}

#[test]
fn negative_aucune_transition_ne_journalise_le_geste_dune_autre() {
    let source = source_expense_handlers();
    let mut fautifs = Vec::new();
    for (nom, attendu) in CORRESPONDANCES {
        let corps = corps_du_gestionnaire(&source, nom);
        for trouve in evenements_journalises(&corps) {
            if trouve.as_str() != *attendu {
                fautifs.push(format!(
                    "{nom} journalise AuditEventType::{trouve}, alors que son nom affirme {attendu}"
                ));
            }
        }
    }
    assert!(
        fautifs.is_empty(),
        "Le registre d'audit ment sur le geste qui a eu lieu :\n  {}\n\n\
         Un copropriétaire qui conteste une écriture lit ce registre pour ce \
         qu'il affirme, pas pour ce que le code a réellement fait (#881).",
        fautifs.join("\n  ")
    );
}

#[test]
fn edge_un_refus_du_domaine_necrit_aucune_ligne_de_succes() {
    let source = source_expense_handlers();
    // `mark_expense_paid` journalise aussi son échec (comportement antérieur
    // à #881, non touché ici) : volontairement absent de cette liste, qui ne
    // porte que sur les quatre transitions qui n'ont jamais journalisé leur
    // branche `Err`.
    for nom in [
        "mark_expense_overdue",
        "cancel_expense",
        "reactivate_expense",
        "unpay_expense",
    ] {
        let corps = corps_du_gestionnaire(&source, nom);
        let branche_err = match corps.find("Err(err) =>") {
            Some(i) => &corps[i..],
            None => panic!("{nom} : branche Err introuvable"),
        };
        assert!(
            !branche_err.contains("AuditLogEntry::new"),
            "{nom} journalise une ligne d'audit dans sa branche Err : une \
             transition refusée par le domaine (ex. dépense déjà annulée) ne \
             doit écrire AUCUNE ligne de succès."
        );
    }
}

#[test]
fn edge_toute_transition_est_enregistree_ici() {
    let source = source_expense_handlers();

    // Aveuglement dans un sens : chaque entrée de CORRESPONDANCES doit
    // encore désigner un gestionnaire réel.
    for (nom, _) in CORRESPONDANCES {
        let signature = format!("pub async fn {nom}(");
        assert!(
            source.contains(signature.as_str()),
            "{nom} n'existe plus dans expense_handlers.rs : retirez son \
             entrée de CORRESPONDANCES, elle ne justifie plus rien."
        );
    }

    // Aveuglement dans l'autre sens : tout gestionnaire de TRANSITION du
    // fichier doit être couvert. `create_expense`, `get_expense` et les deux
    // listes ne sont pas des transitions — elles n'apparaissent dans aucune
    // AuditEventType du cycle de vie « paiement ».
    let non_transitions: BTreeSet<&str> = [
        "create_expense",
        "get_expense",
        "list_expenses",
        "list_expenses_by_building",
    ]
    .into_iter()
    .collect();

    let morceaux: Vec<&str> = source.split("pub async fn ").collect();
    let gestionnaires_expense: Vec<String> = morceaux
        .iter()
        .skip(1)
        .map(|m| m.split(['(', '<']).next().unwrap_or("").trim().to_string())
        .filter(|n| n.contains("expense") && !non_transitions.contains(n.as_str()))
        .collect();

    let connus: BTreeSet<&str> = CORRESPONDANCES.iter().map(|(n, _)| *n).collect();
    let non_enregistres: Vec<&String> = gestionnaires_expense
        .iter()
        .filter(|n| !connus.contains(n.as_str()))
        .collect();

    assert!(
        non_enregistres.is_empty(),
        "Ces gestionnaires de dépense ne sont dans aucune des deux listes de \
         ce fichier (CORRESPONDANCES ou non_transitions) : {non_enregistres:?}.\n\n\
         Une sixième transition doit nommer le geste qu'elle fait et la \
         variante d'AuditEventType qui lui correspond avant de journaliser \
         quoi que ce soit — c'est ce que ce cliquet impose."
    );
}

#[test]
fn security_le_registre_nexpose_aucune_reecriture_retroactive() {
    let chemin =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/application/ports/audit_log_repository.rs");
    let source = fs::read_to_string(&chemin).expect("port audit_log_repository lisible");
    assert!(
        !source.contains("fn update"),
        "Le port AuditLogRepository expose une méthode de mise à jour : une \
         ligne d'audit déjà posée pourrait être réécrite après coup. Corriger \
         le TYPE d'un évènement (#881) ne doit jamais ouvrir ce chemin — une \
         ligne fausse se corrige par une ligne NOUVELLE, jamais par la \
         modification de l'ancienne."
    );
}
