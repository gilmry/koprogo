//! Cliquet : une énumération Rust et sa contrainte SQL disent la MÊME chose.
//!
//! ── Le défaut que ce cliquet ferme ────────────────────────────────────────
//!
//! Le 2026-09-16, `bdd_financial` et les scénarios de rapport prestataire
//! échouaient en CI sur :
//!
//! ```text
//! new row for relation "magic_links" violates check constraint
//! "magic_links_scope_kind_check"
//! ```
//!
//! #835 avait ajouté `MagicLinkScopeKind::ContractorReport` au domaine Rust.
//! La contrainte `CHECK` de la migration de 2026-06-05, elle, était restée
//! aux quatre portées d'origine. Le code compilait, les 2088 tests unitaires
//! passaient — la variante existe bel et bien côté Rust — et c'est la BASE
//! qui refusait à l'exécution.
//!
//! ── Pourquoi aucune revue ne pouvait le voir ──────────────────────────────
//!
//! La branche qui ajoute la variante ne touche pas aux migrations. Celle qui
//! aurait dû écrire la migration n'existe pas. Rien ne relie les deux, et le
//! défaut n'apparaît qu'à la JONCTION — c'est le quatrième de cette famille
//! dans la même fusion, après le DTO sans `ToSchema`, un test BDD qui n'avait
//! pas suivi un changement de signature, et une table `funds` absente.
//!
//! ── Ce que ce cliquet mesure, et ce qu'il ne mesure pas ───────────────────
//!
//! Il compare, pour chaque paire déclarée, les variantes de l'énumération aux
//! valeurs de la contrainte `CHECK` la plus RÉCENTE qui la porte. Il lit du
//! texte, pas du SQL exécuté : une contrainte ajoutée par un chemin qu'il ne
//! connaît pas lui échapperait.
//!
//! Il ne dit donc pas « le schéma est juste ». Il dit « ces deux copies de la
//! même vérité ne divergent pas », ce qui est exactement le défaut observé.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Les paires à tenir ensemble : (fichier Rust, énum, nom de la contrainte).
///
/// Volontairement explicite. Une découverte automatique paraîtrait plus
/// élégante et se tairait le jour où la convention de nommage change — et un
/// cliquet qui se tait est pire qu'absent.
const PAIRES: &[(&str, &str, &str)] = &[
    (
        "src/domain/plateforme/magic_link.rs",
        "MagicLinkScopeKind",
        "magic_links_scope_kind_check",
    ),
    // Story 5.1 (#585) — le registre de modules. L'enum et la contrainte
    // portent la même liste à deux endroits ; sans cette paire, l'une
    // pourrait devancer l'autre sans que rien ne le dise.
    (
        "src/domain/copropriete/acp_enabled_module.rs",
        "Module",
        "acp_enabled_modules_module_check",
    ),
    // Story 4.6 (#581) — l'enum a vécu six jours SANS sa colonne. Le dépôt
    // de résolutions emploie des requêtes vérifiées à l'exécution : rien ne
    // pouvait le dire avant qu'un appel réel échoue. Cette paire est le
    // rattrapage.
    (
        "src/domain/copropriete/resolution.rs",
        "ResolutionKind",
        "resolutions_kind_check",
    ),
];

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Les variantes d'une énumération, converties en `snake_case` — la
/// convention que ce dépôt emploie pour les sérialiser en base.
fn variantes(chemin: &str, nom_enum: &str) -> BTreeSet<String> {
    let source = fs::read_to_string(racine().join(chemin)).expect("source lisible");
    let debut = source
        .find(&format!("pub enum {nom_enum} {{"))
        .unwrap_or_else(|| panic!("énumération {nom_enum} introuvable dans {chemin}"));
    let corps = &source[debut..];
    let fin = corps.find("\n}").expect("fin d'énumération");
    let mut trouvees = BTreeSet::new();
    for ligne in corps[..fin].lines().skip(1) {
        let l = ligne.trim();
        if l.is_empty() || l.starts_with("//") || l.starts_with("#[") {
            continue;
        }
        let Some(variante) = l.split(&[',', '(', '{'][..]).next() else {
            continue;
        };
        let variante = variante.trim();
        if variante.is_empty() || !variante.chars().next().is_some_and(char::is_uppercase) {
            continue;
        }
        let mut snake = String::new();
        for (i, c) in variante.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                snake.push('_');
            }
            snake.extend(c.to_lowercase());
        }
        trouvees.insert(snake);
    }
    trouvees
}

