//! Garde : la matrice de conformité légale ne peut plus se périmer en silence.
//!
//! ── Le constat qui a motivé cette garde ───────────────────────────────────
//!
//! `specs/bdd-non-ecrits/legal_compliance.feature` se présente comme « le
//! POINT CENTRAL de suivi de conformité juridique » : quarante et un
//! scénarios, chacun rattaché à un article du Code civil belge ou du RGPD.
//!
//! Douze portaient `@manquant` — « Exigence NON implémentée (bloquant pour
//! production) ». **Onze étaient faux** (#837). Neuf règles étaient
//! implémentées, une était partielle, et une ligne décrivait une règle qui
//! n'existe pas en droit belge.
//!
//! La matrice a pu se tromper six mois durant pour une seule raison : elle
//! n'était pas exécutée. Tous ses scénarios sont `@wip`, et son fichier
//! n'était chargé par aucun harnais (#838). Ni vert, ni rouge. Une
//! déclaration que personne ne pouvait démentir.
//!
//! ── Pourquoi une erreur « dans le bon sens » compte quand même ────────────
//!
//! La matrice était PESSIMISTE : elle sous-estimait la conformité, personne
//! n'était trompé sur ses droits. Deux raisons de ne pas s'en contenter.
//!
//! Elle faisait chercher du travail déjà fait — onze chantiers ouverts qui
//! n'existaient plus.
//!
//! Et elle décrédibilisait ses lignes exactes. C'est le coût réel : une
//! matrice dont on découvre qu'elle se trompe cesse d'être consultée, et ce
//! sont alors les VRAIES lacunes qui deviennent invisibles. Il en reste une,
//! le plafond de trois ans du mandat de syndic (Art. 3.89).
//!
//! ── Ce que cette garde vérifie ────────────────────────────────────────────
//!
//! Elle ne vérifie pas le droit belge : aucun test ne peut faire cela. Elle
//! vérifie la seule chose qui a effectivement dérivé — **la cohérence entre
//! ce que la matrice déclare et ce que le dépôt contient**.
//!
//! 1. Un scénario `@conforme` NOMME le module qui le satisfait, et ce
//!    fichier existe.
//! 2. Un scénario `@manquant` ne nomme AUCUN module comme le satisfaisant.
//!    C'est la contradiction exacte qui a duré six mois.
//! 3. L'en-tête annonce un score, et ce score correspond au décompte réel
//!    des étiquettes. C'est le chiffre qui s'était périmé (« 31/37 » pour
//!    41 scénarios).
//! 4. Le fichier est toujours là où on le croit, avec ses quarante et un
//!    scénarios. Sans quoi la garde se rendrait verte en ne trouvant rien.
//!
//! Suivi en #837.

use std::fs;
use std::path::{Path, PathBuf};

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn matrice() -> String {
    let chemin = racine().join("specs/bdd-non-ecrits/legal_compliance.feature");
    fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("matrice de conformité illisible en {chemin:?} : {e}"))
}

/// Un scénario de la matrice : ses étiquettes, son titre, ses commentaires.
struct Scenario {
    titre: String,
    etiquettes: Vec<String>,
    commentaires: Vec<String>,
}

fn scenarios(source: &str) -> Vec<Scenario> {
    let lignes: Vec<&str> = source.lines().collect();
    let mut sortie = Vec::new();

    for (i, ligne) in lignes.iter().enumerate() {
        let nu = ligne.trim();
        if !nu.starts_with("Scenario:") && !nu.starts_with("Scénario:") {
            continue;
        }
        let titre = nu
            .split_once(':')
            .map(|(_, t)| t.trim().to_string())
            .unwrap_or_default();

        // Les étiquettes sont sur la ligne juste au-dessus.
        let etiquettes = i
            .checked_sub(1)
            .map(|j| {
                lignes[j]
                    .split_whitespace()
                    .filter(|m| m.starts_with('@'))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Les commentaires vont du titre au premier pas Gherkin.
        let mut commentaires = Vec::new();
        for suite in lignes.iter().skip(i + 1) {
            let nu = suite.trim();
            if nu.starts_with('#') {
                commentaires.push(nu.to_string());
            } else if !nu.is_empty() {
                break;
            }
        }

        sortie.push(Scenario {
            titre,
            etiquettes,
            commentaires,
        });
    }
    sortie
}

/// Le fichier source nommé par un scénario, s'il y en a un.
///
/// Deux conventions cohabitent dans la matrice, et les deux comptent :
/// « # Implémenté : <chemin> — … » pour les lignes rétablies par #837, et
/// « # Code : <fichier>:<lignes> » pour l'audit d'origine. Un
/// « # Code : NON IMPLÉMENTÉ » ne nomme évidemment rien.
fn module_declare(s: &Scenario) -> Option<String> {
    for c in &s.commentaires {
        let apres = c
            .split_once("Implémenté :")
            .or_else(|| c.split_once("Code   :"))
            .or_else(|| c.split_once("Code :"))
            .map(|(_, a)| a)?;
        if apres.contains("NON IMPLÉMENTÉ") {
            continue;
        }
        for mot in apres.split_whitespace() {
            // « convocation.rs:23-30 » → « convocation.rs »
            let mot = mot.trim_end_matches(&[',', ';', ')'][..]);
            let mot = mot.split(':').next().unwrap_or(mot);
            if mot.ends_with(".rs") || mot.ends_with(".astro") || mot.ends_with(".svelte") {
                return Some(mot.to_string());
            }
        }
    }
    None
}

/// Le fichier nommé existe-t-il quelque part dans le dépôt ?
///
/// La matrice le désigne tantôt par un chemin (`domain/copropriete/x.rs`),
/// tantôt par son seul nom (`convocation.rs`). Les deux doivent être
/// acceptés, sans quoi la garde punirait une convention plutôt qu'un défaut.
fn fichier_existe(chemin: &str) -> bool {
    let directs = [
        racine().join("src").join(chemin),
        racine().join(chemin),
        racine()
            .parent()
            .map(|p| p.join(chemin))
            .unwrap_or_default(),
    ];
    if directs.iter().any(|c| Path::new(c).is_file()) {
        return true;
    }

    // Nom seul : on cherche récursivement, côté backend puis frontend.
    let nom = Path::new(chemin)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| chemin.to_string());
    let depot = racine().parent().map(PathBuf::from);
    let racines = [
        Some(racine().join("src")),
        depot.map(|d| d.join("frontend/src")),
    ];
    racines
        .into_iter()
        .flatten()
        .any(|r| contient_le_fichier(&r, &nom))
}

fn contient_le_fichier(dossier: &Path, nom: &str) -> bool {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return false;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            if contient_le_fichier(&chemin, nom) {
                return true;
            }
        } else if chemin.file_name().is_some_and(|n| n == nom) {
            return true;
        }
    }
    false
}

