//! Cliquet : la part des tests qui ne déclarent pas leur catégorie ne grossit
//! pas.
//!
//! ── Le constat de #427, enfin mesuré ──────────────────────────────────────
//!
//! L'issue dit : « **921 scénarios BDD annoncés ≠ 921 chemins testés.** La
//! majorité sont des variations *happy path*. 1 191 `#[test]` ≠ 1 191 cas
//! d'erreur testés. **Le ratio happy/edge/security/negative n'est pas
//! mesuré.** »
//!
//! Il l'est désormais, au 2026-09-08 :
//!
//! ```text
//! Rust        2510 tests   1807 sans catégorie   72 %
//! Playwright   420 tests    354 sans étiquette   84 %
//! BDD         1109 scénarios 919 sans étiquette   82 %
//! ────────────────────────────────────────────────────
//! total       4039          3080                 76 %
//! ```
//!
//! **Le chiffre Rust a demandé trois mesures.** La première annonçait 2005
//! tests et 1285 sans catégorie : son expression régulière exigeait `fn`
//! immédiatement après `#[test]`, et manquait donc tout test portant un
//! `#[serial]` ou un commentaire de documentation entre les deux. La seconde,
//! plus permissive, en trouvait 1807. La troisième, stricte mais complète —
//! l'attribut de test, puis seulement des attributs et du blanc jusqu'à `fn` —
//! confirme 1804.
//!
//! C'est le neuvième écart de mesure de la journée, et le second dans le sens
//! qui MINIMISE la dette. La leçon vaut dans les deux sens : une mesure faite
//! à la main ne vaut pas une mesure exécutée.
//!
//! **Ce chiffre ne dit pas que 72 % des tests sont des chemins nominaux.** Il
//! dit qu'on ne peut pas savoir : la grande majorité ne déclare rien. C'est
//! exactement la maladie que l'issue décrit — « la métrique cache la
//! maladie ». Un décompte de tests sans taxonomie rassure sans informer.
//!
//! ── La convention, relevée et non inventée ────────────────────────────────
//!
//! Le dépôt en a déjà une, appliquée à 720 tests Rust :
//!
//! ```text
//! happy_     229   le chemin nominal
//! negative_  189   l'entrée invalide, l'erreur attendue
//! edge_      154   la borne, le cas limite
//! security_  148   le refus d'accès, la fuite
//! ```
//!
//! Un test nommé `happy_validation_error_maps_to_400` déclare ce qu'il
//! éprouve. Un test nommé `test_create_building` ne déclare rien.
//!
//! ── Ce que ce cliquet garde, et ce qu'il ne garde pas ─────────────────────
//!
//! Il **n'exige pas** de renommer 1285 tests : ce serait une correction de
//! masse invérifiable, et un préfixe posé au hasard vaut moins que pas de
//! préfixe puisqu'il fera croire à une taxonomie.
//!
//! Il empêche le 1286ᵉ. Tout nouveau test déclare sa catégorie, et la dette se
//! résorbe au fil des fichiers qu'on touche.
//!
//! **Le second contrôle est celui qui compte.** Un cliquet sur le seul nombre
//! de tests sans catégorie se satisferait d'une SUPPRESSION de tests. Le total
//! doit donc rester au moins à son niveau : on ne descend la dette qu'en
//! nommant, jamais en effaçant.
//!
//! Suivi en #427.

use std::fs;
use std::path::{Path, PathBuf};

/// Tests Rust ne déclarant pas leur catégorie. **Ne doit que BAISSER.**
/// Le chiffre est celui du DÉTECTEUR, pas de mon estimation.
///
/// Mon relevé Python en trouvait 1777, celui-ci 1779 : les deux analysent le
/// même code avec une tolérance différente entre l'attribut `#[test]` et le
/// `fn` qui suit. Quand la mesure manuelle et la mesure exécutée divergent,
/// c'est la seconde qui a raison — dixième fois de la journée.
const SANS_CATEGORIE_AU_2026_09_08: usize = 1779;

/// Total des tests Rust. **Ne doit pas BAISSER** — sans quoi on solderait la
/// dette en supprimant des tests.
const TOTAL_AU_2026_09_08: usize = 2510;

const CATEGORIES: [&str; 4] = ["happy_", "negative_", "edge_", "security_"];