/// Les valeurs de la contrainte `CHECK`, prises dans la migration la plus
/// RÉCENTE qui la nomme — une contrainte peut être remplacée par un
/// `DROP` + `ADD`, et c'est le dernier mot qui compte.
fn valeurs_contrainte(nom_contrainte: &str) -> BTreeSet<String> {
    let dossier = racine().join("migrations");
    let mut fichiers: Vec<PathBuf> = fs::read_dir(&dossier)
        .expect("migrations lisibles")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|x| x == "sql") && !p.to_string_lossy().ends_with(".down.sql")
        })
        .collect();
    fichiers.sort();

    let mut dernieres = BTreeSet::new();
    for chemin in fichiers {
        let sql = fs::read_to_string(&chemin).unwrap_or_default();
        // La contrainte peut être nommée explicitement, ou déduite par
        // Postgres du nom de colonne dans un CREATE TABLE.
        let pertinent = sql.contains(nom_contrainte)
            || (nom_contrainte.ends_with("_check") && sql.contains("scope_kind      VARCHAR"));
        if !pertinent {
            continue;
        }
        let Some(pos) = sql.rfind(" IN (") else {
            continue;
        };
        let reste = &sql[pos + 5..];
        let Some(fin) = reste.find(')') else { continue };

        // Les commentaires SQL sont retirés AVANT le découpage.
        //
        // Découper d'abord sur les virgules et filtrer ensuite ne suffit
        // pas : un commentaire qui contient une virgule — « même page, même
        // paramètre `t` » — se coupe en deux, et sa seconde moitié ressemble
        // à une valeur. Le cliquet a signalé cette faute sur lui-même à sa
        // première exécution, ce qui est exactement ce qu'on lui demande.
        let sans_commentaires: String = reste[..fin]
            .lines()
            .map(|l| match l.find("--") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut valeurs = BTreeSet::new();
        for brut in sans_commentaires.split(',') {
            let v = brut.trim().trim_matches('\'').trim();
            if !v.is_empty() {
                valeurs.insert(v.to_string());
            }
        }
        if !valeurs.is_empty() {
            dernieres = valeurs;
        }
    }
    dernieres
}

#[test]
fn le_cliquet_lit_bien_quelque_chose() {
    // Vérification d'aveuglement : si la lecture casse, la règle suivante
    // devient vraie sur deux ensembles vides, et un vert obtenu sur rien ne
    // dit rien.
    for (chemin, nom_enum, contrainte) in PAIRES {
        let v = variantes(chemin, nom_enum);
        let c = valeurs_contrainte(contrainte);
        assert!(!v.is_empty(), "aucune variante lue pour {nom_enum}");
        assert!(
            !c.is_empty(),
            "aucune valeur lue pour la contrainte {contrainte} — le motif de \
             lecture ne correspond plus aux migrations"
        );
    }
}

#[test]
fn security_chaque_variante_rust_est_acceptee_par_la_base() {
    for (chemin, nom_enum, contrainte) in PAIRES {
        let rust = variantes(chemin, nom_enum);
        let sql = valeurs_contrainte(contrainte);
        let orphelines: Vec<&String> = rust.difference(&sql).collect();

        assert!(
            orphelines.is_empty(),
            "{nom_enum} déclare des variantes que `{contrainte}` REFUSE :\n{}\n\n\
             Le code compilera, les tests unitaires passeront, et la base \
             rejettera l'écriture à l'exécution :\n\n  \
             new row violates check constraint \"{contrainte}\"\n\n\
             C'est exactement ce qui est arrivé le 2026-09-16 avec \
             `contractor_report` (#835). Une énumération Rust et une \
             contrainte CHECK sont deux copies de la même vérité ; ce cliquet \
             existe parce que rien d'autre ne les tient ensemble.\n\n\
             Écrivez la migration qui étend la contrainte.",
            orphelines
                .iter()
                .map(|v| format!("  {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn edge_la_base_naccepte_rien_que_le_code_ignore() {
    // L'inverse compte aussi, pour une autre raison : une valeur acceptée par
    // la base et inconnue du code est une porte que personne ne surveille.
    // Elle peut venir d'une variante supprimée sans migration de retrait.
    for (chemin, nom_enum, contrainte) in PAIRES {
        let rust = variantes(chemin, nom_enum);
        let sql = valeurs_contrainte(contrainte);
        let fantomes: Vec<&String> = sql.difference(&rust).collect();

        assert!(
            fantomes.is_empty(),
            "`{contrainte}` accepte des valeurs que {nom_enum} ne connaît \
             pas :\n{}\n\n\
             Une valeur acceptée par la base et absente du code est une \
             porte que personne ne surveille : rien ne la lira, et rien ne \
             dira qu'elle a été écrite.",
            fantomes
                .iter()
                .map(|v| format!("  {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
