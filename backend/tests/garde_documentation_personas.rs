//! Cliquet : chaque rôle réel a son document persona, avec son en-tête.
//!
//! ── Le problème que #805 nomme ────────────────────────────────────────────
//!
//! Le dépôt compte 86 documents dans `docs/`, et ils sont bons : PCMN belge,
//! RGPD, convocations, workflow de facture. Mais ils sont rangés **par sujet
//! technique ou réglementaire**, jamais par personne. Aucun ne répond à la
//! question qu'un syndic, un comptable ou un copropriétaire se pose en
//! arrivant : qu'est-ce que je peux faire, dans quel ordre, et pourquoi ?
//!
//! Cette garde ne juge pas la qualité du contenu — aucun test ne le peut —
//! mais la seule chose qu'un cliquet peut effectivement tenir : que le cadre
//! (gabarit + six documents) reste en place, que les rôles non éprouvés le
//! disent plutôt que d'inventer un parcours théorique, qu'une étape rouge
//! reste tracée jusqu'à son issue, et qu'aucune donnée d'apparence réelle ne
//! s'y glisse.
//!
//! ── D'où vient la liste des six rôles ──────────────────────────────────────
//!
//! Elle vient de la story #805 elle-même, croisée avec
//! `frontend/src/lib/auth/permissions.ts` : syndic, owner, accountant,
//! superadmin, admin, community-moderator. `population_recette` reflète le
//! nombre de comptes réellement peuplés en base au 2026-09-13 (5, 5, 2, 1, 0,
//! 0 respectivement) — deux rôles sont non éprouvés, pas absents : admin a
//! une interface (`admin-dashboard` figure au contrat `data-testid`) mais
//! aucun utilisateur réel ; community-moderator n'a ni l'un ni l'autre.
//!
//! Suivi en #805.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// (rôle, fichier, éprouvé en base au 2026-09-13).
///
/// `eprouve = false` ne veut pas dire « à ignorer » : #805 (@edge) exige que
/// ces rôles aient quand même leur document, disant explicitement qu'ils
/// n'ont pas encore été vus en usage réel.
const ROLES_ATTENDUS: [(&str, &str, bool); 6] = [
    ("syndic", "syndic.md", true),
    ("owner", "owner.md", true),
    ("accountant", "accountant.md", true),
    ("superadmin", "superadmin.md", true),
    ("admin", "admin.md", false),
    ("community-moderator", "community-moderator.md", false),
];

/// Champs d'en-tête que #805 rend obligatoires : « persona » (auteur),
/// « superviseur » et « signature_humaine » sont les trois qui disent qui
/// parle et si un humain a validé — le trait n°1 repris de `gilmry/klaar`.
const CHAMPS_REQUIS: [&str; 11] = [
    "persona",
    "role_code",
    "statut",
    "population_recette",
    "eprouve",
    "date",
    "version",
    "superviseur",
    "signature_humaine",
    "issue_cadre",
    "videos",
];

const SECTIONS_REQUISES: [&str; 5] = [
    "## Parcours nominal",
    "## Ce que ce rôle ne peut pas faire, et pourquoi",
    "## Références légales",
    "## Ce qui ne marche pas encore",
    "## Comptes de recette",
];

/// Le compte de documents persona ne peut que grandir (cf. les cliquets
/// `garde_taxonomie_bdd` et `garde-data-testid.test.ts`, qui bornent une
/// dette sans exiger sa disparition immédiate — ici c'est l'inverse : un
/// acquis qui ne doit pas se perdre).
const DOCUMENTS_AU_2026_09_13: usize = 6;

fn racine_personas() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend/ a un dossier parent")
        .join("docs/personas")
}

