//! Garde-fou : un DTO d'écriture qui jette les champs inconnus en silence.
//!
//! ── Le défaut ────────────────────────────────────────────────────────────
//!
//! Sans `#[serde(deny_unknown_fields)]`, serde ignore tout champ qu'il ne
//! connaît pas. Une requête d'écriture mal nommée reçoit alors **201 Created**
//! et la donnée n'est jamais stockée. Ni erreur, ni avertissement, ni trace.
//!
//! Constaté deux fois le 2026-09-04 :
//!
//! - En recette : `supplier_name`, `pcmn_account` et `amount_ht` envoyés à
//!   `/expenses` sont acceptés et perdus. Le rapport les a pris pour un
//!   « mapping incomplet » ; c'est une perte de donnée silencieuse.
//! - En production : `InvoiceForm.svelte` envoie `line_items` — description,
//!   quantité, prix unitaire et TVA de chaque ligne — que `CreateExpenseDto`
//!   n'accepte pas. Un comptable qui saisit une facture ligne par ligne perd
//!   le détail et ne garde que les totaux.
//!
//! Le second cas est le plus parlant : ce n'est pas une faute de frappe d'un
//! testeur, c'est un chemin que l'interface emprunte tous les jours.
//!
//! ── Pourquoi un cliquet plutôt qu'une correction en bloc ─────────────────
//!
//! Soixante-sept DTO d'écriture sont dans ce cas. Les corriger tous d'un coup
//! transformerait chaque champ surnuméraire encore envoyé par le frontend en
//! **400**, donc en panne. La dette se résorbe DTO par DTO, en vérifiant à
//! chaque fois ce que l'interface envoie vraiment.
//!
//! Ce cliquet interdit seulement qu'elle grossisse.

use std::fs;
use std::path::Path;

/// Le nombre de DTO d'écriture sans `deny_unknown_fields` au 2026-09-04,
/// une fois les DTO de dépense corrigés.
///
/// **Ce nombre ne doit que DIMINUER.** Le baisser quand on corrige un DTO
/// fait partie de la correction.
const DETTE_AU_2026_09_04: usize = 63;

/// Les DTO d'écriture repérés dans un fichier source.
fn dto_decriture_sans_garde(source: &str) -> Vec<String> {
    let mut trouves = Vec::new();
    let lignes: Vec<&str> = source.lines().collect();

    for (i, ligne) in lignes.iter().enumerate() {
        let l = ligne.trim();
        let Some(reste) = l.strip_prefix("pub struct ") else {
            continue;
        };
        let nom: String = reste
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !(nom.starts_with("Create") || nom.starts_with("Update")) {
            continue;
        }

        // Les attributs de la struct sont sur les lignes qui précèdent
        // immédiatement, jusqu'au premier blanc ou à la fin d'un commentaire.
        let debut = i.saturating_sub(8);
        let entete = lignes[debut..i].join("\n");
        if !entete.contains("deny_unknown_fields") {
            trouves.push(nom);
        }
    }
    trouves
}

#[test]
fn la_dette_des_champs_ignores_ne_grossit_pas() {
    let repertoire = Path::new("src/application/dto");
    assert!(
        repertoire.is_dir(),
        "le répertoire des DTO a changé de place : {}",
        repertoire.display()
    );

    let mut sans_garde = Vec::new();
    for entree in fs::read_dir(repertoire).expect("lecture du répertoire des DTO") {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&chemin).expect("lecture du fichier");
        for nom in dto_decriture_sans_garde(&source) {
            sans_garde.push(format!(
                "{}::{nom}",
                chemin.file_name().unwrap().to_string_lossy()
            ));
        }
    }

    sans_garde.sort();
    assert!(
        sans_garde.len() <= DETTE_AU_2026_09_04,
        "La dette des champs ignorés a GROSSI : {} DTO d'écriture sans \
         `deny_unknown_fields`, contre {} au 2026-09-04.\n\n\
         Un DTO sans cette annotation accepte n'importe quel champ mal nommé \
         et le jette en silence : la requête rend 201 et la donnée n'existe \
         pas. Ajoutez `#[serde(deny_unknown_fields)]` au nouveau DTO.\n\n\
         Liste :\n{}",
        sans_garde.len(),
        DETTE_AU_2026_09_04,
        sans_garde.join("\n")
    );
}

/// Le recensement trouve-t-il encore quelque chose ?
///
/// Sans ce contrôle, une refonte du répertoire des DTO rendrait le cliquet
/// silencieusement vert en ne trouvant plus rien à compter.
#[test]
fn le_recensement_trouve_bien_des_dto() {
    let source = fs::read_to_string("src/application/dto/expense_dto.rs")
        .expect("expense_dto.rs doit exister");
    let noms: Vec<String> = source
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub struct ").map(str::to_string))
        .collect();
    assert!(
        noms.iter().any(|n| n.starts_with("CreateExpenseDto")),
        "le recensement ne reconnaît plus les DTO : {noms:?}"
    );
}