#[test]
fn la_matrice_est_toujours_la_ou_on_la_croit() {
    let s = matrice();
    let n = scenarios(&s).len();
    assert!(
        n >= 40,
        "seulement {n} scénarios trouvés dans la matrice de conformité. \
         Le fichier a bougé, ou son format a changé. Une garde qui ne trouve \
         plus rien à vérifier devient verte : vérifiez avant de vous réjouir."
    );
}

#[test]
fn toute_regle_declaree_conforme_nomme_un_module_qui_existe() {
    let s = matrice();
    let mut fautes = Vec::new();

    for sc in scenarios(&s) {
        if !sc.etiquettes.iter().any(|e| e == "@conforme") {
            continue;
        }
        match module_declare(&sc) {
            None => {
                fautes.push(format!(
                    "« {} » est @conforme mais ne nomme aucun fichier source \
                     (attendu : « # Implémenté : <chemin> — … » ou « # Code : <fichier> »)",
                    sc.titre
                ));
            }
            Some(chemin) => {
                if !fichier_existe(&chemin) {
                    fautes.push(format!(
                        "« {} » se déclare satisfaite par `{chemin}`, qui n'existe pas",
                        sc.titre
                    ));
                }
            }
        }
    }

    assert!(
        fautes.is_empty(),
        "La matrice de conformité nomme des modules introuvables.\n\n\
         Une ligne « conforme » qui pointe vers un fichier disparu est pire \
         qu'une ligne vide : elle affirme une conformité que plus rien ne \
         porte.\n\n{}",
        fautes.join("\n")
    );
}

#[test]
fn aucune_regle_declaree_manquante_ne_nomme_le_module_qui_la_satisfait() {
    let s = matrice();
    let mut fautes = Vec::new();

    for sc in scenarios(&s) {
        if !sc.etiquettes.iter().any(|e| e == "@manquant") {
            continue;
        }
        if let Some(chemin) = module_declare(&sc) {
            fautes.push(format!(
                "« {} » est marquée @manquant tout en se déclarant implémentée \
                 par `{chemin}`",
                sc.titre
            ));
        }
    }

    assert!(
        fautes.is_empty(),
        "La matrice se contredit : une règle ne peut pas être « NON \
         implémentée (bloquant pour production) » et nommer le module qui la \
         satisfait.\n\n\
         C'est exactement la contradiction qui a duré six mois (#837) : onze \
         lignes sur douze déclaraient manquant ce qui était écrit.\n\n\
         Retirez `@manquant`, ou retirez la ligne « Implémenté : ».\n\n{}",
        fautes.join("\n")
    );
}

#[test]
fn le_score_annonce_en_tete_correspond_au_decompte_reel() {
    let s = matrice();
    let liste = scenarios(&s);

    let compte = |etiquette: &str| {
        liste
            .iter()
            .filter(|sc| sc.etiquettes.iter().any(|e| e == etiquette))
            .count()
    };
    let (conformes, partiels, manquants, total) = (
        compte("@conforme"),
        compte("@partiel"),
        compte("@manquant"),
        liste.len(),
    );

    let attendu = format!(
        "# Score conformité : {conformes} conformes / {partiels} partiels / \
         {manquants} manquant, sur {total} scénarios"
    );
    // Normalisation : le fichier peut couper la ligne, on compare sans
    // espaces multiples.
    let normalise = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    let attendu = normalise(&attendu);

    let trouve = s
        .lines()
        .find(|l| l.contains("Score conformité"))
        .map(normalise)
        .unwrap_or_default();

    assert_eq!(
        trouve, attendu,
        "\n\nL'en-tête de la matrice annonce un score qui ne correspond plus \
         au décompte de ses propres étiquettes.\n\n\
         C'est le chiffre qui s'était périmé : « 31/37 » pour 41 scénarios, \
         pendant six mois (#837). Un score écrit à la main se périme ; \
         celui-ci est désormais vérifié.\n\n\
         Décompte réel : {conformes} conformes, {partiels} partiels, \
         {manquants} manquant, sur {total} scénarios.\n"
    );
}