/// Un frontmatter YAML minimal : `cle: valeur` entre deux lignes `---`.
///
/// Pas de dépendance YAML ajoutée pour six petits fichiers déclaratifs : un
/// parseur ligne à ligne suffit et reste lisible sans bibliothèque.
fn frontmatter(source: &str) -> HashMap<String, String> {
    let mut champs = HashMap::new();
    let mut lignes = source.lines();
    if lignes.next() != Some("---") {
        return champs;
    }
    for ligne in lignes {
        if ligne == "---" {
            break;
        }
        if let Some((cle, valeur)) = ligne.split_once(':') {
            champs.insert(
                cle.trim().to_string(),
                valeur.trim().trim_matches('"').to_string(),
            );
        }
    }
    champs
}

fn lire_persona(nom_fichier: &str) -> Option<(HashMap<String, String>, String)> {
    let source = fs::read_to_string(racine_personas().join(nom_fichier)).ok()?;
    Some((frontmatter(&source), source))
}

#[test]
fn chaque_role_reel_a_son_document_persona() {
    let manquants: Vec<&str> = ROLES_ATTENDUS
        .iter()
        .copied()
        .filter(|(_, fichier, _)| !racine_personas().join(fichier).is_file())
        .map(|(role, _, _)| role)
        .collect();

    assert!(
        manquants.is_empty(),
        "Rôles réels sans document persona : {manquants:?}.\n\n\
         #805 exige un document par rôle de `permissions.ts`, y compris pour \
         les rôles non peuplés en base : ceux-là doivent dire qu'ils ne sont \
         pas éprouvés, pas rester absents."
    );
}

/// Le compte total ne baisse pas — voir le commentaire du const pour le
/// principe (repris de `garde_taxonomie_bdd::le_total_des_scenarios_charges`).
#[test]
fn le_nombre_de_documents_persona_ne_baisse_pas() {
    let total = fs::read_dir(racine_personas())
        .map(|entrees| {
            entrees
                .flatten()
                .filter(|e| {
                    let nom = e.file_name().to_string_lossy().to_string();
                    nom.ends_with(".md") && nom != "README.md" && nom != "GABARIT.md"
                })
                .count()
        })
        .unwrap_or(0);

    assert!(
        total >= DOCUMENTS_AU_2026_09_13,
        "{total} documents persona trouvés dans docs/personas/, contre \
         {DOCUMENTS_AU_2026_09_13} au 2026-09-13. Un document a disparu — \
         s'il a été légitimement fusionné avec un autre, abaissez la \
         constante en disant lequel et pourquoi."
    );
}

/// @happy — chaque document publié porte l'en-tête complet : persona, date,
/// version, superviseur, signature (même vide), et un renvoi (même vide) à
/// ses vidéos.
#[test]
fn chaque_document_persona_porte_son_en_tete_complet() {
    let mut fautes = Vec::new();

    for (role, fichier, _) in ROLES_ATTENDUS {
        let Some((champs, _)) = lire_persona(fichier) else {
            continue; // signalé par chaque_role_reel_a_son_document_persona
        };
        for requis in CHAMPS_REQUIS {
            if !champs.contains_key(requis) {
                fautes.push(format!(
                    "{fichier} ({role}) : champ `{requis}` absent de l'en-tête"
                ));
            }
        }
    }

    assert!(
        fautes.is_empty(),
        "En-tête de persona incomplet — le trait n°1 repris de `gilmry/klaar` \
         est justement qu'on sache qui parle, quand, et si un humain a \
         validé :\n{}",
        fautes.join("\n")
    );
}

#[test]
fn chaque_document_persona_a_ses_cinq_sections() {
    let mut fautes = Vec::new();

    for (role, fichier, _) in ROLES_ATTENDUS {
        let Some((_, corps)) = lire_persona(fichier) else {
            continue;
        };
        for section in SECTIONS_REQUISES {
            if !corps.contains(section) {
                fautes.push(format!(
                    "{fichier} ({role}) : section « {section} » absente"
                ));
            }
        }
    }

    assert!(
        fautes.is_empty(),
        "Sections obligatoires manquantes (cf. gabarit docs/personas/GABARIT.md) :\n{}",
        fautes.join("\n")
    );
}