/// La cinquième catégorie : les cliquets.
///
/// ── Pourquoi elle existe, et pourquoi ce n'est pas une échappatoire ──────
///
/// Cette garde a d'abord compté comme « sans catégorie » les 29 tests des
/// fichiers `tests/garde_*.rs`. Elle a donc rejeté mes propres ajouts, ce qui
/// est le comportement attendu d'un cliquet — mais le diagnostic était faux.
///
/// Ces tests suivent une convention ANTÉRIEURE, la phrase descriptive :
/// `la_dette_de_lecture_imbriquee_ne_grossit_pas`,
/// `le_recensement_trouve_bien_des_routes`. Elle est délibérée, et les
/// quatre catégories ne leur conviennent pas : **un cliquet n'est pas un
/// chemin nominal ni un cas limite DU PRODUIT, c'est un invariant DU DÉPÔT.**
/// Leur coller `edge_` les nommerait mal.
///
/// La garde reconnaît donc ce qu'ils sont, au lieu de le compter comme un
/// manque. Ce n'est pas une exemption ouverte : elle ne vaut que pour les
/// fichiers `tests/garde_*.rs`, et elle porte sur 29 tests contre 1777
/// ailleurs. Un test de produit ne peut pas s'y cacher.
fn est_un_cliquet(fichier: &str) -> bool {
    fichier.starts_with("tests/garde_") || fichier.contains("/garde_")
}

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fichiers_rust(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            fichiers_rust(&chemin, sortie);
        } else if chemin.extension().and_then(|e| e.to_str()) == Some("rs") {
            sortie.push(chemin);
        }
    }
}

/// Le nom de chaque fonction annotée `#[test]`, sous toutes ses formes.
fn noms_de_tests() -> Vec<(String, String)> {
    let mut fichiers = Vec::new();
    fichiers_rust(&racine().join("src"), &mut fichiers);
    fichiers_rust(&racine().join("tests"), &mut fichiers);
    fichiers.sort();

    let mut sortie = Vec::new();
    for chemin in fichiers {
        let Ok(source) = fs::read_to_string(&chemin) else {
            continue;
        };
        let fichier = chemin
            .strip_prefix(racine())
            .unwrap_or(&chemin)
            .to_string_lossy()
            .to_string();

        for (pos, _) in source.match_indices("#[") {
            let reste = &source[pos..];
            // `#[test]`, `#[tokio::test]`, `#[actix_web::test]`…
            let entete: String = reste.chars().take(40).collect();
            if !entete.contains("test]") {
                continue;
            }
            let Some(pos_fn) = reste.find("fn ") else {
                continue;
            };
            // Le `fn` doit suivre de près l'attribut : sinon on a sauté par
            // dessus une fonction non annotée.
            if pos_fn > 120 {
                continue;
            }
            let apres = &reste[pos_fn + 3..];
            let nom: String = apres
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !nom.is_empty() {
                sortie.push((nom, fichier.clone()));
            }
        }
    }
    sortie
}

fn sans_categorie() -> Vec<String> {
    noms_de_tests()
        .into_iter()
        .filter(|(nom, fichier)| {
            !CATEGORIES.iter().any(|c| nom.starts_with(c)) && !est_un_cliquet(fichier)
        })
        .map(|(nom, fichier)| format!("{fichier} :: {nom}"))
        .collect()
}

/// Les tests de cliquet, comptés à part pour rester visibles.
fn cliquets() -> usize {
    noms_de_tests()
        .into_iter()
        .filter(|(nom, fichier)| {
            !CATEGORIES.iter().any(|c| nom.starts_with(c)) && est_un_cliquet(fichier)
        })
        .count()
}

#[test]
fn aucun_test_supplementaire_ne_tait_sa_categorie() {
    let liste = sans_categorie();
    let n = liste.len();

    assert!(
        n <= SANS_CATEGORIE_AU_2026_09_08,
        "{n} tests Rust ne déclarent pas leur catégorie, contre \
         {SANS_CATEGORIE_AU_2026_09_08} au 2026-09-08.\n\n\
         Un décompte de tests sans taxonomie rassure sans informer : il ne dit \
         pas si les chemins d'erreur, les bornes et les refus d'accès sont \
         éprouvés, ou si tout est nominal (#427).\n\n\
         Préfixez le nom du test : `happy_`, `negative_`, `edge_`, \
         `security_`.\n\n\
         ({} tests de cliquet sont comptés à part : ils suivent la convention \
         de la phrase descriptive, et un invariant du dépôt n'est ni un chemin \
         nominal ni un cas limite du produit.)\n\n\
         Les cinq derniers relevés :\n{}",
        cliquets(),
        liste
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Le contrôle qui empêche de solder la dette en supprimant des tests.
///
/// Sans lui, le cliquet précédent se satisferait d'une suppression : moins de
/// tests, donc moins de tests sans catégorie. On ne descend la dette qu'en
/// nommant.
#[test]
fn le_total_des_tests_ne_baisse_pas() {
    let total = noms_de_tests().len();
    assert!(
        total >= TOTAL_AU_2026_09_08,
        "{total} tests Rust relevés, contre {TOTAL_AU_2026_09_08} au \
         2026-09-08.\n\n\
         Le cliquet de taxonomie se satisferait d'une SUPPRESSION de tests : \
         moins de tests, donc moins de tests sans catégorie. Ce contrôle \
         l'interdit.\n\n\
         Si des tests ont été légitimement supprimés — un module retiré, un \
         doublon fusionné — abaissez cette constante en disant lesquels."
    );
}
