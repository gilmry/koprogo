//! Garde-fou : aucune issue du périmètre 0.1.0 n'échappe au WBS.
//!
//! ── Ce qui a été trouvé ────────────────────────────────────────────────────
//!
//! Le 2026-09-06, le WBS se déclarait « seule vérité courante » du périmètre.
//! Il citait 75 des 96 issues étiquetées `release:0.1.0`. **Vingt et une
//! n'étaient nulle part** : l'épopée #556 et ses stories de modularité par ACP,
//! sept issues d'infrastructure, la dette d'IaC, et #780 — le cycle de vie
//! d'une AG, `priority:critical`, ouvert le jour même.
//!
//! Une issue du périmètre absente du plan est une issue que personne ne
//! planifie. Elle n'est pas arbitrée, elle n'est pas ordonnancée, et elle
//! réapparaît au moment de poser le tag.
//!
//! ── Pourquoi un test plutôt qu'une relecture ───────────────────────────────
//!
//! Le WBS a été relu plusieurs fois ce jour-là sans que l'écart apparaisse.
//! C'est normal : personne ne recompte 96 numéros à la main. La divergence
//! s'est installée exactement comme celle de `docs/api/openapi.json`, resté
//! cinq jours en arrière du code sans que rien ne le signale.
//!
//! ── Ce que ce test fait, et ne fait pas ────────────────────────────────────
//!
//! Il ne sait pas classer une issue à la place d'un humain. Il vérifie une
//! seule chose, hors ligne : **tout numéro d'issue nommé par l'inventaire du
//! WBS est bien cité dans le corps du document**, et l'inventaire lui-même est
//! engendré par `scripts/inventaire-wbs.py`, qui échoue si une issue du
//! périmètre n'a pas de track.
//!
//! Le partage des rôles est volontaire : le script parle à GitHub et refuse
//! l'oubli au moment de la génération ; ce test n'a pas besoin du réseau et
//! refuse qu'on vide l'inventaire sans le dire. Un test qui exigerait le réseau
//! serait désactivé au premier passage hors ligne.
//!
//! Voir le WBS, section « Inventaire complet du périmètre 0.1.0 ».

use std::fs;
use std::path::{Path, PathBuf};

fn wbs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/WBS_v0_1_0.md")
}

/// Les numéros d'issue cités dans une portion de texte.
fn issues_citees(texte: &str) -> Vec<u32> {
    let mut trouvees = Vec::new();
    let octets = texte.as_bytes();
    for (i, c) in octets.iter().enumerate() {
        if *c != b'#' {
            continue;
        }
        let chiffres: String = texte[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if chiffres.len() >= 3 {
            if let Ok(n) = chiffres.parse() {
                trouvees.push(n);
            }
        }
    }
    trouvees.sort_unstable();
    trouvees.dedup();
    trouvees
}

fn corps() -> String {
    fs::read_to_string(wbs()).expect("le WBS est lisible")
}

#[test]
fn linventaire_du_perimetre_est_present_et_peuple() {
    let texte = corps();
    let debut = texte
        .find("## Inventaire complet du périmètre 0.1.0")
        .expect(
            "la section « Inventaire complet du périmètre 0.1.0 » a disparu du WBS.\n\n\
             C'est elle qui garantit qu'aucune issue étiquetée `release:0.1.0` n'est \
             hors plan. La retirer rend le périmètre invérifiable.\n\n\
             Régénérez-la : python3 scripts/inventaire-wbs.py",
        );
    let fin = texte[debut..]
        .find("\n## ")
        .map(|f| debut + f)
        .unwrap_or(texte.len());
    let section = &texte[debut..fin];

    // Le nombre ANNONCÉ par l'en-tête, et le nombre RÉELLEMENT tabulé.
    //
    // Un seuil fixe aurait été le mauvais instrument : le périmètre a
    // légitimement baissé de 106 à 88 le 2026-09-06, et un cliquet qui
    // interdit la baisse punit le travail au lieu de l'écart. Ce qu'il faut
    // interdire, c'est la DIVERGENCE entre ce que le document annonce et ce
    // qu'il montre — c'est-à-dire l'édition à la main.
    let annonce: usize = texte[debut..]
        .split_once("issues ouvertes** portent")
        .and_then(|(avant, _)| {
            // `avant` se termine par « …**88 » : le fragment cherché est le
            // DERNIER, donc le premier que `rsplit` rend.
            avant
                .rsplit("**")
                .next()
                .and_then(|s| s.trim().parse().ok())
        })
        .expect("l'en-tête de l'inventaire n'annonce plus un nombre d'issues");

    let lignes = section
        .lines()
        .filter(|l| l.trim_start().starts_with("| #"))
        .count();

    assert_eq!(
        lignes, annonce,
        "L'inventaire annonce {annonce} issues et en tabule {lignes}.\n\n\
         L'écart signifie qu'on a édité le document à la main plutôt que de le \
         régénérer. C'est ainsi que 21 issues du périmètre se sont retrouvées \
         hors plan, dont #780 en `priority:critical`.\n\n\
         Régénérez : python3 scripts/inventaire-wbs.py"
    );

    let citees = issues_citees(section);
    assert!(
        citees.len() >= 40,
        "l'inventaire ne cite plus que {} issues : le périmètre 0.1.0 ne peut \
         pas avoir fondu à ce point sans que le reste du WBS soit réécrit",
        citees.len()
    );

    // Chaque track doit exister et porter au moins une issue.
    for track in [
        "Track R —",
        "Track U —",
        "Track D —",
        "Track M —",
        "Track S —",
        "Track T —",
        "Track K —",
        "Track F —",
        "Track G —",
    ] {
        assert!(
            section.contains(track),
            "le track « {track} » a disparu de l'inventaire : ses issues ne sont \
             plus rattachées à rien"
        );
    }
}

/// Le générateur doit rester présent et exécutable par un humain.
///
/// Sans lui, l'inventaire redevient une liste tenue à la main, c'est-à-dire une
/// liste qui diverge.
#[test]
fn le_generateur_de_linventaire_existe() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/inventaire-wbs.py");
    assert!(
        script.exists(),
        "`scripts/inventaire-wbs.py` a disparu. C'est lui qui interroge GitHub et \
         refuse de produire un inventaire dont une issue du périmètre n'aurait pas \
         de track."
    );
    let source = fs::read_to_string(&script).expect("script lisible");
    assert!(
        source.contains("non classées"),
        "le générateur ne refuse plus les issues sans track : il produirait un \
         inventaire incomplet sans le dire, ce qui est précisément le défaut qu'il \
         existe pour empêcher"
    );
}