/// @edge — deux des six rôles n'ont aucun compte réel en base. Le document
/// doit le dire, pas inventer un parcours théorique par-dessus.
#[test]
fn les_roles_non_peuples_en_base_se_disent_non_eprouves() {
    let mut fautes = Vec::new();

    for (role, fichier, eprouve_attendu) in ROLES_ATTENDUS {
        let Some((champs, corps)) = lire_persona(fichier) else {
            continue;
        };
        let attendu = if eprouve_attendu { "true" } else { "false" };
        if champs.get("eprouve").map(String::as_str) != Some(attendu) {
            fautes.push(format!(
                "{fichier} ({role}) : champ `eprouve` absent ou différent de `{attendu}`"
            ));
        }
        if !eprouve_attendu && !corps.to_lowercase().contains("non éprouvé") {
            fautes.push(format!(
                "{fichier} ({role}) : population_recette=0 mais le corps ne \
                 dit nulle part « non éprouvé » — #805 (@edge) interdit \
                 d'inventer un parcours théorique pour un rôle jamais vu en \
                 usage réel."
            ));
        }
    }

    assert!(fautes.is_empty(), "{}", fautes.join("\n"));
}

/// @negative — une étape rouge sans issue est une affirmation qu'on ne peut
/// pas vérifier ni suivre, exactement le défaut que #805 dénonce ailleurs
/// (matrice de conformité légale périmée en silence, #837).
#[test]
fn toute_etape_rouge_nomme_son_issue() {
    let mut fautes = Vec::new();

    for (_, fichier, _) in ROLES_ATTENDUS {
        let Some((_, corps)) = lire_persona(fichier) else {
            continue;
        };
        for (n, ligne) in corps.lines().enumerate() {
            if !ligne.contains('\u{1F534}') {
                continue; // pas une étape rouge (🔴)
            }
            let nomme_une_issue = ligne
                .split('#')
                .skip(1)
                .any(|apres| apres.chars().next().is_some_and(|c| c.is_ascii_digit()));
            if !nomme_une_issue {
                fautes.push(format!(
                    "{fichier}:{} — étape rouge sans issue tracée : {ligne}",
                    n + 1
                ));
            }
        }
    }

    assert!(
        fautes.is_empty(),
        "Étape rouge non tracée jusqu'à une issue (règle #805 : 🔴 doit \
         toujours nommer l'issue qui la ferme) :\n{}",
        fautes.join("\n")
    );
}

/// @security — les documents persona renvoient vers
/// `docs/specs/00-personas-et-seed.rst` par le NOM du persona fictif, jamais
/// en recopiant une adresse email. Une galerie de vidéos publiée est un
/// document public ; les données d'une copropriété sont nominatives.
#[test]
fn aucune_adresse_email_litterale_dans_les_documents_persona() {
    fn ressemble_a_un_email(ligne: &str) -> bool {
        for mot in ligne.split_whitespace() {
            let mot = mot.trim_matches(|c: char| {
                !(c.is_alphanumeric() || matches!(c, '@' | '.' | '-' | '_'))
            });
            let Some((local, domaine)) = mot.split_once('@') else {
                continue;
            };
            let local_ok = !local.is_empty()
                && local
                    .chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-'));
            let domaine_ok = domaine.contains('.')
                && domaine
                    .chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-'));
            if local_ok && domaine_ok {
                return true;
            }
        }
        false
    }

    let mut fautes = Vec::new();
    for (_, fichier, _) in ROLES_ATTENDUS {
        let Some((_, corps)) = lire_persona(fichier) else {
            continue;
        };
        for (n, ligne) in corps.lines().enumerate() {
            if ressemble_a_un_email(ligne) {
                fautes.push(format!(
                    "{fichier}:{} — adresse email littérale : {ligne}",
                    n + 1
                ));
            }
        }
    }

    assert!(
        fautes.is_empty(),
        "Une adresse email littérale figure dans un document persona. La \
         règle #805 est de ne jamais écrire une donnée d'apparence réelle : \
         renvoyer vers `docs/specs/00-personas-et-seed.rst` par le nom du \
         persona fictif, pas par son email.\n\n{}",
        fautes.join("\n")
    );
}
