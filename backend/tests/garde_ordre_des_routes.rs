//! Une route littérale n'est jamais captée par une route paramétrée.
//!
//! ── Le défaut ──────────────────────────────────────────────────────────────
//!
//! `actix-web` apparie les routes **dans l'ordre d'enregistrement**. Si
//! `/accounts/{id}` est déclarée avant `/accounts/count`, la seconde n'est
//! jamais atteinte : le serveur tente de lire « count » comme un UUID et rend
//! un 400 sur identifiant mal formé.
//!
//! La route existe, elle est documentée, son handler est écrit et testé
//! unitairement. Elle est simplement **inatteignable** — encore le motif
//! dominant de ce dépôt, cette fois causé par un ordre de deux lignes.
//!
//! Deux routes étaient dans ce cas au 2026-09-10, et rien ne les signalait :
//!
//!     GET /accounts/count           capté par /accounts/{id}
//!     GET /call-for-funds/overdue   capté par /call-for-funds/{id}
//!
//! ── La nuance qui évite un faux constat ───────────────────────────────────
//!
//! `actix` apparie la **méthode HTTP** avant le chemin. Un `POST /polls/vote`
//! ne peut donc pas être capté par un `GET /polls/{id}`.
//!
//! Sans cette nuance, la mesure donnait NEUF routes fautives au lieu de deux :
//! j'ai failli publier un constat trois fois trop gros. Une garde qui crie sur
//! du code correct se fait désactiver, et emporte avec elle les cas réels.
//!
//! ── Ce que cette garde ne peut pas voir ───────────────────────────────────
//!
//! Elle lit les attributs `#[get("…")]` des handlers et les `.service(…)` de
//! `routes.rs`. Une route montée autrement — `web::resource()`, un scope
//! imbriqué construit dynamiquement — lui échappe. Le relevé porte sur ce qui
//! est déclaré de façon statique, ce qui est le cas de toutes les routes du
//! dépôt aujourd'hui.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            sortie.push(chemin);
        }
    }
}

/// `(nom du handler) → (méthode, chemin)`, lu sur les attributs de route.
fn handlers() -> HashMap<String, (String, String)> {
    let mut trouves = HashMap::new();
    let mut fichiers = Vec::new();
    fichiers_rust(
        &racine().join("src/infrastructure/web/handlers"),
        &mut fichiers,
    );

    for fichier in fichiers {
        let Ok(source) = fs::read_to_string(&fichier) else {
            continue;
        };
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            let nue = ligne.trim();
            let Some(methode) = ["get", "post", "put", "delete", "patch"]
                .iter()
                .find(|m| nue.starts_with(&format!("#[{m}(\"")))
            else {
                continue;
            };
            let Some(chemin) = nue
                .split_once('"')
                .and_then(|(_, r)| r.split_once('"'))
                .map(|(c, _)| c.to_string())
            else {
                continue;
            };
            // Le nom de la fonction suit, à quelques lignes près (attributs
            // `utoipa` intercalés).
            for suivante in lignes.iter().skip(i + 1).take(6) {
                if let Some(reste) = suivante.trim().strip_prefix("pub async fn ") {
                    if let Some(nom) = reste.split(['(', '<']).next() {
                        trouves.insert(nom.to_string(), (methode.to_string(), chemin));
                    }
                    break;
                }
            }
        }
    }
    trouves
}

/// `(nom du handler) → position de son `.service(…)` dans `routes.rs`.
fn ordre_denregistrement() -> HashMap<String, usize> {
    let source = fs::read_to_string(racine().join("src/infrastructure/web/routes.rs"))
        .expect("routes.rs doit être lisible");
    let mut positions = HashMap::new();
    let mut curseur = 0usize;
    while let Some(debut) = source[curseur..].find(".service(") {
        let absolu = curseur + debut + ".service(".len();
        if let Some(fin) = source[absolu..].find(')') {
            let nom = source[absolu..absolu + fin].trim().to_string();
            positions.entry(nom).or_insert(absolu);
            curseur = absolu + fin;
        } else {
            break;
        }
    }
    positions
}

/// `/accounts/{id}` correspond-elle au préfixe de `/accounts/count` ?
fn est_le_parametre_du_prefixe(chemin_parametre: &str, prefixe: &str) -> bool {
    let Some(reste) = chemin_parametre.strip_prefix(prefixe) else {
        return false;
    };
    reste.starts_with("/{") && reste.ends_with('}') && !reste[2..].contains('/')
}

#[test]
fn aucune_route_litterale_nest_captee_par_une_route_parametree() {
    let handlers = handlers();
    let ordre = ordre_denregistrement();
    let mut fautives: Vec<String> = Vec::new();

    for (nom_litteral, (methode, chemin)) in &handlers {
        // Uniquement les chemins littéraux à deux segments : `/accounts/count`.
        if chemin.contains('{') || chemin.matches('/').count() != 2 {
            continue;
        }
        let Some(prefixe) = chemin.rsplit_once('/').map(|(p, _)| p) else {
            continue;
        };

        for (nom_parametre, (methode_p, chemin_p)) in &handlers {
            // MÊME méthode : `actix` apparie la méthode avant le chemin, et
            // sans cette condition la mesure quadruple en faux positifs.
            if methode_p != methode || !est_le_parametre_du_prefixe(chemin_p, prefixe) {
                continue;
            }
            let (Some(&pos_litterale), Some(&pos_parametree)) =
                (ordre.get(nom_litteral), ordre.get(nom_parametre))
            else {
                continue;
            };
            if pos_parametree < pos_litterale {
                fautives.push(format!(
                    "  {} {chemin} ({nom_litteral}) est enregistrée APRÈS {chemin_p} ({nom_parametre})",
                    methode.to_uppercase()
                ));
            }
        }
    }

    fautives.sort();
    fautives.dedup();
    assert!(
        fautives.is_empty(),
        "des routes littérales sont captées par une route paramétrée :\n{}\n\n\
         `actix-web` apparie dans l'ordre d'enregistrement : la route littérale \
         n'est jamais atteinte, et le serveur tente de lire son dernier segment \
         comme un UUID. Il rend alors un 400 sur identifiant mal formé, ce qui \
         ne ressemble en rien au vrai problème.\n\n\
         Déplacez le `.service(…)` littéral AVANT le paramétré dans \
         `routes.rs`.",
        fautives.join("\n")
    );
}

#[test]
fn la_garde_lit_bien_les_routes_et_leur_ordre() {
    // Vérification d'aveuglement : si le format des attributs ou des
    // `.service(…)` changeait, les deux tables seraient vides et le test
    // ci-dessus passerait sans rien comparer. Son zéro voudrait alors dire
    // « je n'ai rien regardé » plutôt que « rien à signaler ».
    let handlers = handlers();
    let ordre = ordre_denregistrement();

    assert!(
        handlers.len() > 200,
        "seulement {} routes relevées : le motif des attributs a changé",
        handlers.len()
    );
    assert!(
        ordre.len() > 200,
        "seulement {} enregistrements relevés : le motif `.service(…)` a changé",
        ordre.len()
    );

    // Et le rapprochement doit fonctionner : au moins un handler doit être à
    // la fois déclaré et enregistré.
    let apparies = handlers.keys().filter(|h| ordre.contains_key(*h)).count();
    assert!(
        apparies > 200,
        "seulement {apparies} handlers rapprochés de leur enregistrement"
    );
}
