//! Cliquet — toute colonne écrite par un dépôt existe dans les migrations.
//!
//! ## Le défaut que ce cliquet ferme
//!
//! Le 2026-09-16, **quatre** dépôts écrivaient dans des colonnes qu'aucune
//! migration ne crée :
//!
//! | Dépôt | Ce qui manquait | Symptôme |
//! |---|---|---|
//! | `resolution_repository_impl` | `resolutions.kind` (#581) | 400 sur toute création de résolution |
//! | `vote_repository_impl` | `votes.auth_method` (#48) | 500 sur `GET /resolutions/{id}/votes` |
//! | `fund_repository_impl` | la table `fund_reassignments` entière | toute réaffectation échouait |
//! | `resource_booking_repository_impl` | 3 colonnes (#588) | 400 sur toute réservation |
//!
//! Le point commun n'est pas la négligence, c'est l'**outillage** : ces
//! quatre dépôts emploient `sqlx::query(...)` — vérifié à l'EXÉCUTION — et
//! non la macro `sqlx::query!`, vérifiée à la compilation contre le cache
//! `.sqlx`. Le compilateur ne lit pas ces chaînes. Rien, dans toute la
//! chaîne de gates, ne pouvait le dire avant qu'un appel réel échoue.
//!
//! Deux de ces quatre ont été trouvés par hasard, au détour d'une campagne
//! e2e. Les deux autres par une battue lancée parce que le hasard avait
//! frappé deux fois. Ce fichier remplace le hasard.
//!
//! ## Ce qu'il lit, et ce qu'il ne lit pas
//!
//! Il compare les **listes de colonnes d'un `INSERT INTO`** au schéma reconstruit
//! depuis `migrations/*.sql`. Ni base de données, ni réseau : il tourne en CI
//! comme en local.
//!
//! Volontairement borné aux `INSERT` : une liste d'insertion ne contient que
//! de vrais noms de colonnes, alors qu'un `SELECT` porte des alias, des
//! agrégats et des expressions qu'il faudrait un vrai analyseur SQL pour
//! démêler. Un cliquet qui crie au loup finit ignoré. Les quatre défauts
//! ci-dessus sont tous visibles dans un `INSERT` — la borne ne coûte rien
//! sur ce qu'on cherche à attraper.
//!
//! Il ne remplace donc pas une vérification à la compilation : il attrape la
//! faute la plus grossière et la plus coûteuse, pas toutes.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Retire les commentaires `--` jusqu'à la fin de ligne.
///
/// Indispensable AVANT toute découpe : les migrations de ce dépôt sont
/// abondamment commentées, et un `-- ... , ...` ferait apparaître des
/// colonnes fantômes. C'est la faute qu'a commise `garde_enum_contre_contrainte`
/// à sa première exécution.
fn sans_commentaires(sql: &str) -> String {
    sql.lines()
        .map(|l| match l.find("--") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Découpe sur les virgules de PROFONDEUR ZÉRO.
///
/// `NUMERIC(14, 2)` et `CHECK (x IN ('a', 'b'))` portent des virgules qui
/// n'introduisent pas de colonne. Découper naïvement les compterait.
fn separer_au_premier_niveau(contenu: &str) -> Vec<String> {
    let mut morceaux = Vec::new();
    let mut courant = String::new();
    let mut profondeur = 0i32;
    let mut dans_chaine = false;
    for c in contenu.chars() {
        match c {
            '\'' => {
                dans_chaine = !dans_chaine;
                courant.push(c);
            }
            '(' if !dans_chaine => {
                profondeur += 1;
                courant.push(c);
            }
            ')' if !dans_chaine => {
                profondeur -= 1;
                courant.push(c);
            }
            ',' if profondeur == 0 && !dans_chaine => {
                morceaux.push(courant.trim().to_string());
                courant.clear();
            }
            _ => courant.push(c),
        }
    }
    if !courant.trim().is_empty() {
        morceaux.push(courant.trim().to_string());
    }
    morceaux
}

/// Le contenu entre la parenthèse ouvrante qui suit `depuis` et sa fermante.
fn contenu_parenthese(sql: &str, depuis: usize) -> Option<(String, usize)> {
    let ouvrante = sql[depuis..].find('(')? + depuis;
    let mut profondeur = 0i32;
    let mut dans_chaine = false;
    for (i, c) in sql[ouvrante..].char_indices() {
        match c {
            '\'' => dans_chaine = !dans_chaine,
            '(' if !dans_chaine => profondeur += 1,
            ')' if !dans_chaine => {
                profondeur -= 1;
                if profondeur == 0 {
                    return Some((sql[ouvrante + 1..ouvrante + i].to_string(), ouvrante + i));
                }
            }
            _ => {}
        }
    }
    None
}

/// Les mots qui ouvrent une clause de TABLE et non une colonne.
const CLAUSES: &[&str] = &[
    "constraint",
    "primary",
    "unique",
    "foreign",
    "check",
    "exclude",
    "like",
];

fn premier_mot(s: &str) -> String {
    s.trim()
        .trim_start_matches('"')
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

/// Reconstruit `table -> colonnes` depuis les migrations montantes, lues
/// dans l'ordre des versions.
fn schema_declare() -> BTreeMap<String, BTreeSet<String>> {
    let mut fichiers: Vec<_> = fs::read_dir(racine().join("migrations"))
        .expect("dossier migrations lisible")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".sql") && !n.ends_with(".down.sql"))
        .collect();
    fichiers.sort();

    let mut schema: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for nom in fichiers {
        let sql = sans_commentaires(
            &fs::read_to_string(racine().join("migrations").join(&nom)).expect("migration lisible"),
        );
        let bas = sql.to_lowercase();

        // CREATE TABLE [IF NOT EXISTS] <nom> ( ... )
        let mut depuis = 0usize;
        while let Some(i) = bas[depuis..].find("create table") {
            let debut = depuis + i;
            let apres = &sql[debut + "create table".len()..];
            let table = apres
                .split_whitespace()
                .find(|m| {
                    let b = m.to_lowercase();
                    b != "if" && b != "not" && b != "exists"
                })
                .map(|m| {
                    m.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                        .to_lowercase()
                })
                .unwrap_or_default();
            if let Some((contenu, fin)) = contenu_parenthese(&sql, debut) {
                let e = schema.entry(table).or_default();
                for morceau in separer_au_premier_niveau(&contenu) {
                    let mot = premier_mot(&morceau);
                    if !mot.is_empty() && !CLAUSES.contains(&mot.as_str()) {
                        e.insert(mot);
                    }
                }
                depuis = fin;
            } else {
                depuis = debut + "create table".len();
            }
        }

        // ALTER TABLE <nom> ... ADD/DROP/RENAME COLUMN
        let mut depuis = 0usize;
        while let Some(i) = bas[depuis..].find("alter table") {
            let debut = depuis + i;
            let apres = &sql[debut + "alter table".len()..];
            let table = apres
                .split_whitespace()
                .find(|m| {
                    let b = m.to_lowercase();
                    b != "if" && b != "exists" && b != "only"
                })
                .map(|m| {
                    m.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                        .to_lowercase()
                })
                .unwrap_or_default();
            let fin = sql[debut..]
                .find(';')
                .map(|f| debut + f)
                .unwrap_or(sql.len());
            let corps = &sql[debut..fin];
            let e = schema.entry(table).or_default();
            let bas_corps = corps.to_lowercase();
            for (pos, _) in bas_corps.match_indices("add column") {
                let reste = &corps[pos + "add column".len()..];
                let mot = reste
                    .split_whitespace()
                    .find(|m| {
                        let b = m.to_lowercase();
                        b != "if" && b != "not" && b != "exists"
                    })
                    .map(premier_mot)
                    .unwrap_or_default();
                if !mot.is_empty() {
                    e.insert(mot);
                }
            }
            for (pos, _) in bas_corps.match_indices("drop column") {
                let reste = &corps[pos + "drop column".len()..];
                let mot = reste
                    .split_whitespace()
                    .find(|m| {
                        let b = m.to_lowercase();
                        b != "if" && b != "exists"
                    })
                    .map(premier_mot)
                    .unwrap_or_default();
                e.remove(&mot);
            }
            for (pos, _) in bas_corps.match_indices("rename column") {
                let reste = &corps[pos + "rename column".len()..];
                let mots: Vec<String> = reste.split_whitespace().map(premier_mot).collect();
                if mots.len() >= 3 {
                    e.remove(&mots[0]);
                    e.insert(mots[2].clone());
                }
            }
            depuis = fin.max(debut + "alter table".len());
        }
    }
    schema
}

/// `(fichier, table, colonne)` pour chaque colonne d'un `INSERT INTO`.
fn colonnes_inserees() -> Vec<(String, String, String)> {
    let mut trouvees = Vec::new();
    let dossier = racine().join("src/infrastructure/database/repositories");
    for entree in fs::read_dir(&dossier).expect("dossier repositories lisible") {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let nom = chemin
            .file_name()
            .expect("nom de fichier")
            .to_string_lossy()
            .to_string();
        let source = fs::read_to_string(&chemin).expect("source lisible");
        let bas = source.to_lowercase();
        let mut depuis = 0usize;
        while let Some(i) = bas[depuis..].find("insert into") {
            let debut = depuis + i;
            let apres = &source[debut + "insert into".len()..];
            let table = apres
                .split_whitespace()
                .next()
                .map(premier_mot)
                .unwrap_or_default();
            match contenu_parenthese(&source, debut) {
                Some((contenu, fin)) => {
                    // Un `INSERT INTO t SELECT ...` n'a pas de liste : la
                    // parenthèse trouvée serait celle d'autre chose. On ne
                    // garde que les listes d'identifiants simples.
                    let morceaux = separer_au_premier_niveau(&contenu);
                    let simple = morceaux.iter().all(|m| {
                        let t = m.trim();
                        !t.is_empty()
                            && t.chars()
                                .all(|c| c.is_alphanumeric() || c == '_' || c.is_whitespace())
                            && t.split_whitespace().count() == 1
                    });
                    if simple {
                        for m in morceaux {
                            let col = premier_mot(&m);
                            if !col.is_empty() {
                                trouvees.push((nom.clone(), table.clone(), col));
                            }
                        }
                    }
                    depuis = fin;
                }
                None => depuis = debut + "insert into".len(),
            }
        }
    }
    trouvees
}

#[test]
fn security_chaque_colonne_inseree_existe_dans_les_migrations() {
    let schema = schema_declare();
    let mut manquantes: Vec<String> = Vec::new();

    for (fichier, table, colonne) in colonnes_inserees() {
        match schema.get(&table) {
            None => manquantes.push(format!(
                "  {fichier} → INSERT INTO {table} — cette TABLE n'est créée par aucune migration"
            )),
            Some(colonnes) if !colonnes.contains(&colonne) => manquantes.push(format!(
                "  {fichier} → {table}.{colonne} — aucune migration ne crée cette colonne"
            )),
            Some(_) => {}
        }
    }
    manquantes.sort();
    manquantes.dedup();

    assert!(
        manquantes.is_empty(),
        "Des dépôts écrivent dans des colonnes qui n'existent pas.\n\n{}\n\n\
         Ces dépôts emploient `sqlx::query(...)`, vérifié à l'EXÉCUTION : le \
         compilateur ne lit pas ce SQL, et l'erreur n'apparaît qu'au premier \
         appel réel — en 400 ou en 500, chez l'utilisateur.\n\n\
         Écrivez la migration manquante. Ne retirez pas la colonne du code \
         sans avoir vérifié que la fonctionnalité ne s'en sert pas : quatre \
         stories livrées ont été trouvées ainsi le 2026-09-16, toutes \
         fonctionnelles côté Rust et toutes cassées côté base.",
        manquantes.join("\n")
    );
}

#[test]
fn le_cliquet_lit_bien_quelque_chose() {
    // Un cliquet qui ne lit rien passe au vert pour de mauvaises raisons.
    let schema = schema_declare();
    assert!(
        schema.len() > 40,
        "Seulement {} tables reconstruites depuis les migrations — l'analyse \
         a probablement échoué en silence.",
        schema.len()
    );
    let inserees = colonnes_inserees();
    assert!(
        inserees.len() > 200,
        "Seulement {} colonnes d'INSERT relevées dans les dépôts — l'analyse \
         a probablement échoué en silence.",
        inserees.len()
    );
}

#[test]
fn edge_le_schema_reconstruit_suit_les_alter_table() {
    // Témoin ciblé : `funds` est créée par une migration, et `call_for_funds`
    // reçoit `fund_id` par un ALTER dans la MÊME migration. Si l'analyse
    // ignorait les ALTER, cette colonne manquerait.
    let schema = schema_declare();
    let funds = schema.get("funds").expect("table funds reconstruite");
    assert!(funds.contains("kind"), "funds.kind attendu, vu : {funds:?}");

    let cff = schema
        .get("call_for_funds")
        .expect("table call_for_funds reconstruite");
    assert!(
        cff.contains("fund_id"),
        "call_for_funds.fund_id vient d'un ALTER TABLE — s'il manque, \
         l'analyse ne suit pas les ALTER. Vu : {cff:?}"
    );
}
